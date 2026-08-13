## SDD Apply-progress: MP Webhook 500 Fix (mp-webhook-500-diagnosis)

**Status**: All tasks T1-T5 complete. Standard mode (strict_tdd=false). Hybrid artifact store.

### Completed Tasks
- [x] 1.1 Harden `verifyWebhookSignature` in `web/src/lib/payments/mercadopago.ts` — wrapped body in try/catch → console.error + return false
- [x] 1.2 Add `MP_WEBHOOK_SECRET=` to `web/src/.env.example` after `MP_API_URL` (line 17)
- [x] 1.3 Add `bunx vercel env add MP_WEBHOOK_SECRET production` to checklist Step 3 in `web/docs/vercel-rollout.md` + post-fix 401 expectation note
- [x] 2.1 Guard call site at L54 in `web/src/pages/api/webhooks/mercadopago.ts` with try/catch → 401; missing-secret→500 gate (L39-48) preserved unchanged
- [x] 3.1 Create `web/src/test/payments/mercadopago.test.ts` — 4 scenarios (valid sig → true; empty/missing → false + handler 401; malformed non-hex → false + handler 401; missing secret → handler 500)
- [x] 4.1 Runtime verification: tsc + tests

### Files Changed
| File | Action | What Was Done |
|------|--------|---------------|
| `web/src/lib/payments/mercadopago.ts` | Modified | Wrapped `verifyWebhookSignature` body (L214-231) in try/catch → console.error + return false (D2) |
| `web/src/pages/api/webhooks/mercadopago.ts` | Modified | Guarded `verifyWebhookSignature` call site (L54) with try/catch → 401 (D3); missing-secret→500 gate untouched (D1) |
| `web/src/.env.example` | Modified | Added `MP_WEBHOOK_SECRET=` after `MP_API_URL` (L17) with note "distinct from access token" |
| `web/docs/vercel-rollout.md` | Modified | Added `bunx vercel env add MP_WEBHOOK_SECRET production` to Step 3 checklist + non-interactive form + post-fix smoke check note |
| `web/src/test/payments/mercadopago.test.ts` | Created | 7 tests: 3 unit (verifyWebhookSignature direct) + 4 handler integration (401/401/500/200) |

### Work Unit Evidence
| Evidence | Required value | Actual |
|---|---|---|
| Focused test command and exact result | Smallest command proving this unit; command, exit/result, and relevant counts | `bun run test -- src/test/payments/mercadopago.test.ts` → exit 0, 7 passed (7 total) in 92ms |
| Runtime harness command/scenario and exact result | Real integration/runtime path; explicit N/A only when no runtime boundary exists | `bunx tsc --noEmit` (from web/) → 6 errors, ALL pre-existing (db.ts:25, payments.test.ts x5 missing stripe module, checkout-integration.test.ts:61 type comparison). No errors in any modified file. |
| Rollback boundary | Exact files/behavior that can be reverted without removing unrelated work | Revert `mercadopago.ts` L214-236, `webhooks/mercadopago.ts` L54-69, `.env.example` L17, `vercel-rollout.md` Steps 3+5, delete `mercadopago.test.ts`. Env var removal is via Vercel dashboard only. |

### Spec Coverage
- R1 (env.example has MP_WEBHOOK_SECRET after MP_API_URL): satisfied — L17
- R2/R5 (verifyWebhookSignature never throws, missing/malformed → false): satisfied — try/catch + existing logic
- R3/R4 (handler 401 for invalid sig, 500 for missing secret): satisfied — try/catch + 401 block; L39-48 untouched
- R5 (vercel-rollout.md lists env-add + post-fix 401 note): satisfied — Step 3 + smoke check item 5

### Deviations from Design
None — implementation matches design.md exactly.

### Issues Found
- **Pre-existing tsc errors (NOT introduced by this change):**
  - `src/lib/db.ts:25` — `Conversion of type 'Client' to type 'Record<string | symbol, unknown>'` (type mismatch in db proxy)
  - `src/test/payments.test.ts` (5 errors) — imports from nonexistent `../lib/payments/stripe` module (Stripe not implemented; per AGENTS.md)
  - `src/test/payments/checkout-integration.test.ts:61` — type comparison `"basico" | "pro" | "premium"` vs `"nonexistent"` (stale test)
- **Pre-existing uncommitted change:** `web/src/dashboard/index.astro` appears in git diff but was NOT modified by this task
- **Stale test debt:** `checkout-integration.test.ts` mocks `createPreapproval` (nonexistent in real module — function is `createPreference`). Per proposal Q3, deferred to follow-up. Not addressed in this change.

### Pre-existing Failures (separate from this change)
- `bunx tsc --noEmit` exits non-zero due to 6 pre-existing errors across 3 files — none in modified files
- `checkout-integration.test.ts` references `createPreapproval` which doesn't exist in `mercadopago.ts` — this is discovered technical debt, deferred per proposal

### Test Results
- **tsc**: 6 pre-existing errors (all in unmodified files: db.ts, payments.test.ts, checkout-integration.test.ts). Zero new errors.
- **Focused test (`src/test/payments/mercadopago.test.ts`)**: 7 passed, 0 failed (92ms)
- **All other payments tests**: Not run in this batch (out of scope for focused test command)

### Mode
Standard (strict_tdd=false). No TDD cycle evidence required. Work Unit Evidence table included per hard gate.

### Workload / PR Boundary
- Mode: single PR
- Current work unit: "Harden verifyWebhookSignature + guard call site + env docs + tests + typecheck"
- Boundary: Changes start from original `verifyWebhookSignature` (no try/catch) and end with hardened function + guarded call site + env docs + new test file
- Estimated review budget impact: ~110 lines (well under 400), Low risk

### Status
6/6 tasks complete. Ready for verify.
