# Registration: Turso Database Provisioning

> Delta spec for the registration flow with Turso database creation.
> This ADDS to the existing registration behavior — existing registration scenarios still apply.

## Purpose

When a user registers, the system SHALL create an isolated Turso database for their academy before completing the registration. This enables tenant isolation without RLS.

---

## ADDED Requirements

### Requirement: Academy Name Field

The registration form MUST include a required "Academy Name" field (`academy_name`). The system SHALL use this name to derive the Turso database slug.

**Derivation rules:**
- Convert to lowercase
- Replace spaces and special characters with hyphens
- Remove consecutive hyphens
- Limit to 40 characters
- Append a short random suffix for uniqueness (e.g., `academy-music-school-a3f2`)
- MUST be unique (Turso requires unique database names per organization)

#### Scenario: Register with academy name

- GIVEN the user is on the registration page
- WHEN the user fills all fields including `academy_name = "Music School"`
- AND submits the form
- THEN a Turso database is created with name derived from "Music School"
- AND the user account is created inside that database
- AND the registration succeeds

#### Scenario: Registration with duplicate display name (resolved by random suffix)

- GIVEN user A registered with academy "Music School" and got slug `academy-music-school-a3f2`
- WHEN user B registers with academy "Music School"
- THEN a different database is created (unique slug: `academy-music-school-b7k1`)
- AND both users have isolated databases

### Requirement: Turso Database Creation on Registration

The system SHALL call the Turso Platform API during registration to create a new database. If the API call fails, the registration MUST fail and the user MUST NOT be created.

**Flow:**
1. Validate email + password (existing)
2. Check email does not exist (control plane)
3. Derive database slug from academy name
4. Call `POST /v1/organizations/{org}/databases` with the slug
5. On success: generate an auth token for the new DB
6. Connect to the new DB and run all migrations
7. Store user record in the new DB
8. Save mapping to control plane (user_id, email, academy_name, db_url, db_token)
9. Return success

#### Scenario: Registration creates Turso database successfully

- GIVEN valid registration data with `academy_name = "Math Academy"`
- WHEN the registration is submitted
- THEN the Turso Platform API is called to create `academy-math-academy-{suffix}`
- AND all 18 migrations run against the new database
- AND a mapping is saved in the control plane
- AND the response returns `success: true`

#### Scenario: Turso API fails during registration

- GIVEN the Turso Platform API is unavailable
- WHEN a user submits registration
- THEN the registration fails with a "Service unavailable, try again later" error
- AND no user record is created
- AND no database is left in a partial state

### Requirement: Registration Loading State

During the Turso database creation (2-5 seconds), the system SHALL show a loading indicator on the registration button with "Creando tu academia..." text. The button MUST be disabled during this time to prevent double submission.

#### Scenario: Loading state shown during registration

- GIVEN the user has filled all fields and submitted
- WHEN the Turso API call is in progress
- THEN the submit button shows a spinner with "Creando tu academia..."
- AND the button is disabled

#### Scenario: Double-submission prevention

- GIVEN the user clicked submit
- WHEN the registration is in progress
- THEN clicking submit again does nothing
- AND only one Turso API call is made

### Requirement: Academy Name Validation

The academy name field SHALL have validation rules:
- Required (non-empty)
- Minimum 3 characters
- Maximum 100 characters
- Only letters, numbers, spaces, hyphens, and apostrophes allowed
- Error message shown inline below the field

#### Scenario: Empty academy name

- GIVEN the user leaves the academy name field empty
- WHEN the form is submitted
- THEN an inline error "El nombre de la academia es requerido" is shown
- AND the form is not submitted

#### Scenario: Academy name with invalid characters

- GIVEN the user enters `@cad#my!` as academy name
- WHEN the form is submitted
- THEN an inline error "Solo se permiten letras, números y espacios" is shown
- AND the form is not submitted
