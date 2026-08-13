# Proposal: MP Webhook 404 & Resilience Fix

## Intent

POST `/api/webhooks/mercadopago` returns HTTP 500 for ALL `getPayment` failures (404, 401/403, 429, MP 5xx, DB errors) because `getPayment` throws a plain `Error` with no `.status` property and the handler catch returns 500 for everything. MP's webhook contract requires 200 to acknowledge receipt — a 500 makes MP treat the endpoint as broken and abandon retries, so affected payments never persist. This change classifies errors by status and responds per MP semantics.

## Scope

### In Scope
- `mercadopago.ts` getPayment: attach `.status = response.status` to thrown error (matching `createPreference` L156-158)
- `webhooks/mercadopago.ts` POST + GET handler catch: classify by `err.status`
- Remove inner try/catch in `processPayment` (fixes double-logging)
- Tests covering 404/401/429/MP-5xx/DB paths

### Out of Scope
- Signature verification (done in archived `mp-webhook-500-diagnosis`)
- New endpoint, MP URL/routing changes
- `verify-mercadopago.ts` dashboard path (separate change)

## Capabilities

### New Capabilities
- None

### Modified Capabilities
- `mercadopago-payments`: "Processing failures MUST return 500 so MP retries" → per-status classification: 404→200, 401/403→200+log, 429→503+Retry-After, 5xx→503, DB/critical→500

## Approach

Enrich `getPayment` error with `.status` (matching existing `createPreference` pattern at L156-158). Classify at handler-level catch by `err.status`. Remove the inner try/catch in `processPayment` (L126-134) so the enriched error bubbles directly to the handler — eliminates double-logging and preserves `.status`.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `web/src/lib/payments/mercadopago.ts` | Modified | getPayment throw: attach `.status` |
| `web/src/pages/api/webhooks/mercadopago.ts` | Modified | POST+GET catch classifies by err.status; remove inner try/catch |
| `web/src/test/payments/*` | New | Tests for 404/401/429/5xx/DB error paths |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| 404→200 stops MP retries; 404 masking token issue drops payments | Med | Warn-level logging with full context; 401/403 classified separately |
| Double-logging if inner catch not removed | Low | Remove inner try/catch in processPayment |
| getPayment forgets `.status` → falls to 500 | Med | Test asserts `.status` on all non-2xx throws |

## Rollback Plan

Revert the 3 changed files + spec delta. No migrations or schema changes. No data impact.

## Dependencies

Complementary to archived `mp-webhook-500-diagnosis` (secret-gate 500 at L39-48 — a different 500 path; not blocking).

## Success Criteria

- [ ] POST returns 200 (not 500) for getPayment 404
- [ ] Returns 503 for MP 5xx and 429
- [ ] Returns 500 only for DB/critical (no `.status`)
- [ ] No double-logging (single log per error)
- [ ] Tests cover 404/401/403/429/5xx/DB
- [ ] `bunx tsc --noEmit` clean (0 new errors)
