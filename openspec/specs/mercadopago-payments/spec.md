# Mercado Pago Payments Specification

## Purpose

Enable LatAm users (outside CO) to subscribe via Mercado Pago Checkout Pro (single-payment preference) for monthly billing.

## Requirements

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

### Requirement: IPN Reconciliation

The system SHOULD run a daily reconciliation cron to catch missed IPN notifications.

#### Scenario: Missed IPN detected

- GIVEN a subscription with last_payment_at > 35 days ago and status=active
- WHEN the reconciliation cron runs
- THEN it queries MP API for preapproval status and updates accordingly
