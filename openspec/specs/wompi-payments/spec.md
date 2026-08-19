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

The system MUST renew Wompi subscriptions via the 08:00 UTC cron `charge-wompi.ts`, selecting `provider='wompi' AND status='active' AND current_period_end <= ? AND payment_source_token IS NOT NULL`, reading `s.plan_id` (real column, `001_subscriptions.sql:4`). Renewal price MUST come from `PLANS` (`web/src/data/plans.ts`); amount `priceCOP * 100`; reference `renewal-<sub.id>-<Date.now()>`. Immediate failure MUST call `startGracePeriod`; extension MUST come only from the webhook (`activateSubscription`).
(Previously: `s.plan` → 500; stale hardcoded prices.)

#### Scenario: Due subscription charged at current price

- GIVEN status=active, `current_period_end <= now`, token set, `plan_id='pro'`
- WHEN charge-wompi runs
- THEN a 149900×100-cent transaction is created, `charged=1`

#### Scenario: Charge failure starts grace

- GIVEN a due subscription
- WHEN `createTransaction` throws
- THEN `startGracePeriod` sets `status='grace'`, `grace_expires_at`=now+7d, `failed=1`

#### Scenario: Unknown plan id fails loud

- GIVEN a due sub whose `plan_id` has no `PLANS` match
- WHEN charge-wompi runs
- THEN it enters grace — no silent default price

#### Scenario: Webhook confirms payment

- GIVEN a transaction created by the cron
- WHEN the Wompi webhook reports APPROVED
- THEN `activateSubscription` extends `current_period_end` by 30 days

### Requirement: Cron Failure Alerting

All three cron handlers MUST wrap execution in try/catch. On unexpected failure the system MUST log prominently, email `SUPPORT_EMAIL`, and respond 500. Missing `CRON_SECRET` MUST stay 500; invalid auth MUST stay 401 without alert.

#### Scenario: Unexpected cron error alerts support

- GIVEN a cron handler throws (e.g. Turso down)
- WHEN the top-level catch runs
- THEN prominent log + alert email to `SUPPORT_EMAIL` + HTTP 500

#### Scenario: Unauthorized request stays 401

- GIVEN a request without Bearer `CRON_SECRET`
- WHEN any cron handler runs
- THEN HTTP 401, no alert email
