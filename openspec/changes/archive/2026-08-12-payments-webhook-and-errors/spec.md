# Spec: payments-webhook-and-errors

Retrospective delta for commit `42c2d86` (`web/`). Touches `wompi-payments`, `mercadopago-payments`, `subscription-lifecycle` from `payments-and-subscriptions`.

## wompi-payments

### MODIFIED Requirements

### Requirement: Wompi Webhook Signature Verification

The system MUST verify Wompi webhooks by SHA-256 over the values of fields listed in `event.signature.properties` (resolved against the payload, in order) + `event.timestamp` + `WOMPI_EVENTS_SECRET`, falling back to `transaction.id`, `transaction.status`, `transaction.amount_in_cents` when `signature.properties` is absent. Missing `x-event-checksum` or non-matching checksum MUST yield 401 with no processing.
(Previously: hardcoded hash over `id + status + reference` — real events 401'd)

#### Scenario: Valid checksum

- GIVEN an event whose `signature.properties` values hash to the received checksum
- WHEN POST /api/webhooks/wompi receives it
- THEN the event is processed

#### Scenario: Missing or non-matching checksum

- GIVEN no `x-event-checksum`, or a checksum computed over other fields (e.g. including `reference`)
- WHEN the endpoint receives it
- THEN 401 and nothing is processed

### ADDED Requirements

### Requirement: Wompi Transaction Ownership Verification

The system MUST fetch Wompi transactions with `Authorization: Bearer <WOMPI_PUBLIC_KEY>` and MUST NOT activate a transaction whose reference does not embed the caller's user id (first 5 UUID parts of `{userId}-{planId}-{uuid}`) or whose status is not APPROVED. API failures MUST return 502 with a logged error. The widget MUST call verify-wompi on APPROVED results before redirecting.

#### Scenario: Approved transaction owned by the caller

- GIVEN an authenticated user whose id matches the reference's first 5 UUID parts and an APPROVED transaction
- WHEN POST /api/payments/verify-wompi { transactionId }
- THEN 200 with plan and status=active; subscription activates; one payment row recorded

#### Scenario: Ownership mismatch

- GIVEN a transaction whose reference embeds another user's id
- WHEN verified (endpoint or dashboard return)
- THEN endpoint 400 / dashboard error banner; nothing activates

### Requirement: Wompi Webhook First-Payment Activation

The system MUST derive `plan_id` from the first-payment reference, obtain the subscription via `getOrCreateTrialSubscription`, persist `provider`, `provider_subscription_id`, `payment_source_token`, and `plan_id`, activate, and record the payment idempotently.

#### Scenario: Desktop-first user's first payment

- GIVEN a verified APPROVED event whose user has no subscription row
- WHEN the webhook processes the first-payment branch
- THEN a trial row is created, `plan_id` set from the reference, subscription activated, payment recorded

## mercadopago-payments

### REMOVED Requirements

### Requirement: Preapproval Creation

(Reason: Colombia does not support the MP preapproval/subscriptions API; the flow uses Checkout Pro.)
(Migration: Checkout Pro Preference Creation)

### Requirement: IPN Webhook Processing

(Reason: replaced by a payment-notification webhook with `x-signature` verification.)
(Migration: Webhook Signature Verification and Payment Processing)

### ADDED Requirements

### Requirement: Checkout Pro Preference Creation

The system MUST create a single-payment Checkout Pro preference with `auto_return=approved`, `external_reference = {userId}-{planId}-{uuid}`, and webhook `notification_url`, redirecting to `init_point` (`sandbox_init_point` for TEST- tokens). On MP rejection it MUST return 502 including MP's `detail`, displayed in the UI.

#### Scenario: Preference created

- GIVEN an authenticated user selecting a plan
- WHEN POST /api/checkout/mercadopago { planId }
- THEN 200 with the checkout URL; reference stored as `provider_subscription_id`

#### Scenario: MP rejects the preference

- GIVEN MP returns an error body containing `detail`
- WHEN `createPreference` fails
- THEN 502 with error + detail; UI shows the detail

### Requirement: Webhook Signature Verification and Payment Processing

The system MUST verify MP webhook POSTs via `x-signature` — HMAC-SHA256 over `id:<data.id>;request-id:<x-request-id>;ts:<ts>;` with `MP_WEBHOOK_SECRET` (timing-safe). Invalid JSON MUST return 400, missing secret 500, invalid signature 401. Verified `payment.created`/`payment.updated`/`topic=payment` notifications MUST fetch the payment and activate it when approved with `external_reference`. Processing failures MUST return 500 so MP retries; non-approved payments MUST be acknowledged without activating.

#### Scenario: Valid signature, approved payment

- GIVEN a POST with a valid `x-signature` for `data.id` and an approved payment carrying `external_reference`
- WHEN the webhook processes it
- THEN 200 and the payment is activated

#### Scenario: Processing failure

- GIVEN a valid signature but the payment fetch or activation throws
- WHEN the webhook runs
- THEN 500 with a logged error, so MP retries

### Requirement: Replay-Safe Payment Activation

`activateApprovedPayment` MUST reject an already-recorded `provider_payment_id`, MUST resolve the subscription by `provider_subscription_id = external_reference` (lazily creating a trial row), and MUST treat malformed references as a no-op.

#### Scenario: Replayed payment

- GIVEN a `payment_id` already present in `subscription_payments`
- WHEN activation runs again (webhook retry or dashboard + webhook)
- THEN no second payment row; no duplicate activation

#### Scenario: Desktop-first user

- GIVEN an approved payment whose `external_reference` matches no subscription
- WHEN activation runs
- THEN a trial row is created, the subscription activates, and a payment is recorded

### Requirement: Dashboard Checkout Pro Redirect Handling

The dashboard MUST process Checkout Pro redirects via `payment_id` or `collection_id` (not `preapproval_id`), verify the payment against MP, call `activateApprovedPayment` when approved with `external_reference`, show an informational message when pending, and surface failures.

#### Scenario: Approved redirect

- GIVEN `/dashboard?payment_id=X&status=approved&external_reference=Y`
- WHEN the dashboard SSR handler runs
- THEN the payment is verified and `activateApprovedPayment` is called

## subscription-lifecycle

### ADDED Requirements

### Requirement: Lazy Trial Subscription Creation

All activation paths (Wompi verify/webhook, MP checkout/activation) MUST use `getOrCreateTrialSubscription`: existing trial row, else any existing row, else a new trial row using web-schema columns (`plan_id`, `trial_starts_at`, `grace_expires_at`).

#### Scenario: Desktop-first user pays

- GIVEN a user with no subscription row
- WHEN any activation path runs
- THEN a trial row with `plan_id` is created and activation proceeds
