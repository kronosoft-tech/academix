# Academix Web — Specification

## Purpose

Public-facing web presence for Academix: marketing, downloads, authentication (shared with desktop), customer/admin dashboards, payments, and support.

---

## 1. web-landing Specification

### Requirement: Marketing Pages

The system MUST serve static prerendered marketing pages including hero, features showcase, FAQ, testimonials, video embed, and contact form.

#### Scenario: Visitor views landing page

- GIVEN a visitor navigates to the root URL
- WHEN the page loads
- THEN the hero section, features, FAQ, testimonials, and video embed are displayed
- AND the page is fully rendered without JavaScript (prerendered)

#### Scenario: Contact form submission

- GIVEN a visitor fills out the contact form with valid name, email, and message
- WHEN the visitor submits the form
- THEN the system sends the message and displays a success confirmation

#### Scenario: Contact form validation failure

- GIVEN a visitor submits the contact form with an empty email field
- WHEN the form is submitted
- THEN the system displays a validation error without sending

---

## 2. web-downloads Specification

### Requirement: OS-Aware Download Portal

The system MUST detect the visitor's OS, display the matching installer, and provide architecture selection (x64/arm64). Download links MUST route through a custom redirect endpoint that increments a counter before forwarding to the GitHub Release asset URL.

#### Scenario: Auto-detected OS download

- GIVEN a visitor on Windows accesses the downloads page
- WHEN the page loads
- THEN the Windows installer is highlighted as the primary download
- AND alternative OS options are visible

#### Scenario: Download tracking

- GIVEN a visitor clicks a download link
- WHEN the redirect endpoint processes the request
- THEN the download counter increments by 1
- AND the visitor is forwarded to the GitHub Release asset URL

#### Scenario: Unknown OS fallback

- GIVEN a visitor on an unrecognized OS accesses the downloads page
- WHEN the page loads
- THEN all OS options are displayed equally without a primary highlight

---

## 3. web-auth Specification

### Requirement: Custom JWT Authentication

The system MUST implement JWT-based authentication with bcrypt password hashing, httpOnly cookies, and shared Turso user table. Registration MUST be bidirectional (web or desktop, same credentials).

#### Scenario: User registration

- GIVEN a visitor provides a valid email and password (min 8 chars)
- WHEN registration is submitted
- THEN a user record is created in Turso with bcrypt-hashed password
- AND a JWT is issued in an httpOnly secure cookie

#### Scenario: Login with valid credentials

- GIVEN a registered user provides correct email and password
- WHEN login is submitted
- THEN a JWT is issued in an httpOnly secure cookie
- AND the user is redirected to their dashboard

#### Scenario: Login with invalid credentials

- GIVEN a user provides an incorrect password
- WHEN login is submitted
- THEN the system returns "Invalid credentials" without revealing which field is wrong

#### Scenario: Password reset flow

- GIVEN a user requests password reset with a registered email
- WHEN the request is processed
- THEN a time-limited reset token is generated and sent via email

#### Scenario: Middleware blocks unauthenticated access

- GIVEN an unauthenticated request targets a protected route
- WHEN the middleware validates the JWT cookie
- THEN the system redirects to the login page

---

## 4. web-admin-dashboard Specification

### Requirement: Internal Metrics Dashboard

The system MUST display real-time metrics: total clients, revenue, download counts, active payments, and churn rate. Access MUST be restricted to admin-role users.

#### Scenario: Admin views dashboard

- GIVEN an authenticated admin user accesses the admin dashboard
- WHEN the page loads
- THEN client count, revenue totals, download counts, payment stats, and churn rate are displayed

#### Scenario: Non-admin access denied

- GIVEN an authenticated user without admin role accesses the admin dashboard URL
- WHEN the middleware checks the role
- THEN the system returns 403 Forbidden

---

## 5. web-customer-dashboard Specification

### Requirement: Subscription Management Portal

The system MUST allow customers to view subscription status, payment history, invoices, and expiration alerts. Customers SHOULD be able to cancel their subscription.

#### Scenario: Customer views subscription

