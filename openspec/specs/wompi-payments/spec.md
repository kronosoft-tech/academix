# Wompi Payments Specification

## Purpose

Enable Colombian users to pay via Wompi widget with tokenized card storage for recurring monthly charges.

## Requirements

### Requirement: Wompi Widget Checkout

The system MUST render the Wompi payment widget for users routed to the CO gateway.

#### Scenario: Successful first payment via Wompi

- GIVEN a user in Colombia with no active subscription
- WHEN they complete payment through the Wompi widget
- THEN a payment_source token is stored for the user
- AND subscription status transitions to active
- AND a subscription_payments record is created

#### Scenario: Widget payment fails

- GIVEN a user interacting with the Wompi widget
- WHEN the transaction is declined
- THEN the system displays an error message
- AND no subscription status change occurs

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

### Requirement: Recurring Charge via Cron

The system MUST charge tokenized cards monthly via cron using stored payment_source tokens.

#### Scenario: Successful monthly charge

- GIVEN a Wompi subscription due for renewal (last_payment_at + 30 days <= now)
- WHEN the recurring charge cron executes
- THEN a transaction is created against the stored payment_source
- AND on success, subscription period extends by 30 days

#### Scenario: Recurring charge fails

- GIVEN a due Wompi subscription
- WHEN the charge attempt fails
- THEN subscription transitions to grace with grace_expires_at = now + 7 days
- AND a reminder email is triggered
