# Design: Billing Cron Fixes

## Overview / Goals

Three daily billing crons (web/vercel.json 06:00/07:00/08:00) silently lose revenue: charge-wompi selects non-existent s.plan (500 daily → renewals never charged) and charges stale prices (~50% low); send-reminders selects non-existent s.grace_end; failures invisible. Fix queries to real columns, source prices from plans.ts, flip grace expiry to cancelled, add failure alerting, fix test fixtures. Forward-only; no migration.

## Architecture Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Price lookup | getPlanById() in plans.ts, cron + both checkouts | Checkouts inline PLANS.find (checkout/wompi.ts:28,64; mercadopago.ts:30,53); one function kills the duplication behind W2 |
| Grace expiry | expire-subscriptions calls cancelSubscription (lifecycle.ts:131) | Exists, tested; cron stays thin |
| Alert transport | new sendCronAlert wrapping sendEmail (lib/email.ts:26) | sendEmail never throws (returns false w/o GMAIL creds) → cannot mask the 500; dedupes log+email |
| Unknown plan | fail loud → grace + log | R2: no silent default price |

## Approach (R1–R6)

**R1 — charge-wompi** (charge-wompi.ts): SELECT :40 s.plan → s.plan_id; keep filters :43-46 (provider='wompi' AND status='active' AND current_period_end <= ? AND payment_source_token IS NOT NULL). Interface :8-15 → plan_id only (no price_cop). Loop (:62-81): getPlanById(sub.plan_id); !plan → log + startGracePeriod + failed++ (no transaction); else createTransaction(plan.priceCOP*100, 'COP', 'renewal-'+sub.id+'-'+Date.now(), token, email) → charged++; catch → startGracePeriod + failed++. Drop getPlanPriceCOP (:92-98). Extension only in webhook activateSubscription (lifecycle.ts:99).

**R2 — prices** (plans.ts): add getPlanById(planId): Plan | undefined = PLANS.find; refactor checkout/wompi.ts:28 + mercadopago.ts:30 to it. Real prices 89900/149900/259900 (plans.ts:16,32,49).

**R3 — send-reminders** (send-reminders.ts): :75 SELECT and :79 WHERE s.grace_end → s.grace_expires_at. Trial block :41-71 unchanged.

**R4 — grace expiry → cancelled** (expire-subscriptions.ts:31-33): grace loop → cancelSubscription; trial loop :36-39 stays expireSubscription. cancelled = status TEXT value (001:5, no CHECK), terminal → login blocked. getExpiredGraceSubscriptions (lifecycle.ts:148) filters status='grace' AND grace_expires_at < ? → already-cancelled/expired/activated rows never re-enter (idempotent); activation mid-grace clears grace_expires_at (:106), exiting the set.

**R5 — failure alert** (all 3 handlers): auth checks outside the catch (missing CRON_SECRET → 500 no alert; bad auth → 401 no alert). Body in try/catch → sendCronAlert('<handler>', err) + 500. New web/src/lib/payments/cron-alert.ts logs '[cron:<handler>] FAILED:' then sendEmail({ to: SUPPORT_EMAIL || 'support@academix.app' /* contact.ts:5 */ }) in its own try/catch — never masks the 500. Per-sub failures stay local (send-reminders:68-70,100-102; charge catch → grace); throwing startGracePeriod propagates to top-level alert.

**R6 — tests**: fixtures: payments.test.ts:199-211 (plan/grace_start/grace_end → plan_id + grace_expires_at); subscription-lifecycle.test.ts:76-83,155-163,201-209,221-229 (drop stale columns). New cron-billing.test.ts (mock db + lifecycle + email): charge SQL asserts s.plan_id + filters; amount 149900×100 for 'pro'; unknown plan/throw → grace. send-reminders asserts s.grace_expires_at, graceWarnings=1. expire-subscriptions: grace → cancelSubscription, trial → expireSubscription. R5: db reject → 500 + email spy; 401 → none. R4: grace expiry → cancelled.

## Data Flow

06:00 grace_expires_at <= now → cancelSubscription · 07:00 grace_expires_at > now → sendGraceWarning · 08:00 due subs → getPlanById → createTransaction (ok → webhook extends; fail → startGracePeriod) · top-level throw → sendCronAlert → HTTP 500.

## File Changes

| File | Action | Description |
|------|--------|-------------|
| web/src/data/plans.ts | Modify | Add getPlanById |
| web/src/pages/api/cron/charge-wompi.ts | Modify | s.plan_id + prices + alert |
| web/src/pages/api/cron/send-reminders.ts | Modify | grace_expires_at + alert |
| web/src/pages/api/cron/expire-subscriptions.ts | Modify | grace → cancel + alert |
| web/src/lib/payments/cron-alert.ts | Create | sendCronAlert |
| web/src/pages/api/checkout/{wompi,mercadopago}.ts | Modify | use getPlanById |
| web/src/test/payments.test.ts | Modify | real-column fixture |
| web/src/test/payments/subscription-lifecycle.test.ts | Modify | real-column fixtures |
| web/src/test/payments/cron-billing.test.ts | Create | R1–R6 cron tests |

## Interfaces / Contracts

getPlanById(planId): Plan | undefined · sendCronAlert(handler, err): Promise<void> · DueSubscription { id; user_id; plan_id; payment_source_token; email }.

## Testing Strategy

- Unit: getPlanById → PLANS entry | undefined.
- Integration (cron-billing.test.ts): real-column SQL, price = PLANS, grace → cancelled, 500 + email spy, 401 none. E2E: none (cron endpoints).

## Threat Matrix

N/A — no routing, shell, subprocess, VCS/PR automation, executable-file classification, or process-integration boundary. Crons are pre-existing authenticated HTTP endpoints; auth semantics preserved, tested under R5 (401 no alert; 500 no alert on missing secret).

## Migration / Rollout

No migration — plan_id (001:4) and grace_expires_at (002:10) exist; migrate.ts untouched. Order: plans.ts helper → checkout refactor → cron-alert → handlers → tests (one web PR). Rollback: git revert; no DDL.

## Risks & Mitigations

- Double charge if webhook fails (W9) — Med; deferred to payment-integrity; period filter skips confirmed subs; unique reference per run.
- Wompi outage — Med; per-sub catch → grace; alert; retry next day.
- Alert email fails — Low; sendCronAlert catches; 500 preserved.
- Plans drift — Low; single getPlanById for cron + checkouts.
- Legacy unknown plan_id — Low; fail loud → grace + log.

## Out of Scope

Backfill; DB plans table; grandfathering; charge idempotency (W9 → payment-integrity); webhook verification (W4); unique constraints (W10/W20); Stripe; desktop.

## Open Questions

None blocking.