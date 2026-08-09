# Stripe Payments Specification

## Purpose

Manage Stripe checkout sessions, 15-day free trial, recurring billing, and subscription lifecycle via webhooks.

## Requirements

### Requirement: Checkout Session Creation

The system MUST create a Stripe Checkout Session with `trial_period_days: 15` for new subscribers selecting a plan.

#### Scenario: New subscriber starts trial via Stripe

- GIVEN a registered user with no active subscription
- WHEN the user selects a plan and clicks "Subscribe with Stripe"
- THEN a Stripe Checkout Session is created with mode=subscription, trial_period_days=15
- AND the user is redirected to Stripe's hosted checkout page

#### Scenario: Existing subscriber attempts checkout

- GIVEN a user with status=active
- WHEN they attempt to create a checkout session
- THEN the system returns an error indicating an active subscription exists

### Requirement: Webhook Event Processing

The system MUST handle Stripe webhook events with signature verification and idempotent processing.

#### Scenario: Successful payment after trial

- GIVEN a subscription with status=trialing in Stripe
- WHEN `invoice.payment_succeeded` webhook fires
- THEN subscription status transitions to active in the control plane
- AND a subscription_payments record is created with provider_payment_id

#### Scenario: Payment failure triggers grace

- GIVEN an active subscription
- WHEN `invoice.payment_failed` webhook fires
- THEN subscription status transitions to grace with grace_expires_at = now + 7 days

#### Scenario: Subscription cancelled

- GIVEN any active or grace subscription
- WHEN `customer.subscription.deleted` webhook fires
- THEN subscription status transitions to cancelled

#### Scenario: Duplicate webhook (idempotency)

- GIVEN a webhook event already processed (provider_payment_id exists)
- WHEN the same event fires again
- THEN the system returns 200 without creating duplicate records

### Requirement: Customer Portal Access

The system SHOULD provide a Stripe Customer Portal link for self-service billing management.

#### Scenario: User accesses billing portal

- GIVEN an authenticated user with a Stripe customer_id
- WHEN they request the billing portal
- THEN a portal session URL is returned for redirect
