# Tasks: MP Webhook 404 & Resilience Fix

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~120 |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | Single PR |
| Delivery strategy | auto-chain |
| Chain strategy | pending |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: pending
400-line budget risk: Low

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| 1 | Enrich `getPayment` error + refactor handler classification + tests | Single PR | `bun run test -- src/test/payments/mercadopago.test.ts` | N/A — unit/integration only, no live MP secret required | Revert `mercadopago.ts` + `webhooks/mercadopago.ts` restores pre-hardening 500 behavior |

## Phase 1: Foundation (Error Shape)

- [ ] 1.1 Add exported `MpApiError` interface to `web/src/lib/payments/mercadopago.ts` (status?, detail?, retryAfter?).
- [ ] 1.2 Enrich `getPayment` throw (L185-188): cast to `MpApiError`, set `.status = response.status`, `.detail`, `.retryAfter` from `retry-after` header (mirror `createPreference` L156-158).

## Phase 2: Core Implementation (Handler Classification)

- [ ] 2.1 Extract shared `classifyMpError(err)` helper in `web/src/pages/api/webhooks/mercadopago.ts` returning `{status, body, level, tag}`.
- [ ] 2.2 Replace POST catch (L84-93): classify via helper — 404→200 warning, 401/403→200+error log `MP_API_AUTH_ERROR`, 429→503+Retry-After, 5xx→503, none→500. Single log at handler.
- [ ] 2.3 Apply same `classifyMpError` to GET catch (L109-115).
- [ ] 2.4 Remove `processPayment` inner try/catch (L126-134) so `getPayment` error bubbles with `.status` intact.

## Phase 3: Testing

- [ ] 3.1 Add `getPayment` unit test: mock `fetch` → 404/429; assert thrown `MpApiError.status`.
- [ ] 3.2 Add handler classification tests: 404→200{warning}, 401→200, 429→503+Retry-After, 500→503, DB error no-status→500.
- [ ] 3.3 Assert single log line: `console.error`/`console.warn` called exactly once (inner catch removed).

## Phase 4: Verification

- [ ] 4.1 Run `bunx tsc --noEmit` (root) — type-check.
- [ ] 4.2 Run `bun run test -- src/test/payments/mercadopago.test.ts` — all green.
- [ ] 4.3 Note pre-existing failures (e.g. Stripe, unrelated) in verification report — do not block on those.
