# Mercado Pago Payments Specification

## Purpose

Enable LatAm users (outside CO) to subscribe via Mercado Pago Checkout Pro (single-payment preference) for monthly billing. This spec also governs webhook signature verification hardening and deployment-time env var provisioning.

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

**Robustness guarantee (no-throw):** `verifyWebhookSignature` MUST be wrapped so it NEVER throws — any exception or malformed header MUST return `false` (→ 401). The handler call site MUST also be guarded with try/catch → 401. The missing-secret→500 gate (deployment-time signal) is preserved and is NOT subject to this guarantee.

#### Scenario: Valid signature, approved payment

- GIVEN a POST with a valid `x-signature` for `data.id` and an approved payment carrying `external_reference`
- WHEN the webhook processes it
- THEN 200 and the payment is activated

#### Scenario: Processing failure

- GIVEN a valid signature but the payment fetch or activation throws
- WHEN the webhook runs
- THEN 500 with a logged error, so MP retries

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

### Requirement: Webhook Secret Documented in .env.example

The system SHOULD document `MP_WEBHOOK_SECRET` in `web/src/.env.example` alongside `MP_ACCESS_TOKEN` and `MP_API_URL`, so operators provisioning the webhook secret can copy the value.

#### Scenario: env.example contains MP_WEBHOOK_SECRET

- GIVEN the repository is checked out
- WHEN `web/src/.env.example` is read
- THEN it contains a `MP_WEBHOOK_SECRET=` line after `MP_API_URL`
- AND `MP_ACCESS_TOKEN` and `MP_API_URL` lines are present

### Requirement: Vercel Rollout Checklist Includes MP_WEBHOOK_SECRET

`web/docs/vercel-rollout.md` MUST list `bunx vercel env add MP_WEBHOOK_SECRET production` in its env-add checklist and document the expected post-fix behavior (manual payload without `x-signature` → 401).

#### Scenario: rollout checklist references the secret

- GIVEN an operator following `vercel-rollout.md` deployment steps
- WHEN they reach the env-add section
- THEN `MP_WEBHOOK_SECRET` is listed alongside `MP_ACCESS_TOKEN`
- AND the checklist notes that adding the var + redeploying resolves the 500

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
