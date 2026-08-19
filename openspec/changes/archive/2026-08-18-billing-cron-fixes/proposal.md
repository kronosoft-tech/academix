# Proposal: billing-cron-fixes

## Intent
Fix billing crons silently losing revenue daily: (W1) `charge-wompi` selects non-existent `s.plan` → 500 every run (08:00 UTC, `web/vercel.json`) → renewals never charged; (W2) same cron charges stale hardcoded prices (~50% below current); (W3) `send-reminders` selects non-existent `s.grace_end` → grace warnings never sent. Add minimal failure alerting + the grace-expiry → cancelled rule.

## Scope

### In Scope
- W1: `charge-wompi.ts:40,53` `s.plan` → `plan_id` (real: `web/migrations/001_subscriptions.sql:4`).
- W2: replace hardcoded `getPlanPriceCOP` (`charge-wompi.ts:92-98`) with `PLANS` from `plans.ts` (89900/149900/259900; same one checkouts use).
- W3: `send-reminders.ts:75,79,84` → `grace_expires_at` (`002_multi_gateway.sql:10`); grace-expired subs → `cancelled` (`cancelSubscription`, `lifecycle.ts:131`) instead of `expired` (`expire-subscriptions.ts:32`).
- Alert: prominent log + support email (nodemailer) on failure; return 500.
- Tests: fix stale fixtures mocking `plan`/`grace_end` (`test/payments.test.ts:207`, `subscription-lifecycle.test.ts:76-83`); add cron tests asserting real columns.

### Out of Scope
- No backfill of missed charges (assumed loss).
- No payment-integrity items (W4–W6, W9, W10, W20): webhook amount/plan verification, unique constraints, charge idempotency tokens.
- No DB `plans` table; no price grandfathering; no desktop changes.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `wompi-payments`: charge cron selects `plan_id`, prices from `plans.ts`, failure alert, per-sub failure → grace.
- `subscription-lifecycle`: grace warnings read `grace_expires_at`; grace expiry → `cancelled`.

## Approach
Forward-only; no migration (columns exist). `charge-wompi.ts`: select `plan_id`, price from `PLANS` (`plans.ts`); try/catch → log + alert `SUPPORT_EMAIL` → 500. `send-reminders.ts`: `grace_expires_at`. `expire-subscriptions.ts`: grace path → `cancelSubscription`. Keep `current_period_end <= now` filter — webhook success advances the period, skipping paid subs (W9 partial).

## Affected Areas
- `web/src/pages/api/cron/charge-wompi.ts` (Modified) — plan_id, price source, alert
- `web/src/pages/api/cron/send-reminders.ts` (Modified) — grace_expires_at
- `web/src/pages/api/cron/expire-subscriptions.ts` (Modified) — grace expiry → cancelled
- `web/src/test/payments/*` (Modified) — real-column fixtures + cron tests

## Risks
- Double-charge if webhook fails (W9) — Med: deferred; alert logs references; period filter skips paid subs
- Wompi outage — Med: per-sub catch → grace; alert; retry next day
- Legacy rows without plan match — Low: explicit handling + alert; no silent default

## Rollback Plan
Git revert of cron handler changes; no DDL or data migration.

## Dependencies
- `plans.ts`; `SUPPORT_EMAIL`; `WOMPI_PRIVATE_KEY`.

## Success Criteria
- [ ] charge-wompi returns 200 in prod
- [ ] Renewal amount = current `plans.ts` price (tested)
- [ ] send-reminders returns 200; grace warnings sent
- [ ] Grace-expired subs end `cancelled`
- [ ] Cron failure → log + email + 500

## Confirmed decisions (2026-08-18)
- **Price source**: `plans.ts` (89900/149900/259900), same as checkouts. No `plans` table exists in `web/migrations/`; none created here.
- **No backfill**: missed renewals assumed loss; forward-only.
- **Grace expiry → cancelled**: failed renewal + `grace_expires_at` passed without payment.
- **Failure alert**: in scope — log + email + 500.