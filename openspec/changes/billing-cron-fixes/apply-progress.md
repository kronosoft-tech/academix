# Apply Progress: Billing Cron Fixes

**Status:** Complete — 12/12 tasks implemented (hybrid persistence: Engram topic `sdd/billing-cron-fixes/apply-progress` + this file).
**Mode:** Standard (strict_tdd: false). Work unit evidence recorded below.
**Delivery:** single-pr-default (forecast ~380–440 lines, Medium risk, no chain decision required).

## Work Unit Evidence

| Evidence | Value |
|----------|-------|
| Focused test command + result | `bun run test -- src/test/payments/cron-billing.test.ts src/test/payments/subscription-lifecycle.test.ts` → 25 passed (12 new cron + 13 lifecycle), 0 failed |
| Runtime harness command + result | `bun run build` in `web/` → astro build Complete (0 errors). Crons are CRON_SECRET-authed HTTP endpoints; handler behavior exercised in tests via mocked db/lifecycle/email — no live gateway creds (N/A for live E2E). |
| Rollback boundary | Revert `web/src/pages/api/cron/{charge-wompi,send-reminders,expire-subscriptions}.ts`, `web/src/data/plans.ts`, `web/src/lib/payments/cron-alert.ts`, checkout refactors, and the 3 test files — no DDL/migration involved |

## Completed Tasks

- [x] 1.1 `getPlanById(planId)` added to `web/src/data/plans.ts` — single source of truth (basico 89900 / pro 149900 / premium 259900).
- [x] 1.2 Checkouts `web/src/pages/api/checkout/{wompi,mercadopago}.ts` refactored to `getPlanById` (inline `PLANS.find` removed).
- [x] 1.3 `web/src/lib/payments/cron-alert.ts` created — `sendCronAlert(handler, err)`: prominent `[cron:<handler>] FAILED:` log + `sendEmail({ to: SUPPORT_EMAIL || 'support@academix.app' })`, own try/catch (never masks the 500).
- [x] 2.1 `charge-wompi.ts` — interface `{id, user_id, plan_id, payment_source_token, email}`; SQL `s.plan` → `s.plan_id` with filters `provider='wompi' AND status='active' AND current_period_end <= ? AND payment_source_token IS NOT NULL`; `getPlanPriceCOP` deleted; loop uses `getPlanById` — unknown plan → log + `startGracePeriod` + failed++ (no silent default); else `createTransaction(plan.priceCOP*100, 'COP', 'renewal-<id>-<ts>', token, email)` → charged++; per-sub catch → grace + failed++.
- [x] 2.2 `charge-wompi.ts` — body wrapped in top-level try/catch → `sendCronAlert('charge-wompi', err)` + 500; auth checks outside catch (missing CRON_SECRET 500 / bad auth 401, no alert).
- [x] 2.3 `send-reminders.ts` — `s.grace_end` → `s.grace_expires_at` (SELECT :75, WHERE :79, row :84); trial block unchanged; top-level try/catch → `sendCronAlert('send-reminders')` + 500; per-sub catches stay local.
- [x] 2.4 `expire-subscriptions.ts` — grace loop → `cancelSubscription(sub.id)` (was `expireSubscription`); trial loop unchanged; top-level try/catch → `sendCronAlert('expire-subscriptions')` + 500.
- [x] 3.1 `web/src/test/payments.test.ts` :196-227 fixture → real columns `plan_id` + `grace_expires_at` (dropped `plan`/`grace_start`/`grace_end`).
- [x] 3.2 `web/src/test/payments/subscription-lifecycle.test.ts` — 4 fixture sites (:76-83, :155-163, :201-209, :221-229) use real columns only.
- [x] 3.3 `web/src/test/payments/cron-billing.test.ts` created (webhook-flows vi.mock pattern) — R1 SQL asserts `s.plan_id` + filters, amount 149900×100 for 'pro'; R2 unknown plan / throw → `startGracePeriod`; R3 `s.grace_expires_at`, `graceWarnings=1`, none → 0/200; R4 grace → `cancelSubscription`, trial → `expireSubscription`; R5 db reject → 500 + email spy, 401 → no email, missing secret → 500 no email; R6 `getPlanById('pro')` → 149900, unknown → undefined.
- [x] 4.1 `bun run build` in `web/` — passes.
- [x] 4.2 `bun run test` in `web/` — 192 passed / 7 failed. All 7 failures are PRE-EXISTING and unrelated (5× `payments.test.ts` importing non-existent `lib/payments/stripe`; 2× `checkout-integration.test.ts` expecting Stripe gateway routing) — identical to the baseline run before this change.

## Verification Results

- `bun run test -- src/test/payments/cron-billing.test.ts src/test/payments/subscription-lifecycle.test.ts` → 25/25 passed
- `bun run test` (full) → 192 passed, 7 pre-existing failures (unchanged baseline), 0 regressions
- `bun run build` → astro build Complete
- `bunx tsc --noEmit` → only 7 pre-existing errors remain (5 stale stripe imports, 1 stale checkout-integration comparison, 1 `lib/db.ts` cast) — zero new type errors from this change

## Files Changed

| File | Action | What Was Done |
|------|--------|---------------|
| `web/src/data/plans.ts` | Modified | Added `getPlanById()` |
| `web/src/pages/api/checkout/wompi.ts` | Modified | Uses `getPlanById` |
| `web/src/pages/api/checkout/mercadopago.ts` | Modified | Uses `getPlanById` |
| `web/src/lib/payments/cron-alert.ts` | Created | `sendCronAlert` (log + email, never throws) |
| `web/src/pages/api/cron/charge-wompi.ts` | Modified | `s.plan_id` + PLANS prices + per-sub grace + top-level alert |
| `web/src/pages/api/cron/send-reminders.ts` | Modified | `s.grace_expires_at` + top-level alert |
| `web/src/pages/api/cron/expire-subscriptions.ts` | Modified | grace → `cancelSubscription` + top-level alert |
| `web/src/test/payments.test.ts` | Modified | Real-column fixture |
| `web/src/test/payments/subscription-lifecycle.test.ts` | Modified | Real-column fixtures (4 sites) |
| `web/src/test/payments/cron-billing.test.ts` | Created | R1–R6 cron tests |

## Deviations from Design

None — implementation matches design.md. (Minor: `getPlanById` call sites narrow `planId` before calling so the helper keeps the designed `string` parameter type.)

## Issues Found

None new. Pre-existing (noted, not blocking): stale `lib/payments/stripe` imports in `payments.test.ts` and stale Stripe gateway expectations in `checkout-integration.test.ts`; both are listed as known traps in AGENTS.md.