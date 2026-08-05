# User Subscription Management Specification

## Purpose

View-only subscription and billing pages. Users can see their current plan, subscription status, and payment history from the control-plane database.

## Requirements

### Requirement: Control-Plane Migration

The system MUST create `subscriptions` and `subscription_payments` tables on the control-plane database via an idempotent migration (IF NOT EXISTS).

The `subscriptions` table MUST store: user reference, plan name, status, billing period start/end, trial end, and currency.

The `subscription_payments` table MUST store: subscription reference, amount, currency, payment date, payment method, and status.

#### Scenario: Migration runs on fresh database

- GIVEN a control-plane database without subscription tables
- WHEN the migration executes
- THEN `subscriptions` and `subscription_payments` tables are created

#### Scenario: Migration is idempotent

- GIVEN a control-plane database where subscription tables already exist
- WHEN the migration executes again
- THEN no error occurs and existing data is preserved

### Requirement: Subscription Page

The system MUST render `/dashboard/subscription` showing the user's current plan name, status, billing period end, and plan limits.

The system MUST display an upgrade CTA for users not on the highest plan.

The system MUST NOT perform any subscription mutations from the web app.

#### Scenario: Active subscription display

- GIVEN an authenticated user with an active "Básico" subscription
- WHEN they navigate to `/dashboard/subscription`
- THEN the page shows plan name "Básico", status "active", period end date, and plan limits
- AND an upgrade CTA is displayed

#### Scenario: Trial subscription display

- GIVEN an authenticated user with a trial subscription (status: "trial")
- WHEN they navigate to `/dashboard/subscription`
- THEN the page shows trial status, trial end date, and a CTA to choose a plan

#### Scenario: Highest plan — no upgrade CTA

- GIVEN an authenticated user on the "Premium" plan
- WHEN they navigate to `/dashboard/subscription`
- THEN no upgrade CTA is displayed

### Requirement: Payments History Page

The system MUST render `/dashboard/payments` showing a paginated list of subscription payments from the control-plane database.

Each payment entry MUST display: date, amount with currency, payment method, and status.

#### Scenario: User views payment history

- GIVEN an authenticated user with 3 subscription payments
- WHEN they navigate to `/dashboard/payments`
- THEN a table displays all 3 payments with date, amount, method, and status

#### Scenario: User with no payment history

- GIVEN an authenticated user with no subscription payments
- WHEN they navigate to `/dashboard/payments`
- THEN an empty state message is shown (e.g., "No payments yet")

### Requirement: Pricing Plans Definition

The system MUST define pricing plans as a shared constant:
- Básico: $49,900 COP/month
- Pro: $89,900 COP/month
- Premium: $149,900 COP/month

Each plan MUST specify display limits (students, courses, groups). Plan limits are display-only; enforcement is deferred to the desktop app.

#### Scenario: Plans data is accessible across pages

- GIVEN the pricing plans definition exists in `src/data/plans.ts`
- WHEN the subscription or pricing page renders
- THEN plan names, prices, and limits are displayed correctly from the shared constant
