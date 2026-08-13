# Delta for mp-webhook-robustness

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Webhook Signature Verification and Payment Processing

The system MUST verify MP webhook POSTs via `x-signature` — HMAC-SHA256 over `id:<data.id>;request-id:<x-request-id>;ts:<ts>;` with `MP_WEBHOOK_SECRET` (timing-safe). `verifyWebhookSignature` MUST be wrapped so it NEVER throws — any exception or malformed header MUST return `false` (→ 401). Invalid JSON MUST return 400, missing secret 500, invalid signature 401. Verified `payment.created`/`payment.updated`/`topic=payment` notifications MUST fetch the payment and activate it when approved with `external_reference`. Processing failures MUST return 500 so MP retries; non-approved payments MUST be acknowledged without activating.

(Previously: an uncaught exception in `verifyWebhookSignature` or its call site could propagate as HTTP 500; now the call is guarded so signature failures NEVER 500. The missing-secret→500 gate is preserved as an intentional deployment-time signal.)

#### Scenario: Missing secret returns 500 (deployment-time gate, preserved)

- GIVEN `MP_WEBHOOK_SECRET` is unset in Vercel
- WHEN the webhook receives any POST
- THEN 500 "Mercado Pago webhook secret not configured" is returned before signature verification
- AND this is the operator-action gate (add the var + redeploy), not a code defect

#### Scenario: Valid signature, approved payment

- GIVEN a POST with a valid `x-signature` for `data.id` and an approved payment carrying `external_reference`
- WHEN the webhook processes it
- THEN 200 and the payment is activated

#### Scenario: Processing failure

- GIVEN a valid signature but the payment fetch or activation throws
- WHEN the webhook runs
- THEN 500 with a logged error, so MP retries

#### Scenario: No x-signature header → 401 (not 500)

- GIVEN `MP_WEBHOOK_SECRET` is set
- WHEN a POST arrives with no `x-signature` header
- THEN `verifyWebhookSignature` returns `false` without throwing
- AND the handler responds 401 Invalid signature (not 500)

#### Scenario: Malformed non-hex v1 → 401 (not 500)

- GIVEN `MP_WEBHOOK_SECRET` is set and `x-signature` is present
- WHEN `v1` is malformed (non-hex)
- THEN `verifyWebhookSignature` returns `false` (Node `Buffer.from(...,'hex')` does not throw) — no exception can propagate
- AND the handler responds 401 Invalid signature (not 500)
