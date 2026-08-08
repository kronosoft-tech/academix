# Pricing and Checkout Specification

## Purpose

Display pricing plans with country-based gateway auto-detection and route users to the correct payment flow.

## Requirements

### Requirement: Gateway Auto-Detection

The system MUST auto-detect the user's country and select the appropriate payment gateway. Manual override MUST be available.

#### Scenario: Colombian user sees Wompi

- GIVEN a user accessing the pricing page from Colombia (geo-IP)
- WHEN the page loads
- THEN Wompi is selected as the default gateway
- AND checkout buttons reflect Wompi flow

#### Scenario: Other LatAm user sees Mercado Pago

- GIVEN a user from Argentina, Mexico, Chile, or other supported LatAm countries
- WHEN the page loads
- THEN Mercado Pago is selected as the default gateway

#### Scenario: Non-LatAm user sees Stripe

- GIVEN a user from US, EU, or non-LatAm region
- WHEN the page loads
- THEN Stripe is selected as the default gateway

#### Scenario: Manual gateway override

- GIVEN any user on the pricing page
- WHEN they manually select a different gateway
- THEN checkout buttons update to reflect the chosen gateway

### Requirement: Plan Display

The system MUST display three plans: Basico ($49,900), Pro ($89,900), Premium ($149,900) COP/month.

#### Scenario: Plans rendered correctly

- GIVEN the pricing page is loaded
- WHEN plans data is fetched
- THEN all three plans display with name, price, and feature list

### Requirement: Checkout Button Behavior

The system MUST route checkout to the gateway-specific endpoint based on selected plan and gateway.

#### Scenario: Authenticated user clicks subscribe

- GIVEN an authenticated user with no active subscription
- WHEN they click "Subscribe" on a plan
- THEN a POST is made to /api/checkout/[gateway] with plan_id
- AND user is redirected to the gateway's payment page

#### Scenario: Unauthenticated user clicks subscribe

- GIVEN a visitor not logged in
- WHEN they click "Subscribe"
- THEN they are redirected to the login/register page with return URL preserved
