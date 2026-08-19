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

The system MUST provide a 7-day grace period on payment failure. Grace warnings MUST be sent by `send-reminders` for `status='grace' AND grace_expires_at > now`. Grace expiry without payment MUST end the subscription `cancelled` (via `cancelSubscription`), not `expired`.
(Previously: grace expiry → `expired`; `s.grace_end` → 500.)

#### Scenario: Payment failure triggers grace

- GIVEN an active subscription
- WHEN a payment fails via any webhook
- THEN status → grace, `grace_expires_at`=now+7d, access retained

#### Scenario: Grace expires without payment

- GIVEN status=grace and `grace_expires_at <= now`
- WHEN expire-subscriptions runs
- THEN status → cancelled; login blocked

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

### Requirement: Lazy Trial Subscription Creation

All activation paths (Wompi verify/webhook, MP checkout/activation) MUST use `getOrCreateTrialSubscription`: existing trial row, else any existing row, else a new trial row using web-schema columns (`plan_id`, `trial_starts_at`, `grace_expires_at`).

#### Scenario: Desktop-first user pays

- GIVEN a user with no subscription row
- WHEN any activation path runs
- THEN a trial row with `plan_id` is created and activation proceeds

### Requirement: Grace Warning Reminder Cron

`send-reminders` MUST query grace subs with `s.grace_expires_at` (real column, `002_multi_gateway.sql:10`), filter `status='grace' AND grace_expires_at > ?`, compute `days_left`, and send `sendGraceWarning` per sub. Trial reminders (`trial_end`) MUST stay unchanged.

#### Scenario: Grace warning sent

- GIVEN status=grace and `grace_expires_at` = now + 3 days
- WHEN send-reminders runs
- THEN `sendGraceWarning` sent with `days_left=3`, response `graceWarnings=1`

#### Scenario: No grace subscriptions

- GIVEN no subs with `status='grace' AND grace_expires_at > now`
- WHEN send-reminders runs
- THEN `graceWarnings=0` and HTTP 200

### Requirement: Schema-Accurate Test Fixtures

The payments test suite MUST mock only real columns — `plan_id`, `grace_expires_at`; never `plan`, `grace_start`, `grace_end` (`payments.test.ts:196-227`, `subscription-lifecycle.test.ts:76-83`). Cron tests MUST assert real columns in SQL so drift fails CI.

#### Scenario: Stale fixture fails

- GIVEN a fixture mocking `plan`/`grace_end` instead of `plan_id`/`grace_expires_at`
- WHEN the test suite runs
- THEN the test fails (missing column)

#### Scenario: Cron SQL asserts real columns

- GIVEN charge-wompi and send-reminders tests
- WHEN they assert the executed SQL
- THEN `s.plan_id` and `s.grace_expires_at` are present
