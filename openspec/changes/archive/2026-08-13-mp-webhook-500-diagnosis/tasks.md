# Tasks: MP Webhook 500 Fix — Signature Hardening

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~110 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | auto-forecast |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

> Operator prerequisite (proposal Phase 1): `MP_WEBHOOK_SECRET` must be added to Vercel production via `bunx vercel env add MP_WEBHOOK_SECRET production` before the code changes are meaningful. This is a manual gate, not a task.

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| 1 | Harden `verifyWebhookSignature` + guard call site + env docs + tests + typecheck | PR 1 | `bun run test -- web/src/test/payments/mercadopago.test.ts` | `bunx tsc --noEmit` (from `web/`) | Revert `mercadopago.ts`, `webhooks/mercadopago.ts`, `.env.example`, `vercel-rollout.md`; delete test file |

## Phase 1: Foundation / Infrastructure

- [x] 1.1 Harden `verifyWebhookSignature` in `web/src/lib/payments/mercadopago.ts` (L214-231): wrap body in `try/catch` → `console.error` + `return false`; missing `ts`/`v1` → `false`; non-hex `v1` → `false`. (D2, R5) No deps.
- [x] 1.2 Add `MP_WEBHOOK_SECRET=` line after `MP_API_URL` (L16) in `web/src/.env.example`; note "distinct from access token". (R4)
- [x] 1.3 Add `bunx vercel env add MP_WEBHOOK_SECRET production` to checklist Step 3 in `web/docs/vercel-rollout.md` + post-fix 401 expectation note. (R5)

## Phase 2: Core Implementation

- [x] 2.1 Guard call site at `web/src/pages/api/webhooks/mercadopago.ts` L54: wrap `verifyWebhookSignature(...)` in `try/catch` → 401 "Invalid signature"; preserve L39-48 missing-secret→500 gate unchanged (D1, D3, R3). Deps: 1.1.

## Phase 3: Testing

- [x] 3.1 Create `web/src/test/payments/mercadopago.test.ts` (vitest, node env): 4 scenarios — (a) valid HMAC sig → `verifyWebhookSignature` returns `true`; (b) empty/missing `x-signature` → returns `false` / handler 401 (not 500); (c) malformed non-hex `v1` → returns `false` / handler 401 (not 500); (d) missing secret → handler 500 "not configured" (gate preserved). Deps: 1.1, 2.1.

## Phase 4: Verification

- [x] 4.1 Runtime verification from `web/`: `bunx tsc --noEmit` + `bun run test -- src/test/payments/mercadopago.test.ts`. Document pre-existing failures separately (e.g. stale `checkout-integration.test.ts` referencing nonexistent `createPreapproval`). Deps: 1.1–3.1.
