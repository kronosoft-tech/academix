# Authentication: Turso Database Resolution

> Delta spec for the login flow with dynamic database resolution.
> This MODIFIES the existing login behavior — login now resolves the user's Turso DB.

## Purpose

When a user logs in, the system SHALL resolve their specific Turso database from the control plane and authenticate against that database instead of a single shared database.

---

## MODIFIED Requirements

### Requirement: Login with Database Resolution

The login flow SHALL now resolve the user's database before authentication:

1. Receive email + password
2. Look up email in the **control plane** (`user_databases` table)
3. If not found: fail with "Invalid credentials" (don't reveal that the user doesn't exist)
4. Get or create a `libsql` connection to the user's Turso DB (via ConnectionManager)
5. Query the `users` table in the **user's Turso DB** for the email
6. Verify password against the stored hash
7. Create session in the **user's Turso DB** `sessions` table
8. Return LoginResponse

#### Scenario: Login resolves Turso DB successfully

- GIVEN user A is registered with email "teacher@example.com"
- AND user A's Turso DB mapping exists in the control plane
- WHEN the user submits login with email "teacher@example.com" and correct password
- THEN the system looks up the email in the control plane
- AND establishes a connection to user A's Turso DB
- AND verifies the password against user A's DB
- AND creates a session in user A's DB
- AND returns a session token

#### Scenario: Login with email not in control plane

- GIVEN an email "unknown@example.com" has no control plane mapping
- WHEN the user attempts to log in with this email
- THEN the system returns "Invalid credentials"
- AND no database connection attempt is made

#### Scenario: Login with correct email but user's Turso DB is unreachable

- GIVEN user's Turso database is temporarily down
- WHEN the user submits valid credentials
- THEN the login fails with "Service unavailable. Please try again later."
- AND no session is created

### Requirement: Token Validation with Database Resolution

Token validation (`validate_token`) SHALL now:
1. Parse the session token
2. Extract the user_id (or look up by token)
3. Resolve the user's Turso DB from control plane
4. Validate the session in the user's DB

#### Scenario: Validate token across user's DB

- GIVEN user A has a valid session token
- WHEN the system validates the token
- THEN it resolves user A's Turso DB from control plane
- AND queries the sessions table in that DB
- AND returns the user if valid

### ADDED Requirement: Connection Caching

The system SHOULD cache database connections in the `ConnectionManager` for the duration of the Tauri process. Connections are lazily created on first access and reused for subsequent requests.

#### Scenario: Connection is cached after first login

- GIVEN user A logged in successfully (connection created and cached)
- WHEN user A makes another authenticated request
- THEN the cached connection is reused
- AND no new database resolution is needed

#### Scenario: Connection is created on first request for a new user

- GIVEN user A logged in previously
- WHEN a brand new user B logs in for the first time
- THEN a NEW connection to user B's Turso DB is created
- AND user A's cached connection is unaffected
