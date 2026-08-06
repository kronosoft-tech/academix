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

The system MUST verify Wompi webhook signatures using the events secret before processing.

#### Scenario: Valid webhook signature

- GIVEN a Wompi webhook event with correct signature hash
- WHEN the endpoint receives the event
- THEN the event is processed and subscription updated accordingly

#### Scenario: Invalid webhook signature

- GIVEN a webhook request with incorrect or missing signature
- WHEN the endpoint receives it
- THEN the system returns 401 and does not process the event

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
