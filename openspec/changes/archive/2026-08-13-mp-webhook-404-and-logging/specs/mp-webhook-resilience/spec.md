# Delta for MP Webhook Resilience

**Non-goals:** Signature verification (already done in archived `mp-webhook-500-diagnosis`), new endpoints, MP URL/routing changes.

## ADDED Requirements

### Requirement: getPayment Enriches Thrown Error with HTTP Status

The system MUST attach the MP API HTTP response `.status` to the Error thrown by `getPayment` on non-2xx responses, matching the existing `createPreference` pattern (`mercadopago.ts` L156–158).

#### Scenario: 404 throws with .status

- GIVEN `getPayment` is called for a payment MP cannot resolve
- WHEN MP responds with 404
- THEN the thrown Error has `.status === 404`

#### Scenario: 429 throws with .status

- GIVEN `getPayment` is called when MP rate-limits
- WHEN MP responds with 429
- THEN the thrown Error has `.status === 429`

### Requirement: Webhook Handler Classifies Errors by Status

The webhook handler MUST classify `getPayment`/MP-API failures by `err.status`: 404 → 200 + warning; 401/403 → 200 + `MP_API_AUTH_ERROR` log; 429 → 503 + `Retry-After`; MP 5xx → 503; errors without `.status` (DB/TURSO) → 500.

| err.status | HTTP response | Log level |
|------------|---------------|-----------|
| 404        | 200 + warning | warn      |
| 401/403    | 200           | error (`MP_API_AUTH_ERROR`) |
| 429        | 503 + Retry-After | warn   |
| 5xx        | 503           | error     |
| none (DB)  | 500           | error     |

#### Scenario: 404 acknowledged with 200

- GIVEN a valid signature but `getPayment` throws with `.status = 404`
- WHEN the handler catch runs
- THEN 200 `{ received: true, warning: "payment not found" }` + warn log

#### Scenario: 401 token error acknowledged with 200

- GIVEN `getPayment` throws with `.status = 401`
- WHEN the handler catch runs
- THEN 200 `{ received: true }` + error log tagged `MP_API_AUTH_ERROR`

#### Scenario: 429 returns 503 with Retry-After

- GIVEN `getPayment` throws with `.status = 429`
- WHEN the handler catch runs
- THEN 503 + `Retry-After` header + warn log

#### Scenario: MP 5xx returns 503

- GIVEN MP API returns 500 to `getPayment`
- WHEN the handler catch runs
- THEN 503 + error log

#### Scenario: DB/TURSO down returns 500

- GIVEN `activateApprovedPayment` throws with no `.status` (TURSO down)
- WHEN the handler catch runs
- THEN 500 + error log

#### Scenario: Single log line per error

- GIVEN `getPayment` throws inside `processPayment`
- WHEN the error reaches the handler catch
- THEN only one log line is emitted (inner try/catch removed)

## MODIFIED Requirements

### Requirement: Webhook Signature Verification and Payment Processing

The system MUST verify MP webhook POSTs via `x-signature` — HMAC-SHA256 over `id:<data.id>;request-id:<x-request-id>;ts:<ts>;` with `MP_WEBHOOK_SECRET` (timing-safe). Invalid JSON MUST return 400, missing secret 500, invalid signature 401. Verified `payment.created`/`payment.updated`/`topic=payment` notifications MUST fetch the payment and activate it when approved with `external_reference`. Processing failures MUST be classified by MP API HTTP status: 404/401/403 → 200 ack, 429/MP 5xx → 503, only DB/critical (no `.status`) → 500; 500 MUST NOT be returned for 404/401/403. Non-approved payments MUST be acknowledged without activating. (Previously: "Processing failures MUST return 500 so MP retries" — a blanket 500 for ALL errors including 404/401/403.)

**Robustness guarantee (no-throw):** `verifyWebhookSignature` MUST be wrapped so it NEVER throws — any exception or malformed header MUST return `false` (→ 401). The handler call site MUST also be guarded with try/catch → 401. The missing-secret→500 gate (deployment-time signal) is preserved and is NOT subject to this guarantee.

#### Scenario: Valid signature, approved payment

- GIVEN a POST with a valid `x-signature` for `data.id` and an approved payment carrying `external_reference`
- WHEN the webhook processes it
- THEN 200 and the payment is activated

#### Scenario: Processing failure classified by status

- GIVEN a valid signature but `getPayment` throws with an HTTP `.status`
- WHEN the webhook catch runs
- THEN 200 (404/401/403), 503 (429/MP 5xx), or 500 (no `.status` / DB), with a single log line

#### Scenario: Missing x-signature header → 401 (not 500)

- GIVEN `MP_WEBHOOK_SECRET` is set
- WHEN a POST arrives with no `x-signature` header
- THEN `verifyWebhookSignature` returns `false` without throwing
- AND the handler responds 401 Invalid signature (not 500)

#### Scenario: Malformed non-hex v1 → 401 (not 500)

- GIVEN `MP_WEBHOOK_SECRET` is set and `x-signature` is present
- WHEN `v1` is malformed (non-hex)
- THEN `verifyWebhookSignature` returns `false` (Node `Buffer.from(...,'hex')` does not throw) — no exception can propagate
- AND the handler responds 401 Invalid signature (not 500)

#### Scenario: Missing secret returns 500 (deployment-time gate, preserved)

- GIVEN `MP_WEBHOOK_SECRET` is unset in Vercel
- WHEN the webhook receives any POST
- THEN 500 "Mercado Pago webhook secret not configured" is returned before signature verification
- AND this is the operator-action gate (add the var + redeploy), not a code defect
