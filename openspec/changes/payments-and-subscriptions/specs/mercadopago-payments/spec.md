# Mercado Pago Payments Specification

## Purpose

Enable LatAm users (outside CO) to subscribe via Mercado Pago preapproval (auto-debit) for recurring monthly billing.

## Requirements

### Requirement: Preapproval Creation

The system MUST create a Mercado Pago preapproval (subscription) with auto_recurring for the selected plan amount.

#### Scenario: New subscriber starts via Mercado Pago

- GIVEN a registered user routed to the MP gateway
- WHEN the user selects a plan and clicks "Subscribe with Mercado Pago"
- THEN a preapproval is created with reason=plan_name, auto_recurring.frequency=1, frequency_type=months
- AND the user is redirected to MP's checkout page

#### Scenario: User cancels at MP checkout

- GIVEN a preapproval init_point was generated
- WHEN the user abandons the MP checkout page
- THEN no subscription record is created or modified

### Requirement: IPN Webhook Processing

The system MUST handle Mercado Pago IPN notifications for preapproval status changes.

#### Scenario: Preapproval authorized (payment successful)

- GIVEN an IPN notification with type=subscription_preapproval, status=authorized
- WHEN the webhook endpoint processes it
- THEN subscription status transitions to active
- AND a subscription_payments record is created

#### Scenario: Preapproval paused (payment failed)

- GIVEN an IPN notification with status=paused
- WHEN the webhook endpoint processes it
- THEN subscription transitions to grace with grace_expires_at = now + 7 days

#### Scenario: Preapproval cancelled

- GIVEN an IPN notification with status=cancelled
- WHEN the webhook endpoint processes it
- THEN subscription status transitions to cancelled

### Requirement: IPN Reconciliation

The system SHOULD run a daily reconciliation cron to catch missed IPN notifications.

#### Scenario: Missed IPN detected

- GIVEN a subscription with last_payment_at > 35 days ago and status=active
- WHEN the reconciliation cron runs
- THEN it queries MP API for preapproval status and updates accordingly
