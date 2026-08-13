# Archive Report: mp-webhook-500-diagnosis

**Archive date**: 2026-08-13
**Mode**: hybrid (OpenSpec file + Engram) — BOTH backends written
**Archived to**: `openspec/changes/archive/2026-08-13-mp-webhook-500-diagnosis/`
**Engram**: `sdd/mp-webhook-500-diagnosis/archive-report`

## Change Summary

`POST /api/webhooks/mercadopago` returned HTTP 500 for all payloads. Root cause: `MP_WEBHOOK_SECRET` was unset in Vercel production — the handler (`webhooks/mercadopago.ts:40`) returned 500 "Mercado Pago webhook secret not configured" as the first guard, before signature verification, before `getPayment`, before any DB call. Fix: (1) operator adds `MP_WEBHOOK_SECRET` via Vercel; (2) code hardens `verifyWebhookSignature` to never throw (try/catch → `false` → 401) and guards the call site (try/catch → 401). The missing-secret→500 gate is preserved as an intentional deployment-time signal.

## Gates Passed

- **Task completion gate**: `tasks.md` shows **6/6** tasks `[x]` (Phase 1: 3/3, Phase 2: 1/1, Phase 3: 1/1, Phase 4: 1/1), zero unchecked implementation tasks. **No stale unchecked implementation tasks in the archived audit trail.**
- **Verification gate**: 5/5 spec requirements (R1–R5) compliant; all 4 spec scenarios covered by passing runtime tests (7/7 passed, exit 0); **0 CRITICAL** findings. `bunx tsc --noEmit` shows 6 pre-existing errors in unmodified files; zero new errors introduced. Verdict: **PASS WITH WARNINGS**.
- **Review/archive authorization**: No native `reviews/` bundle exists for this change (consistent with prior changes). The verify-report serves as verification evidence with "PASS WITH WARNINGS" verdict — 0 CRITICAL findings, two WARNINGs and one SUGGESTION regarding scope deviations (documented below).

## Intentional Archive with Warnings

### WARNING W1: Scope deviation — `dashboard/index.astro` modified outside planned scope

`web/src/pages/dashboard/index.astro` was changed in the working tree (63 lines: removed `activateApprovedPayment` import, refactored MP payment verification to call `/api/payments/verify-mercadopago` instead of direct MP API fetch, replaced Spanish UI error messages with English). This file is NOT listed in:
- design.md File Changes table (only 4 files: mercadopago.ts, webhooks/mercadopago.ts, .env.example, vercel-rollout.md)
- tasks.md task descriptions
- apply-progress.md Files Changed table

**Impact**: Unplanned scope expansion. The English-string replacement in dashboard UI is user-facing copy unrelated to webhook hardening. This change was committed alongside the `mercado-pago-persistence` change.

### WARNING W2: Scope deviation — `expectedUserId` ownership guard added (not in design/tasks)

`mercadopago.ts` was modified beyond the planned try/catch wrapping (D2). Two additional changes:
- `expectedUserId?: string` added to `ApprovedPaymentInput` interface (L244)
- Ownership guard added to `activateApprovedPayment` (L263-268): `if (expectedUserId && !externalReference.startsWith(expectedUserId)) return;`

This is NOT in design.md (D4 explicitly says "Minimal-diff: add guards around existing logic, don't rewrite") or tasks.md. The `verify-mercadopago.ts` endpoint passes `expectedUserId: payload.sub` (L84) to exercise this guard.

### SUGGESTION S1: Untested ownership guard

The `expectedUserId` ownership guard in `activateApprovedPayment` (L263-268, mercadopago.ts) is not covered by any test. The `mercadopago.test.ts` file mocks `activateApprovedPayment` (lines 28-29, 89), so the guard's internal logic is never exercised at runtime. Should have a dedicated unit test or be moved to a separate change.

### Scope deviation documentation

