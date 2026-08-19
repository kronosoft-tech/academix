# Verify Report: billing-cron-fixes

- **Change**: `billing-cron-fixes`
- **Verified at**: HEAD `0237455` (5 commits ahead of `origin/main`, not pushed)
- **Commits**: `6773aa8` refactor getPlanById · `b5b3452` cron failure alerts · `8c9c311` repair billing crons · `492a8ef` real-column fixtures · `0237455` apply-progress
- **Mode**: Standard verification (`strict_tdd: false` per `sdd-init/academix` engram #344 — no strict-tdd module loaded)
- **Artifacts read**: proposal.md, spec.md, design.md, tasks.md, apply-progress.md (all present)
- **Verdict**: **PASS** — implementation satisfies all 5 spec requirements (12/12 scenarios covered by passing tests). The 7 failing tests and 7 typecheck errors in the full suite are pre-existing baseline failures, proven untouched by this change.

## Summary

The three billing crons were repaired against the real schema (`s.plan_id`, `s.grace_expires_at`), renewal prices now come from `PLANS` via the new `getPlanById()` single source (89900/149900/259900), grace-expired subscriptions now end `cancelled` instead of `expired`, and all three handlers gained top-level failure alerting (`sendCronAlert` → log + `SUPPORT_EMAIL` + 500) with auth checks preserved outside the catch. All 12 task checkboxes are checked, and every new/updated test passes at runtime.

## Verified Requirements

Spec requirements: **5** (2 MODIFIED, 3 ADDED). Spec scenarios: **12**.

| # | Requirement | Status | Implementation evidence | Test evidence (runtime) |
|---|---|---|---|---|
| R1 | Recurring Charge via Cron (MODIFIED) | ✅ COMPLIANT | `web/src/pages/api/cron/charge-wompi.ts:43-51` — SELECT `s.plan_id`, filters `provider='wompi' AND status='active' AND current_period_end <= ? AND payment_source_token IS NOT NULL`; price `getPlanById(sub.plan_id).priceCOP * 100`; reference `renewal-<id>-<Date.now()>`; `getPlanPriceCOP` deleted; per-sub catch → `startGracePeriod`; extension only in webhook `activateSubscription` (`web/src/pages/api/webhooks/wompi.ts:49,79` → `lifecycle.ts:99-111`, +30d) | `cron-billing.test.ts` R1: SQL asserts `s.plan_id` + all 4 filters; amount `149900*100` for 'pro'; body `{charged:1, failed:0, total:1}` → PASSED. Webhook extension: `webhook-flows.test.ts` asserts `mockActivate` on APPROVED → PASSED |
| R2 | Cron Failure Alerting (ADDED) | ✅ COMPLIANT | All 3 handlers wrap body in try/catch → `sendCronAlert('<handler>', err)` + 500 (`charge-wompi.ts:105-111`, `send-reminders.ts:115-121`, `expire-subscriptions.ts:56-62`); auth outside catch: missing secret → 500 no alert, bad auth → 401 no alert (all 3 handlers). `cron-alert.ts` logs `[cron:<handler>] FAILED:` + `sendEmail({to: SUPPORT_EMAIL || 'support@academix.app'})` in own try/catch (never masks 500) | `cron-billing.test.ts` R5: db reject → 500 + email spy to `support@academix.app` with subject containing `charge-wompi`; 401 → no email; missing `CRON_SECRET` → 500 no email → PASSED |
| R3 | Grace Period (MODIFIED) | ✅ COMPLIANT | `startGracePeriod` sets `status='grace'`, `grace_expires_at`=now+7d (`lifecycle.ts:113-122`); `getExpiredGraceSubscriptions` filters `status='grace' AND grace_expires_at < ?` (`lifecycle.ts:138-153`, idempotent); `expire-subscriptions.ts:36-38` routes grace → `cancelSubscription` (`status='cancelled'`, `lifecycle.ts:131-136`), trial loop unchanged → `expireSubscription` | `cron-billing.test.ts` R4: grace subs → `cancelSubscription`, trial → `expireSubscription` → PASSED. `subscription-lifecycle.test.ts` asserts `status='grace'` + `grace_expires_at = ?` (+7d) → PASSED |
| R4 | Grace Warning Reminder Cron (ADDED) | ✅ COMPLIANT | `send-reminders.ts:77-84` SELECT/WHERE `s.grace_expires_at` with `status='grace' AND grace_expires_at > ?`; computes `days_left`; per-sub `sendGraceWarning`; trial block (`trial_end`) unchanged; response `{trialReminders, graceWarnings}` | `cron-billing.test.ts` R3: SQL asserts `s.grace_expires_at` + `s.status = 'grace'` + `s.grace_expires_at > ?`; `sendGraceWarning` called with `days_left=3`, body `graceWarnings:1`; no grace subs → `graceWarnings:0` + HTTP 200 → PASSED |
| R5 | Schema-Accurate Test Fixtures (ADDED) | ✅ COMPLIANT | Fixtures in `payments.test.ts:196-227` → `plan_id`/`grace_expires_at`; `subscription-lifecycle.test.ts` 4 sites (:76-83, :155-163, :201-209, :221-229) use real columns only. Repo-wide grep: no fixture mocks `plan`/`grace_start`/`grace_end` (only doc comments in `webhook-flows.test.ts:283-284`, non-fixture) | `cron-billing.test.ts` asserts `s.plan_id` / `s.grace_expires_at` in executed SQL (drift fails CI) → PASSED. Lifecycle + cron suites green |

## Task Completion

- **12/12 tasks checked** (`tasks.md`), including Phase 4 verification tasks (4.1 build, 4.2 full test run with pre-existing failures noted).
- Cross-checked against `apply-progress.md` — per-task evidence matches source inspection (10 files changed, matching design's file list).

## Test Results

| Command (in `web/`) | Result | Notes |
|---|---|---|
| `bun run test -- src/test/payments/cron-billing.test.ts src/test/payments/subscription-lifecycle.test.ts src/test/payments.test.ts` | **32 passed / 5 failed** (exit 1) | `cron-billing.test.ts` 12/12 PASSED (new); `subscription-lifecycle.test.ts` PASSED; the 5 failures are all `Cannot find module '../lib/payments/stripe'` in `payments.test.ts` Stripe describe blocks — **pre-existing** |
| `bun run test` (full suite) | **192 passed / 7 failed** (199 total, exit 1) | 7 failures: 5× `payments.test.ts` stale Stripe import + 2× `checkout-integration.test.ts` stale Stripe gateway routing expectations — **pre-existing**, identical to apply-progress baseline. Proven untouched: `git log 6773aa8~1..HEAD` shows no change touched `checkout-integration.test.ts` / `gateway.ts` / `stripe.ts`, and the `payments.test.ts` diff contains 0 Stripe-import lines |
| `bun run build` (Astro) | **Complete** (exit 0) | Only warning: local Node 26 vs Vercel Node 24 runtime (environment note, non-blocking) |
| `bunx tsc --noEmit` | **7 errors, all pre-existing** (exit 2) | 5× TS2307 stale `lib/payments/stripe` import, 1× TS2367 stale comparison (`checkout-integration.test.ts:61`), 1× TS2352 (`lib/db.ts:25`) — zero new type errors from this change |

## Design Coherence

| Design decision | Implementation | Conforms |
|---|---|---|
| `getPlanById()` single source (cron + both checkouts) | `plans.ts:70-72`; `checkout/wompi.ts:28`, `checkout/mercadopago.ts:30` | ✅ |
| Grace expiry → `cancelSubscription` (cron stays thin) | `expire-subscriptions.ts:36-38` | ✅ |
| `sendCronAlert` wrapping `sendEmail`, own try/catch, never masks 500 | `cron-alert.ts:17-34` | ✅ |
| Unknown plan → fail loud → grace + log (no silent default) | `charge-wompi.ts:67-76` | ✅ |
| Auth checks outside the catch (missing secret 500 / bad auth 401, no alert) | All 3 handlers | ✅ |
| `DueSubscription {id, user_id, plan_id, payment_source_token, email}` | `charge-wompi.ts:10-16` | ✅ |
| File change list (10 files) | All present as designed | ✅ |

No substantive deviations. Minor: `getPlanById` call sites narrow `planId` before calling (`planId ? getPlanById(planId) : undefined`) — helper keeps the designed `string` parameter type; documented in apply-progress.

## Issues

### CRITICAL
None.

### WARNING
None attributable to this change.

### SUGGESTION
1. `web/src/test/payments/webhook-flows.test.ts:283-284` — documentation comments still mention non-existent `grace_start`/`grace_end` column names. Comments only (no fixture impact), but drift contradicts the "schema-accurate" requirement intent; worth a one-line cleanup.
2. Pre-existing baseline debt (out of scope, known traps per AGENTS.md): 5 Stripe-module imports in `payments.test.ts`, 2 Stripe gateway expectations in `checkout-integration.test.ts`, `lib/db.ts:25` cast. Candidates for a separate cleanup change.
3. W9 double-charge risk (webhook failure after charge) remains deferred to `payment-integrity` — per-sub failures go to grace and the period filter skips confirmed subs, per design.

## Risks / Follow-ups

- **Deferred (by design)**: charge idempotency (W9), webhook amount/plan verification (W4), unique constraints (W10/W20) → `payment-integrity`.
- **Legacy unknown `plan_id` rows**: enter grace with a log (fail loud) — no silent default price; monitor alert noise for legacy rows.
- **Node 26 local vs Node 24 Vercel runtime**: build warning only; confirm the Vercel project pins Node 24.
- **Archive readiness**: no review lifecycle run here; tasks complete, verification PASS → proceed to `/sdd-archive`.

## Conclusion

All 12 tasks complete; all 5 spec requirements (12 scenarios) satisfied with passing runtime tests; design decisions followed; zero new test or type failures. The full-suite failures are pre-existing baseline (documented in AGENTS.md as known traps) and were not introduced by this change. **Verdict: PASS** — ready to archive.