- GIVEN an authenticated customer with an active subscription
- WHEN they access the customer dashboard
- THEN current plan, next billing date, and payment history are displayed

#### Scenario: Expiration alert display

- GIVEN a customer whose subscription expires within 7 days
- WHEN they access the dashboard
- THEN a prominent expiration warning is displayed

#### Scenario: Customer without subscription

- GIVEN an authenticated user with no active subscription
- WHEN they access the customer dashboard
- THEN the system displays a prompt to subscribe with a link to pricing

---

## 6. web-payments Specification

### Requirement: Webhook-Based Subscription Lifecycle

The system MUST process subscription payments via Stripe webhooks (MVP), with future support for MercadoPago and PayU. Subscriptions MUST include a 7-day free trial and a 7-day grace period after payment failure before desktop access deactivation.

#### Scenario: New subscription with trial

- GIVEN a user selects the Basic plan and completes Stripe checkout
- WHEN the subscription is created
- THEN a 7-day free trial period begins
- AND the user gains immediate desktop app access

#### Scenario: Trial expires with payment success

- GIVEN a user's trial period ends
- WHEN Stripe charges the first payment successfully
- THEN the subscription transitions to active status

#### Scenario: Payment failure grace period

- GIVEN an active subscription's payment fails
- WHEN the webhook processes the failure event
- THEN the subscription enters a 7-day grace period
- AND the user retains desktop access during grace

#### Scenario: Grace period expires without payment

- GIVEN a subscription in grace period for 7 days with no successful payment
- WHEN the grace period expires
- THEN desktop app access is deactivated
- AND the customer dashboard shows "Subscription Expired"

#### Scenario: Webhook signature validation

- GIVEN an incoming webhook request with an invalid signature
- WHEN the endpoint processes the request
- THEN the system rejects it with 401 and logs the attempt

---

## 7. web-support Specification

### Requirement: PQRS Ticket System

The system MUST provide a ticket submission form with states: open → in-progress → resolved. Customers MUST view their ticket history and status updates.

#### Scenario: Customer creates ticket

- GIVEN an authenticated customer fills out the PQRS form (type, subject, description)
- WHEN the form is submitted
- THEN a ticket is created with status "open"
- AND the customer sees a confirmation with ticket ID

#### Scenario: Customer views ticket status

- GIVEN a customer has submitted tickets previously
- WHEN they access the support portal
- THEN all their tickets are listed with current status

### Requirement: AI Technical Chat

The system MUST provide an AI-powered chat using Groq and Cerebras free tiers in round-robin rotation. The architecture MUST be extensible for adding more providers.

#### Scenario: User sends chat message

- GIVEN an authenticated user types a question in the support chat
- WHEN the message is sent
- THEN the system routes to the next provider in the round-robin rotation
- AND returns the AI response within 10 seconds

#### Scenario: Provider failure fallback

- GIVEN the current provider in rotation returns an error or times out
- WHEN the system detects the failure
- THEN it retries with the next provider in rotation

#### Scenario: All providers unavailable

- GIVEN all configured providers are failing
- WHEN a user sends a message
- THEN the system displays "Support chat temporarily unavailable. Please create a ticket."

---

## 8. web-pricing Specification

### Requirement: Localized Pricing Display

The system MUST display pricing localized by country (CO, MX, CL, AR) via server islands. Country detection MUST use request geolocation headers. Exchange rates MUST be loaded from a static JSON config. MVP offers Basic plan only ($20 USD/mo equivalent).

#### Scenario: Country-specific pricing displayed

- GIVEN a visitor from Colombia accesses the pricing page
- WHEN the server island renders
- THEN the Basic plan price is shown in COP with the locally equivalent amount

#### Scenario: Unsupported country fallback

- GIVEN a visitor from a country not in the supported list (CO, MX, CL, AR)
- WHEN the pricing page loads
- THEN the price is displayed in USD ($20/mo)

#### Scenario: Country array config structure

- GIVEN the static pricing JSON config
- WHEN the server island reads it
- THEN each entry MUST contain: country_code, currency_code, currency_symbol, monthly_price, and display_name