Per archive instructions, the following cross-change edits are documented as known issues and belong to the `mercado-pago-persistence` change, NOT this one:
- `web/src/lib/payments/mercadopago.ts` — `expectedUserId` ownership guard (D2 area) was added by the `mercado-pago-persistence` wiring
- `web/src/pages/dashboard/index.astro` — refactored to call `/api/payments/verify-mercadopago` endpoint, English UI strings (outside this change's plan)

These cross-change edits are committed with the other change, not here. The archive does not commit any git changes (working tree left as-is).

## Specs Synced (main specs = source of truth)

The delta spec (`specs/mp-webhook-robustness/spec.md`) was merged into `openspec/specs/mercadopago-payments/spec.md` (the main spec already mandated the core behavior per the proposal):

| Action | Details |
|--------|---------|
| MERGED (no-throw guarantee) | Added **Robustness guarantee (no-throw)** note to the existing "Webhook Signature Verification and Payment Processing" requirement |
| ADDED (3 scenarios) | Added "Missing x-signature header → 401 (not 500)" scenario to the webhook requirement |
| ADDED (1 scenario) | Added "Malformed non-hex v1 → 401 (not 500)" scenario to the webhook requirement |
| ADDED (1 scenario) | Added "Missing secret returns 500 (deployment-time gate, preserved)" scenario to the webhook requirement |
| ADDED (2 requirements) | "Webhook Secret Documented in .env.example" requirement (R: Should) with 1 scenario |
| ADDED (2 requirements) | "Vercel Rollout Checklist Includes MP_WEBHOOK_SECRET" requirement (R: Must) with 1 scenario |

All requirements NOT mentioned in the delta were preserved unchanged.

## Implementation (git diff summary)

| File | Action | What Was Done |
|------|--------|---------------|
| `web/src/lib/payments/mercadopago.ts` | Modified | Wrapped `verifyWebhookSignature` body (L214-231) in try/catch → console.error + return false (D2) |
| `web/src/pages/api/webhooks/mercadopago.ts` | Modified | Guarded `verifyWebhookSignature` call site (L54) with try/catch → 401 (D3); missing-secret→500 gate untouched (D1) |
| `web/src/.env.example` | Modified | Added `MP_WEBHOOK_SECRET=` after `MP_API_URL` (L17) with note "distinct from MP_ACCESS_TOKEN" |
| `web/docs/vercel-rollout.md` | Modified | Added `bunx vercel env add MP_WEBHOOK_SECRET production` to Step 3 checklist + post-fix smoke check note |
| `web/src/test/payments/mercadopago.test.ts` | Created | 7 tests covering valid sig, empty/missing x-signature, malformed non-hex v1, missing secret → all pass |
| `web/src/pages/dashboard/index.astro` | Modified (cross-change, W1) | Refactored to call `/api/payments/verify-mercadopago`; English UI strings — belongs to `mercado-pago-persistence` change |
| `web/src/lib/payments/mercadopago.ts` (expectedUserId) | Modified (cross-change, W2) | Added `expectedUserId` ownership guard — belongs to `mercario-pago-persistence` change |

**Changed lines**: 104 insertions, 60 deletions (5 tracked files).
**New untracked file**: `web/src/test/payments/mercadopago.test.ts`

## Verification Outcome

| Metric | Result |
|--------|--------|
| Tasks | 6/6 complete |
| Spec requirements | 5/5 compliant (R1–R5) |
| Spec scenarios | 4/4 spec scenarios covered by passing runtime tests |
| CRITICAL findings | 0 |
| Focused tests | `bun run test -- src/test/payments/mercadopago.test.ts` → 7 passed, 0 failed (exit 0) |
| Typecheck | 6 pre-existing errors (all in unmodified files: db.ts, payments.test.ts, checkout-integration.test.ts); zero new errors |

## Archive Contents

- `proposal.md` ✅
- `design.md` ✅
- `specs/mp-webhook-robustness/spec.md` ✅ (delta spec preserved as audit trail)
- `tasks.md` ✅ (6/6 tasks complete, all `[x]`)
- `apply-progress.md` ✅
- `verify-report.md` ✅ (PASS WITH WARNINGS)
- `exploration.md` ✅
- `archive-report.md` ✅ (this file)

## Rollback Notes

- Revert code/doc patches: `mercadopago.ts` (remove try/catch in `verifyWebhookSignature`), `webhooks/mercadopago.ts` (remove try/catch at call site), `.env.example` (remove `MP_WEBHOOK_SECRET=` line), `vercel-rollout.md` (remove env-add + smoke check). Delete `web/src/test/payments/mercadopago.test.ts`.
- Env var removal is via Vercel dashboard only — the handler's missing-secret gate already returns a clear 500.
- `dashboard/index.astro` and `expectedUserId` ownership guard roll back with the `mercario-pago-persistence` change, NOT this one.

## Cross-Change Context

- This change (`mp-webhook-500-diagnosis`) and `mercado-pago-persistence` share the same working tree. The latter adds the `expectedUserId` ownership guard and the dashboard refactor to call `/api/payments/verify-mercadopago`. These are intentional scope expansions that belong to `mercado-pago-persistence`, not this change.
- `mercado-pago-persistence` should be archived separately once its tasks are complete and verified.

## Engram Traceability (observation IDs — history preserved, not deleted)

| Artifact | Observation ID |
|----------|----------------|
| Exploration | #1126 |
| Proposal | #1127 |
| Spec (delta) | #1128 |
| Design | #1129 |
| Tasks | #1130 |
| Apply progress | #1132 |
| Verify report | #1134 |
| Archive report (this) | `sdd/mp-webhook-500-diagnosis/archive-report` |

## Open Follow-ups (documented, NOT blocking — schedule as new changes)

1. **FU1 — Scope deviation resolution**: The `expectedUserId` ownership guard (W2/S1) needs either a dedicated unit test or migration to a separate concern. Currently untested because `mercadopago.test.ts` mocks `activateApprovedPayment`.
2. **FU2 — Dashboard Englishization**: The Spanish→English UI string replacement in `dashboard/index.astro` (W1) was done as part of this change's working tree but belongs to `mercado-pago-persistence`. Confirm whether it should be reverted or kept.
3. **FU3 — Stale test debt**: `checkout-integration.test.ts` references `createPreapproval` (nonexistent in real module — function is `createPreference`). Deferred per proposal Q3. Pre-existing, not introduced here.

## SDD Cycle Complete

Change `mp-webhook-500-diagnosis` planned, implemented, verified (5/5 requirements, 4/4 scenarios, 0 CRITICAL, 7/7 tests), and archived with warnings. Main specs updated with no-throw guarantee, documentation requirements, and post-fix scenarios. Change folder moved to `openspec/changes/archive/2026-08-13-mp-webhook-500-diagnosis/`. Archive report persisted to both OpenSpec and Engram.

**Next recommended**: `sdd-archive mercado-pago-persistence` (separate change sharing this working tree).
