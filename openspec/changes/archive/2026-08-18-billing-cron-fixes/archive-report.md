# Archive Report: Billing Cron Fixes

**Change**: `billing-cron-fixes`
**Date**: 2026-08-18
**Status**: ✅ Complete — planned, implemented, verified (PASS), archived
**Archived to**: `openspec/changes/archive/2026-08-18-billing-cron-fixes/`

## Summary

Repaired the three daily billing crons (`web/vercel.json` 06:00/07:00/08:00) that were silently losing revenue: `charge-wompi.ts` selected the non-existent `s.plan` column (500 every run → renewals never charged) and charged stale hardcoded prices (~50% below current); `send-reminders.ts` selected the non-existent `s.grace_end` (grace warnings never sent). This change switched queries to real schema columns (`s.plan_id`, `s.grace_expires_at`), sourced renewal prices from `PLANS` via the new `getPlanById()` single source (basico 89900 / pro 149900 / premium 259900, same as checkouts), routed grace-expired subscriptions to `cancelled` (via `cancelSubscription`) instead of `expired`, added top-level failure alerting (`sendCronAlert` → prominent log + email to `SUPPORT_EMAIL` + HTTP 500) to all three handlers, and aligned test fixtures with the real schema so column drift fails CI.

## Review Gate

No review lifecycle was run for this change (the repo does not run the native review gate for this cycle — see verify-report.md "Archive readiness" note). The orchestrator explicitly instructed not to call review lifecycle commands. The gate approval is evidenced by the independent verify-report.md **PASS** verdict (all 5 spec requirements / 12 scenarios verified against runtime test evidence, zero new test or type failures).

## Task Completion

- **12/12 tasks checked** in `tasks.md` (Phases 1–4: foundation, core implementation, testing, verification).
- Cross-checked against `apply-progress.md` — per-task evidence matches source inspection; all 10 files changed as designed.
- Verified at HEAD `0237455` (5 commits ahead of `origin/main`, not pushed): `6773aa8` getPlanById · `b5b3452` cron failure alerts · `8c9c311` repair billing crons · `492a8ef` real-column fixtures · `0237455` apply-progress.

## Spec Sync

Delta spec (`spec.md`, kept at the change root per this repo's convention) merged into the main specs. Scenario headings normalized from the delta's `#####` to the main specs' `####` level; `(Previously: ...)` notes preserved per the existing main-spec style.

| Domain | Main spec | Action | Details |
|--------|-----------|--------|---------|
| `wompi-payments` | `openspec/specs/wompi-payments/spec.md` | Updated | 1 MODIFIED (`Recurring Charge via Cron` — real `s.plan_id` column, prices from `PLANS`/`getPlanById`, per-sub failure → `startGracePeriod`, extension only via webhook; 4 scenarios replaced), 1 ADDED (`Cron Failure Alerting` — try/catch + log + `SUPPORT_EMAIL` + 500, auth stays outside catch; 2 scenarios) |
| `subscription-lifecycle` | `openspec/specs/subscription-lifecycle/spec.md` | Updated | 1 MODIFIED (`Grace Period` — warnings via `send-reminders` on `grace_expires_at`, grace expiry → `cancelled` not `expired`; 2 scenarios replaced), 2 ADDED (`Grace Warning Reminder Cron` — real `s.grace_expires_at`, `days_left`, `sendGraceWarning`, trial reminders unchanged; 2 scenarios; and `Schema-Accurate Test Fixtures` — real-column-only mocks, SQL assertions; 2 scenarios) |

No main spec content was deleted — only the two delta-touched requirements were replaced in full (per the MODIFIED convention) and the three ADDED requirements were appended. All other requirements in both main specs are untouched.

## Archive Contents

- proposal.md ✅
- spec.md (delta) ✅
- design.md ✅
- tasks.md ✅ (12/12 tasks complete, no unchecked implementation tasks)
- apply-progress.md ✅
- verify-report.md ✅ (PASS)
- archive-report.md ✅ (this file)

## Verification Evidence (from verify-report.md)

- `bun run test -- src/test/payments/cron-billing.test.ts src/test/payments/subscription-lifecycle.test.ts src/test/payments.test.ts` → 32 passed / 5 failed (5 failures pre-existing stale Stripe import in `payments.test.ts`); new `cron-billing.test.ts` 12/12 PASSED.
- `bun run build` (Astro) → Complete (exit 0); `bunx tsc --noEmit` → 7 pre-existing errors, 0 new.
- No CRITICAL issues; no WARNING attributable to this change.

## Rollout / Rollback

No DB migration (columns `plan_id` at `001_subscriptions.sql:4` and `grace_expires_at` at `002_multi_gateway.sql:10` already exist). Rollback: git revert of the cron handler changes (`charge-wompi.ts`, `send-reminders.ts`, `expire-subscriptions.ts`), `plans.ts`/`cron-alert.ts`, checkout refactors, and test files — no DDL. Do not push (commits intentionally left unpushed at the orchestrator's instruction).

## Follow-ups

- Deferred to `payment-integrity` (by design): charge idempotency (W9), webhook amount/plan verification (W4), unique constraints (W10/W20).
- Legacy unknown `plan_id` rows enter grace with a log (fail loud, no silent default) — monitor alert noise.
- Pre-existing baseline debt (out of scope): stale `lib/payments/stripe` imports in `payments.test.ts`, stale Stripe gateway expectations in `checkout-integration.test.ts`, `lib/db.ts:25` cast — candidates for a separate cleanup change.
- Minor: documentation comments in `webhook-flows.test.ts:283-284` still mention non-existent `grace_start`/`grace_end` columns (comments only, no fixture impact) — worth a one-line cleanup.
- Confirm the Vercel project pins Node 24 (local Node 26 vs Vercel Node 24 runtime warning is environment-only).
