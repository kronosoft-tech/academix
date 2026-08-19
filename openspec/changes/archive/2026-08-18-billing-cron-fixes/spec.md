# Delta for Billing Cron Fixes

**Non-goals:** No backfill (forward-only); no DB `plans` table; no price grandfathering; no charge idempotency (W9 → `payment-integrity`); no webhook verification (W4)/unique constraints (W10/W20); no desktop changes.

## wompi-payments

### MODIFIED Requirements

#### Requirement: Recurring Charge via Cron

The system MUST renew Wompi subscriptions via the 08:00 UTC cron `charge-wompi.ts`, selecting `provider='wompi' AND status='active' AND current_period_end <= ? AND payment_source_token IS NOT NULL`, reading `s.plan_id` (real column, `001_subscriptions.sql:4`). Renewal price MUST come from `PLANS` (`web/src/data/plans.ts`); amount `priceCOP * 100`; reference `renewal-<sub.id>-<Date.now()>`. Immediate failure MUST call `startGracePeriod`; extension MUST come only from the webhook (`activateSubscription`).
(Previously: `s.plan` → 500; stale hardcoded prices.)

##### Scenario: Due subscription charged at current price

- GIVEN status=active, `current_period_end <= now`, token set, `plan_id='pro'`
- WHEN charge-wompi runs
- THEN a 149900×100-cent transaction is created, `charged=1`

##### Scenario: Charge failure starts grace

- GIVEN a due subscription
- WHEN `createTransaction` throws
- THEN `startGracePeriod` sets `status='grace'`, `grace_expires_at`=now+7d, `failed=1`

##### Scenario: Unknown plan id fails loud

- GIVEN a due sub whose `plan_id` has no `PLANS` match
- WHEN charge-wompi runs
- THEN it enters grace — no silent default price

##### Scenario: Webhook confirms payment

- GIVEN a transaction created by the cron
- WHEN the Wompi webhook reports APPROVED
- THEN `activateSubscription` extends `current_period_end` by 30 days

### ADDED Requirements

#### Requirement: Cron Failure Alerting

All three cron handlers MUST wrap execution in try/catch. On unexpected failure the system MUST log prominently, email `SUPPORT_EMAIL`, and respond 500. Missing `CRON_SECRET` MUST stay 500; invalid auth MUST stay 401 without alert.

##### Scenario: Unexpected cron error alerts support

- GIVEN a cron handler throws (e.g. Turso down)
- WHEN the top-level catch runs
- THEN prominent log + alert email to `SUPPORT_EMAIL` + HTTP 500

##### Scenario: Unauthorized request stays 401

- GIVEN a request without Bearer `CRON_SECRET`
- WHEN any cron handler runs
- THEN HTTP 401, no alert email

## subscription-lifecycle

### MODIFIED Requirements

#### Requirement: Grace Period

The system MUST provide a 7-day grace period on payment failure. Grace warnings MUST be sent by `send-reminders` for `status='grace' AND grace_expires_at > now`. Grace expiry without payment MUST end the subscription `cancelled` (via `cancelSubscription`), not `expired`.
(Previously: grace expiry → `expired`; `s.grace_end` → 500.)

##### Scenario: Payment failure triggers grace

- GIVEN an active subscription
- WHEN a payment fails via any webhook
- THEN status → grace, `grace_expires_at`=now+7d, access retained

##### Scenario: Grace expires without payment

- GIVEN status=grace and `grace_expires_at <= now`
- WHEN expire-subscriptions runs
- THEN status → cancelled; login blocked

### ADDED Requirements

#### Requirement: Grace Warning Reminder Cron

`send-reminders` MUST query grace subs with `s.grace_expires_at` (real column, `002_multi_gateway.sql:10`), filter `status='grace' AND grace_expires_at > ?`, compute `days_left`, and send `sendGraceWarning` per sub. Trial reminders (`trial_end`) MUST stay unchanged.

##### Scenario: Grace warning sent

- GIVEN status=grace and `grace_expires_at` = now + 3 days
- WHEN send-reminders runs
- THEN `sendGraceWarning` sent with `days_left=3`, response `graceWarnings=1`

##### Scenario: No grace subscriptions

- GIVEN no subs with `status='grace' AND grace_expires_at > now`
- WHEN send-reminders runs
- THEN `graceWarnings=0` and HTTP 200

#### Requirement: Schema-Accurate Test Fixtures

The payments test suite MUST mock only real columns — `plan_id`, `grace_expires_at`; never `plan`, `grace_start`, `grace_end` (`payments.test.ts:196-227`, `subscription-lifecycle.test.ts:76-83`). Cron tests MUST assert real columns in SQL so drift fails CI.

##### Scenario: Stale fixture fails

- GIVEN a fixture mocking `plan`/`grace_end` instead of `plan_id`/`grace_expires_at`
- WHEN the test suite runs
- THEN the test fails (missing column)

##### Scenario: Cron SQL asserts real columns

- GIVEN charge-wompi and send-reminders tests
- WHEN they assert the executed SQL
- THEN `s.plan_id` and `s.grace_expires_at` are present

**Test mapping:** R1/R2 → cron test asserting `s.plan_id` in SQL and price equals `PLANS`; R3 → cron test asserting `s.grace_expires_at`; R4 → lifecycle test asserting grace expiry → `'cancelled'`; R5 → handler throw → 500 + email spy; R6 → updated fixtures in `payments.test.ts:196-227` and `subscription-lifecycle.test.ts:76-83`.