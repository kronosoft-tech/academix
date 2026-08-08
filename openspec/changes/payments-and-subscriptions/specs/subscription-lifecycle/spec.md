# Subscription Lifecycle Specification

## Purpose

Manage subscription state machine transitions: trial (15d) → active → grace (7d) → expired/cancelled, with login enforcement.

## Requirements

### Requirement: Trial Period

The system MUST create a 15-day trial starting at user registration. No card required.

#### Scenario: Trial starts at registration

- GIVEN a new user registers on the platform
- WHEN registration completes
- THEN a subscription record is created with status=trialing, trial_ends_at = now + 15 days

#### Scenario: Trial expires without payment

- GIVEN a subscription with status=trialing and trial_ends_at <= now
- WHEN the expire-subscriptions cron runs
- THEN subscription status transitions to expired

### Requirement: Grace Period

The system MUST provide a 7-day grace period on payment failure before blocking access.

#### Scenario: Payment failure triggers grace

- GIVEN an active subscription
- WHEN a payment fails (via any gateway webhook)
- THEN status transitions to grace with grace_expires_at = now + 7 days
- AND the user retains access during the grace period

#### Scenario: Grace period expires

- GIVEN a subscription with status=grace and grace_expires_at <= now
- WHEN the expire-subscriptions cron runs
- THEN subscription status transitions to expired
- AND login is blocked on next attempt

### Requirement: Login Enforcement

The system MUST block login for users with expired or cancelled subscriptions.

#### Scenario: Expired user attempts login

- GIVEN a user with subscription status=expired
- WHEN they attempt to log in (web or desktop)
- THEN login is denied with message indicating subscription expired
- AND a link to reactivate/subscribe is provided

#### Scenario: Active/trialing user logs in

- GIVEN a user with subscription status=active or trialing
- WHEN they log in
- THEN access is granted normally

### Requirement: Successful Payment Reactivation

The system MUST reactivate expired subscriptions upon successful payment.

#### Scenario: Expired user pays successfully

- GIVEN a user with status=expired
- WHEN a successful payment webhook is received
- THEN subscription transitions to active
