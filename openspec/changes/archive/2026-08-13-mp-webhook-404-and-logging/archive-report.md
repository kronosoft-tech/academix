# Archive Report: MP Webhook 404/No-Throw Fix

**Change**: `mp-webhook-404-and-logging`
**Date**: 2026-08-13
**Status**: ✅ Complete — planned, implemented (by orchestrator — see note), verified, archived

## Summary
Fixed HTTP 500 on `POST https://academix-three-gilt.vercel.app/api/webhooks/mercadopago` for `getPayment` failures. Root cause confirmed in exploration: `getPayment` threw a plain `Error` with no `.status`, so the handler catch blanket-returned 500 for everything (including MP 404 "Payment not found"), causing MP to abandon retries and never persist the payment. Fix: `getPayment` enriches its error with `.status`/`.detail`/`.retryAfter`, and the handler classifies via `classifyMpError` — 404→200, 401/403→200+log, 429→503+Retry-After, MP 5xx→503, DB/critical→500. Missing-secret→500 gate (the operator-config issue from mp-webhook-500-diagnosis) is intentionally preserved.

## Implementation Evidence
- Tests: `bun run test -- src/test/payments/mercadopago.test.ts` → **12 passed / 0 failed** (7 from signature hardening + 5 classification matrix).
- `bunx tsc --noEmit`: 6 pre-existing errors (db.ts:25, payments.test.ts x5 stripe, checkout-integration.test.ts:61) — **0 new** in modified files.
- `astro build`: prior passing (no .astro changes here).

## Spec Sync
Delta spec merged into `openspec/specs/mercadopago-payments/spec.md` — added the "no-throw guarantee" and the classification matrix to the existing webhook-signature-and-payment-processing requirement.

## Scope Note (IMPORTANT)
This change's implementation was performed directly by the orchestrator — the sdd-apply sub-agent failed 3× to persist artifacts for the broader SDD work in this session (empty result blocks, no Engram/openspec writes despite reporting "completed"). The orchestrator wrote the code line-by-line against design.md, ran the tests, and persisted the artifacts manually to meet the user's explicit request ("arreglalo").

## Rollout
No DB migration. Redeploy Vercel. The MP webhook URL `https://academix-three-gilt.vercel.app/api/webhooks/mercadopago` is unchanged. Operator must still have `MP_WEBHOOK_SECRET` configured (separate concern, mp-webhook-500-diagnosis).

## Rollback
1. Revert `getPayment` error enrichment (restore plain throw) in mercadopago.ts.
2. Revert webhooks/mercadopago.ts POST+GET catches to the blanket-500 (or the previous mp-webhook-500-diagnosis version).
3. Restore `processPayment` inner try/catch if double-logging was intended.
4. Delete `web/src/test/payments/mercadopago.test.ts`.

## Follow-ups
- Fix `checkout-integration.test.ts` (`createPreapproval` nonexistent → `createPreference`) — separate cleanup.
- Consider a DB-level unique index on `subscription_payments(provider_payment_id)` as a hard replay guard (out of scope, design-design §Open Questions).
