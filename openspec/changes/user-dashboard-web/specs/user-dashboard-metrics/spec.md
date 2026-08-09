# User Dashboard Metrics Specification

## Purpose

Read-only academy metrics page that queries the user's individual Turso database to display student, course, group counts, income, expenses, and recent payments.

## Requirements

### Requirement: Academy Metrics Display

The system MUST display the following metrics from the user's individual Turso DB: total students, total courses, total groups, total income, total expenses, and recent payments.

The system MUST use `getFullTokenPayload(cookies)` to obtain `dbUrl`/`dbToken` from the JWT and `getUserDb(payload)` to connect to the user's database.

The system MUST NOT perform any write operations on the user's database.

#### Scenario: Authenticated user views dashboard with data

- GIVEN an authenticated user with an academy containing students, courses, groups, and payments
- WHEN the user navigates to `/dashboard`
- THEN the page displays total students, total courses, total groups, total income, total expenses
- AND a recent payments list is shown

#### Scenario: Authenticated user with empty academy

- GIVEN an authenticated user whose academy database has no data
- WHEN the user navigates to `/dashboard`
- THEN all metric counts display zero
- AND an empty-state CTA prompts the user to download the desktop app

#### Scenario: Unauthenticated access attempt

- GIVEN an unauthenticated visitor
- WHEN they attempt to access `/dashboard`
- THEN the system redirects to the login page

### Requirement: Auth Helper for Full Token Payload

The system MUST provide a `getFullTokenPayload(cookies)` helper that verifies the JWT token and returns the full `CustomerJwtPayload` including `dbUrl` and `dbToken` claims.

The helper MUST return `null` if the token is missing, expired, or invalid.

#### Scenario: Valid token extraction

- GIVEN a request with a valid JWT cookie containing dbUrl and dbToken claims
- WHEN `getFullTokenPayload(cookies)` is called
- THEN it returns the full `CustomerJwtPayload` with all claims

#### Scenario: Missing or invalid token

- GIVEN a request with no JWT cookie or an expired/invalid token
- WHEN `getFullTokenPayload(cookies)` is called
- THEN it returns `null`

### Requirement: Dashboard Charts Integration

The system SHOULD render charts (income vs expenses, attendance trends) using the existing `DashboardCharts.tsx` component wired to real data from the user's database.

#### Scenario: Charts render with available data

- GIVEN an authenticated user with payment and attendance records
- WHEN the dashboard loads
- THEN charts display income vs expenses and attendance data

#### Scenario: Charts with no data

- GIVEN an authenticated user with an empty database
- WHEN the dashboard loads
- THEN charts render in an empty/placeholder state without errors
