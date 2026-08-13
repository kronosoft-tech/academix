# Apply Progress — mp-webhook-404-and-logging

## Status: ✅ COMPLETE (T1-T4 + T3 classification tests)

NOTE: This change was implemented directly by the orchestrator (not the sdd-apply sub-agent), because the sub-agent repeatedly failed to persist artifacts (empty result blocks, no Engram/openspec writes). Code was written line-by-line following design.md.

## Implementation

| File | Action | What Was Done |
|------|--------|---------------|
| `web/src/lib/payments/mercadopago.ts` | Modified | `getPayment` now enriches its thrown `Error` with `.status`, `.detail`, `.retryAfter` (mirroring `createPreference` pattern). |
| `web/src/pages/api/webhooks/mercadopago.ts` | Modified | Added shared `classifyMpError(err, paymentId)` helper mapping: 404→200{warning}, 401/403→200+error log, 429→503+Retry-After, MP 5xx→503, no-`.status`(DB)→500. Both POST and GET catches use it. Removed `processPayment` redundant inner try/catch (single log point). Missing-secret→500 gate preserved. |
| `web/src/test/payments/mercadopago.test.ts` | Created | 12 tests: 3 verifyWebhookSignature unit + 4 handler (401/malformed-401/500-gate/200-happy) + 5 classification matrix (404→200, 401→200, 429→503, 503→503, DB→500). |

## Verification Evidence
- Focused: `bun run test -- src/test/payments/mercadopago.test.ts` → **12 passed / 0 failed**
- `bunx tsc --noEmit`: **6 pre-existing errors** (db.ts:25, payments.test.ts x5 stripe, checkout-integration.test.ts:61) — ZERO in modified files.

## Deviations
None. Matches design.md D1-D4.
