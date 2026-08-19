# Tasks: Billing Cron Fixes

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~380–440 |
| 400-line budget risk | Medium |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | single-pr-default (800-line session budget) |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Medium

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| 1 | R1–R6: cron fixes + alert + tests | Single PR | `bun run test -- src/test/payments/cron-billing.test.ts` (in `web/`) | N/A — crons are `CRON_SECRET`-authed HTTP endpoints; tested with mocked db/lifecycle/email, no live gateway creds | Revert `web/` cron handlers + `plans.ts` + `cron-alert.ts`; no DDL/migration |

## Phase 1: Foundation

- [x] 1.1 Add `getPlanById(planId: string): Plan | undefined` to `web/src/data/plans.ts` (PLANS.find; basico 89900 / pro 149900 / premium 259900).
- [x] 1.2 Refactor `web/src/pages/api/checkout/wompi.ts:28` and `web/src/pages/api/checkout/mercadopago.ts:30`: replace inline `PLANS.find` with `getPlanById(planId)`.
- [x] 1.3 Create `web/src/lib/payments/cron-alert.ts` — `sendCronAlert(handler, err)`: `console.error('[cron:<handler>] FAILED:', err)` + `sendEmail({ to: SUPPORT_EMAIL || 'support@academix.app' })` (`web/src/lib/email.ts:26`), wrapped in its own try/catch so email failure never masks the 500.

## Phase 2: Core Implementation

- [x] 2.1 `web/src/pages/api/cron/charge-wompi.ts`: interface :8-15 → `{id, user_id, plan_id, payment_source_token, email}`; SQL :40 `s.plan` → `s.plan_id`; delete `getPlanPriceCOP` (:92-98); loop (:62-81): `getPlanById(sub.plan_id)` — unknown plan → log + `startGracePeriod` + failed++ (no silent default price); else `createTransaction(plan.priceCOP * 100, 'COP', 'renewal-'+sub.id+'-'+Date.now(), token, email)` → charged++; catch → grace + failed++.
- [x] 2.2 `web/src/pages/api/cron/charge-wompi.ts`: wrap body (after auth checks :22-34) in top-level try/catch → `sendCronAlert('charge-wompi', err)` + 500; missing `CRON_SECRET` stays 500, bad auth stays 401 — both outside catch, no alert.
- [x] 2.3 `web/src/pages/api/cron/send-reminders.ts`: :75/:79 `s.grace_end` → `s.grace_expires_at`, :84 `row.grace_end` → `row.grace_expires_at`; trial block :41-71 unchanged; top-level try/catch → `sendCronAlert('send-reminders')` + 500 (per-sub catches :68-70/:100-102 stay local).
- [x] 2.4 `web/src/pages/api/cron/expire-subscriptions.ts`: import `cancelSubscription`; grace loop :31-33 → `cancelSubscription(sub.id)` (was `expireSubscription`); trial loop :36-39 unchanged; top-level try/catch → `sendCronAlert('expire-subscriptions')` + 500.

## Phase 3: Testing

- [x] 3.1 `web/src/test/payments.test.ts` :196-227: fixture `plan`/`grace_start`/`grace_end` → `plan_id`/`grace_expires_at` (real columns).
- [x] 3.2 `web/src/test/payments/subscription-lifecycle.test.ts` :76-83, :155-163, :201-209, :221-229: drop stale `plan`/`grace_start`/`grace_end`; keep real `grace_expires_at`.
- [x] 3.3 Create `web/src/test/payments/cron-billing.test.ts` (vi.mock pattern per `webhook-flows.test.ts`): R1 charge SQL asserts `s.plan_id` + filters (provider/status/period/token), amount 149900×100 for 'pro' equals PLANS; R2 unknown plan/throw → `startGracePeriod`; R3 send-reminders asserts `s.grace_expires_at`, `graceWarnings=1`; none → 0/200; R4 grace → `cancelSubscription`, trial → `expireSubscription`; R5 db reject → 500 + `sendEmail` spy, 401 → no email; `getPlanById('pro')` → 149900, unknown → undefined.

## Phase 4: Verification

- [x] 4.1 `bun run build` in `web/` (astro type-safe check) — passes.
- [x] 4.2 `bun run test` in `web/` — new + updated suites green; note pre-existing unrelated failures (e.g. stale checkout-integration Stripe test) without blocking.