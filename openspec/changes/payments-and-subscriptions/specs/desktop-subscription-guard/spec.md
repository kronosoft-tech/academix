# Desktop Subscription Guard Specification

## Purpose

Enforce subscription status on the Tauri desktop app login, with 24-hour offline cached grace.

## Requirements

### Requirement: Post-Login Subscription Check

The system MUST verify subscription status against the control plane after successful authentication in the desktop app.

#### Scenario: Active subscription allows access

- GIVEN the desktop user authenticates successfully
- WHEN auth.rs queries the control plane for subscription status
- THEN status=active or trialing is returned
- AND the user proceeds to the main application

#### Scenario: Expired subscription blocks access

- GIVEN the desktop user authenticates successfully
- WHEN the control plane returns status=expired or cancelled
- THEN the desktop app displays a blocking message with reactivation link
- AND the user cannot proceed past the login screen

### Requirement: Offline Cached Grace

The system MUST cache the last known subscription status locally. If the control plane is unreachable, the cached status is valid for 24 hours.

#### Scenario: Offline with valid cache

- GIVEN the desktop app cannot reach the control plane
- AND the cached subscription check is less than 24 hours old with status=active
- WHEN the user logs in
- THEN access is granted using the cached status

#### Scenario: Offline with expired cache

- GIVEN the desktop app cannot reach the control plane
- AND the cached subscription check is more than 24 hours old
- WHEN the user logs in
- THEN access is denied with message indicating network required for verification

#### Scenario: Cache invalidation on successful check

- GIVEN a successful control plane response
- WHEN subscription status is verified
- THEN the local cache is updated with the new status and timestamp
