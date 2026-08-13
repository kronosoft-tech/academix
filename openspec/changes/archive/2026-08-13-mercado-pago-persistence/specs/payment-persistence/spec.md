# Delta for Payment Persistence (MercadoPago)

## Intent

Guarantee MercadoPago payment persistence in the control-plane `subscription_payments` table. Today only Wompi persists reliably because the MP webhook 500s in production (missing `MP_WEBHOOK_SECRET` — user action, out of scope). This delta adds an owner-checked `verify-mercadopago` endpoint mirroring `verify-wompi` so the dashboard redirect path records the payment even when the webhook failed.

## ADDED Requirements

### Requirement: Verify-MercadoPago Endpoint

The system MUST expose `GET /api/payments/verify-mercadopago` (Astro SSR, `web/src/pages/api/payments/`) with query params `payment_id` and `external_reference`. It MUST return 401 when no valid JWT cookie is present and MUST return 403 when `external_reference` does not start with the JWT `sub`. It MUST fetch the payment from the MP API and MUST NOT activate or persist unless MP `status` is `approved`. It MUST be idempotent — no duplicate `subscription_payments` rows on repeated calls. Success response SHALL be `{ success: true, plan, status }`; failures SHALL include `message`.

#### Scenario: Happy path — approved owned payment

- GIVEN an authenticated user whose JWT `sub` prefixes `external_reference`, and MP returns `status=approved`
- WHEN `GET /api/payments/verify-mercadopago?payment_id=P&external_reference=U-plan-x`
- THEN 200 `{ success: true, plan, status: 'active' }`; subscription activates; one `subscription_payments` row with `provider='mercadopago'`

#### Scenario: Missing authentication

- GIVEN no valid JWT cookie
- WHEN the endpoint is called
- THEN 401 with `message`; no activation or persistence

#### Scenario: Ownership mismatch

- GIVEN a JWT whose `sub` does not prefix `external_reference`
- WHEN the endpoint is called
- THEN 403 with `message`; no activation or persistence

#### Scenario: MP payment not approved

- GIVEN MP returns a non-`approved` status (pending/rejected)
- WHEN the endpoint is called
- THEN no activation or persistence; response reflects the non-approved status

#### Scenario: MP API error

- GIVEN the MP payment fetch fails (non-2xx or network error)
- WHEN the endpoint is called
- THEN 502 with logged error; no activation or persistence

### Requirement: Idempotent Double Verification

The system MUST check for an existing `subscription_payments` row by `provider_payment_id` before inserting, so a repeated verify of the same payment does not double-record or double-activate.

#### Scenario: Webhook and verify both succeed

- GIVEN the webhook already recorded payment `P` and activated the subscription
- WHEN the user returns via redirect and verifies `payment_id=P`
- THEN 200 success with no second payment row and no re-activation

### Requirement: Environment Documentation

The documentation MUST note that `MP_WEBHOOK_SECRET` is required for webhook signature verification and that its absence makes the webhook 500 (MP retries), with the verify endpoint as the dashboard-side persistence fallback.

#### Scenario: Docs locate the secret

- GIVEN a developer setting up `web/` env vars
- WHEN reading the env documentation
- THEN they find `MP_WEBHOOK_SECRET` listed with its location and failure mode

## MODIFIED Requirements

### Requirement: Replay-Safe Payment Activation

The system MUST reject an already-recorded `provider_payment_id`, MUST resolve the subscription by `provider_subscription_id = external_reference` (lazily creating a trial row), MUST treat malformed references as a no-op, MUST record payments with `provider='mercadopago'` reusing lifecycle activation, and MUST accept an optional `expectedUserId` guard: when provided, activation MUST NOT proceed unless `external_reference` starts with it.
(Previously: no ownership guard; provider persistence was implicit in the webhook path)

#### Scenario: Replayed payment

- GIVEN a `payment_id` already present in `subscription_payments`
- WHEN activation runs again (webhook retry or dashboard verify)
- THEN no second payment row; no duplicate activation

#### Scenario: Desktop-first user

- GIVEN an approved payment whose `external_reference` matches no subscription
- WHEN activation runs from verify with the caller's `sub` as `expectedUserId`
- THEN a trial row is created, the subscription activates, and a `provider='mercadopago'` payment is recorded

#### Scenario: Ownership-guarded activation mismatch

- GIVEN `expectedUserId` provided and `external_reference` does not start with it
- WHEN activation runs
- THEN nothing is activated or persisted

### Requirement: Dashboard Checkout Pro Redirect Handling

The dashboard MUST process Checkout Pro redirects via `payment_id` or `collection_id` (not `preapproval_id`) by fetching `GET /api/payments/verify-mercadopago` with the JWT session, MUST show success ONLY when the response is `{ success: true }`, SHOULD show an informational message for non-approved statuses, and MUST surface endpoint failures.
(Previously: dashboard verified the payment against MP inline and called activation directly)

#### Scenario: Approved redirect

- GIVEN `/dashboard?payment_id=X&status=approved&external_reference=Y` with a logged-in user
- WHEN the dashboard handler fetches the verify endpoint
- THEN success UI shown only on `{ success: true }`; subscription active; payment recorded

#### Scenario: Endpoint failure surfaced

- GIVEN the verify endpoint returns 401/403/502
- WHEN the dashboard handler receives it
- THEN the dashboard shows the failure message and no success state

## REMOVED Requirements

None.

## Non-Goals (out of scope)

- Configuring `MP_WEBHOOK_SECRET` itself (user action; webhook remains 500 until set)
- MP renewal/reconciliation cron for this change (existing IPN reconciliation unchanged)
- Stripe support and subscription plan changes
