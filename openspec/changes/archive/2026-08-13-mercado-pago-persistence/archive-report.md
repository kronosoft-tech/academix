# Archive Report: MercadoPago Payment Persistence (verify-mercadopago)

**Change**: `mercado-pago-persistence`
**Date**: 2026-08-13
**Status**: ✅ Complete — planned, implemented, verified

## Summary
Guarantees MercadoPago payment persistence in the control-plane `subscription_payments` table via an owner-checked `GET /api/payments/verify-mercadopago` endpoint. This is the dashboard-side fallback when the MP webhook 500s (production root cause: missing `MP_WEBHOOK_SECRET` — user action, documented as out of scope). Wompi persistence was already working and is unaffected.

## Implementation Evidence
All 6 tasks (T1-T6) complete.

| File | Action | What Was Done |
|------|--------|---------------|
| `web/src/pages/api/payments/verify-mercadopago.ts` | Created | GET endpoint (Astro SSR, `prerender = false`): JWT auth via `getFullTokenPayload` (401), `payment_id`/`external_reference` params (400), ownership check `externalReference.startsWith(payload.sub)` (403), `getPayment` to MP API (502 on failure), status gate `=== 'approved'` (400 otherwise), `activateApprovedPayment` with `expectedUserId: payload.sub`, idempotent INSERT, response `{ success, plan, status: 'active' }` |
| `web/src/lib/payments/mercadopago.ts` | Modified | `activateApprovedPayment` gains optional `expectedUserId?: string` guard: early-returns unless `externalReference.startsWith(expectedUserId)`. Webhook path never passes the guard. Replay guard (SELECT before INSERT) unchanged. |
| `web/src/pages/dashboard/index.astro` | Modified | Replaced inline MP verification with same-origin fetch to `/api/payments/verify-mercadopago`; `paymentSuccess` set only on `{ success: true }`; surfaces 401/403/502/400-status failures |
| `web/src/test/payments/verify-mercadopago.test.ts` | Created | 12 tests, 5 suites: happy path (asserts `expectedUserId`), auth 401, ownership 403, idempotency (INSERT once via real impl + mocked DB), MP API failure 502 |
| `web/docs/vercel-rollout.md` | Modified | Added MP env vars (MP_WEBHOOK_SECRET, MP_ACCESS_TOKEN, MP_API_URL) to env rollout + MP sandbox test cards/context |

## Test Results
- Focused suite (`verify-mercadopago.test.ts`): **12 passed / 0 failed** (92ms)
- Webhook-flows suite: **15/15 passed** (webhook callers unchanged)
- `bunx tsc --noEmit` (from `web/`): **8 pre-existing errors** — all in unmodified files (`db.ts:25` proxy type, `payments.test.ts` x5 nonexistent `../lib/payments/stripe`, `checkout-integration.test.ts:61` stale type comparison). **Zero new errors** in any modified file.
- Full web suite: **7 pre-existing failures** (`payments.test.ts` 5 Stripe import errors, `checkout-integration.test.ts` 2 stale geoToGateway assertions). None caused by this change.
- `bun run build` (astro): **Complete!** ✓

## Pre-existing Technical Debt (NOT fixed — out of scope)
- `web/src/test/payments/checkout-integration.test.ts` mocks `createPreapproval` (nonexistent) — real function is `createPreference`. Stale test, unrelated to this change.

## PR Boundary & Commit Plan
- Mode: **size-exception** single PR with work-unit commits.
- Final authored diff: **466 changed lines** (437 add + 29 del), marginally above the 400-line budget — documented as `size:exception`.
- Work-unit commit splits (per tasks.md): (1) activateApprovedPayment guard, (2) verify-mercadopago endpoint + tests, (3) dashboard wiring, (4) docs. All implemented together; committed as one cohesive change.

## Rollback
1. Delete `web/src/pages/api/payments/verify-mercadopago.ts` + `web/src/test/payments/verify-mercadopago.test.ts`.
2. Revert `mercadopago.ts` (drop `expectedUserId` field + guard — webhook never passed it).
3. Revert `web/src/pages/dashboard/index.astro` to inline MP verification (restore removed import + Spanish strings).
4. Revert `web/docs/vercel-rollout.md` MP rows.
No DB migration to reverse; `subscription_payments` schema unchanged; duplicate-safe by design.

## Follow-ups
- Backfill existing web-registered users into per-user DBs (separate change).
- Configure `MP_WEBHOOK_SECRET` in Vercel production (MP Dashboard → app → Webhooks → Secret key) — non-goal of this change; the verify endpoint functions as the dashboard-side persistence fallback regardless.
- Fix stale `checkout-integration.test.ts` (`createPreapproval` → `createPreference`) — separate cleanup.
