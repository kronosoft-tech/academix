# Superadmin: Control Plane Management

> New specification for superadmin capabilities via the control plane.

## Purpose

The superadmin (master account) SHALL manage all client databases through the control plane. This provides visibility into which academies are registered and their database health, without exposing individual tenant data.

---

## ADDED Requirements

### Requirement: Superadmin Identification

The system SHALL identify the superadmin by a special email/role combination seeded on first run. The superadmin user record exists ONLY in the **control plane** — they do NOT have a separate Turso database.

The superadmin email SHALL be configurable via `ADMIN_EMAIL` environment variable (existing pattern), and their role SHALL be `Admin` (existing).

#### Scenario: Superadmin seeding on first run

- GIVEN the control plane is initialized
- WHEN the app starts for the first time
- THEN a superadmin record is created in the control plane with email from `ADMIN_EMAIL` env var (or default)
- AND the superadmin has role `Admin`

### Requirement: Superadmin Login

The superadmin SHALL log in through the same login form but their authentication SHALL happen against the control plane instead of a Turso database.

**Flow:**
1. User enters email + password
2. System checks control plane first (is this a superadmin?)
3. If yes: authenticate against control plane users table
4. If no: proceed with normal Turso DB resolution flow

#### Scenario: Superadmin login authenticates against control plane

- GIVEN the superadmin email and password
- WHEN the superadmin submits the login form
- THEN the system checks the control plane first
- AND authenticates against the control plane's users table
- AND returns a session token (stored in control plane sessions)

#### Scenario: Regular user login is unaffected

- GIVEN a regular user registered through the normal flow
- WHEN the user submits the login form
- THEN the system does NOT find them in the control plane
- AND proceeds with normal Turso DB resolution
- AND authentication happens against their Turso database

### Requirement: List All Client Databases (Superadmin)

The system SHALL provide a command `list_client_databases` that returns all user→DB mappings from the control plane. This SHALL only be accessible to users with `Admin` role.

#### Scenario: Superadmin lists all databases

- GIVEN the superadmin is authenticated
- WHEN the superadmin calls `list_client_databases`
- THEN the response includes all records from the control plane's `user_databases` table
- AND shows: email, academy_name, db_url, created_at
- AND does NOT expose db_token

#### Scenario: Non-admin tries to list databases

- GIVEN a regular user (role: Empleado) is authenticated
- WHEN the user calls `list_client_databases`
- THEN the command returns "Unauthorized"
- AND no database information is exposed

### Requirement: Control Plane Schema

The control plane SHALL have the following tables:

```sql
-- Standard users table for superadmin auth (same schema as existing)
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Sessions for superadmin
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    token TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- User-to-database mapping for all tenants
CREATE TABLE IF NOT EXISTS user_databases (
    user_id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    academy_name TEXT NOT NULL,
    db_url TEXT NOT NULL,
    db_token TEXT NOT NULL,
    org TEXT NOT NULL DEFAULT 'academix',
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_user_databases_email ON user_databases(email);
```
