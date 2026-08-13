# Delta for User Provisioning

## ADDED Requirements

### Requirement: R1: Registration Form Contract

The system MUST accept `name`, `email`, `password`, `academyName`, and `confirmPassword` in the web registration form.

The system MUST validate `academyName` with `z.string().trim().min(2)`; whitespace-only values MUST be rejected.

The system MUST validate that `confirmPassword` equals `password`; a mismatch MUST be rejected with an error on `confirmPassword`.

The system MUST NOT accept a client-supplied slug field; the database slug MUST be derived server-side via `generateDbSlug(academyName)`.

#### Scenario: Valid registration with academy name

- GIVEN a registration form with `academyName` of 2+ non-whitespace characters and matching passwords
- WHEN the form is submitted
- THEN validation passes
- AND the DB slug is derived server-side from `academyName`

#### Scenario: Whitespace-only academy name

- GIVEN a registration form with `academyName` containing only whitespace
- WHEN the form is submitted
- THEN the action rejects with a validation error on `academyName`

#### Scenario: Password confirmation mismatch

- GIVEN a registration form where `confirmPassword` differs from `password`
- WHEN the form is submitted
- THEN the action rejects with an error on `confirmPassword`

### Requirement: R2: Per-User Database Provisioning

The system MUST provide `generateDbSlug(academyName)` producing `academy-{slug}-{4-char-hex}` where `{slug}` is the lowercased, hyphenated, 30-char-capped academy name.

The system MUST provide `createDatabase(org, name, group)` calling `POST /v1/organizations/{org}/databases` with the Turso API token; on a 409 (slug collision) it MUST retry up to 3 times, generating a fresh 4-char hex suffix each attempt.

The system MUST provide `createAuthToken(org, dbName)` calling `POST /v1/organizations/{org}/databases/{dbName}/auth/tokens` with `{ permission: "full-access" }`, passing the bare database name (not the `libsql://` URL).

The system MUST set `dbUrl = libsql://{hostname}` from the database creation response.

#### Scenario: First-attempt provisioning

- GIVEN an available `{academy-slug}` database name and Turso env vars configured
- WHEN registration provisions the user
- THEN `createDatabase` succeeds on the first call
- AND a `full-access` auth token is created for the bare DB name
- AND `dbUrl` is set to `libsql://{hostname}`

#### Scenario: Slug collision retries with fresh suffix

- GIVEN `createDatabase` returns 409 for the initial slug
- WHEN registration retries
- THEN a fresh 4-char hex suffix is generated per attempt, up to 3 retries
- AND provisioning succeeds if any retry returns non-409

#### Scenario: Provisioning exhausted

- GIVEN `createDatabase` returns 409 on the initial attempt and all 3 retries
- WHEN provisioning cannot create a database
- THEN registration fails with a clear error and no account is written

### Requirement: R3: Per-User Migration Runner

The system MUST provide `runMigrationsOnDb(client)` that runs the copies of desktop migrations 001–020 from `web/migrations/per-user/` against the per-user database.

The system MUST track applied migrations in a `_schema_migrations` table; re-running on an already-migrated database MUST be a no-op (idempotent).

The runner MUST mirror the desktop `run_migrations_on_db` behavior (per-file batched execution).

#### Scenario: Fresh database is migrated

- GIVEN a newly created per-user database
- WHEN `runMigrationsOnDb` runs
- THEN all migration files 001–020 execute
- AND `_schema_migrations` records each applied migration

#### Scenario: Migration rerun is idempotent

- GIVEN a per-user database where all migrations already applied
- WHEN `runMigrationsOnDb` runs again
- THEN no migration re-executes
- AND no error is raised

### Requirement: R4: Registration Ordering and Writes

The register action MUST execute in this order: validate input → check email existence → hash password → provision DB + run migrations → insert per-user `users` row (role `'Admin'`) → insert shared `users` row → upsert `user_databases` (INSERT OR REPLACE, including `academy_name`) → `createTrialSubscription` → `signToken` with `dbUrl`, `dbToken`, and `academyName`.

The system MUST sign the customer JWT with the existing `dbUrl`/`dbToken`/`academyName` claims (resolving the current TS2345).

The system MUST reject registration with `CONFLICT` if the email already exists, before any provisioning.

#### Scenario: Full happy-path registration

- GIVEN a new email and all Turso env vars configured
- WHEN the register action runs
- THEN provisioning completes, both per-user and shared rows exist, `user_databases` holds the DB mapping with `academy_name`, a shared trial row is created, and the JWT contains `dbUrl`, `dbToken`, and `academyName`

#### Scenario: Duplicate email short-circuits

- GIVEN a registration attempt with an existing email
- WHEN the register action runs
- THEN the action returns `CONFLICT`
- AND no database is provisioned and no rows are written

### Requirement: R5: Failure Handling and Cleanup

If any step after database creation fails, the system MUST attempt a best-effort `DELETE /v1/organizations/{org}/databases/{name}` for the created database.

The system MUST fail closed on registration errors: the user MUST NOT receive a signed JWT or be treated as registered.

The user MUST receive a clear error message describing the failure.

#### Scenario: Failure after database creation

- GIVEN a database was created but a later step (e.g., migration or user insert) fails
- WHEN registration catches the error
- THEN a best-effort API DELETE is attempted for the created database
- AND registration returns a clear error and no JWT is signed

#### Scenario: Failure before database creation

- GIVEN a failure occurs before DB creation (e.g., invalid input or env missing)
- WHEN registration catches the error
- THEN no cleanup is needed
- AND registration returns a clear error and no JWT is signed

### Requirement: R6: Environment Gating

The system MUST require `TURSO_API_TOKEN`, `TURSO_ORG`, and `TURSO_GROUP` for registration provisioning.

If any is missing, registration MUST fail closed with an actionable error — provisioning MUST NOT silently degrade to shared-DB-only registration.

#### Scenario: Missing Turso env vars

- GIVEN `TURSO_API_TOKEN`, `TURSO_ORG`, or `TURSO_GROUP` is unset
- WHEN a user registers
- THEN registration is rejected with a clear, actionable error
- AND no database is provisioned and no account is created

#### Scenario: All env vars present

- GIVEN all three Turso env vars are set
- WHEN a user registers
- THEN provisioning proceeds normally

### Requirement: R7: Payment Lifecycle on Shared Database

This change MUST NOT write subscription or payment data to per-user databases.

All subscription/payment lifecycle flows (trial creation, webhooks, checkout, crons) MUST continue operating against the shared database exactly as before.

#### Scenario: Trial row lands in shared DB only

- GIVEN a web-registered user completes registration
- WHEN the trial subscription is created
- THEN the trial row is written to the shared database
- AND no subscription rows are written to the per-user database

#### Scenario: Existing payment flows unchanged

- GIVEN an active payment lifecycle (webhook/checkout/cron)
- WHEN the flow operates on a web-registered user
- THEN behavior is identical to pre-change behavior against the shared database

### Requirement: R8: Baseline Protection

The change MUST NOT increase pre-existing failures: failing tests MUST remain at 7 or fewer, and `tsc --noEmit` errors MUST remain at 7 or fewer after the register action type fix.

#### Scenario: Regression budget holds

- GIVEN the change is applied
- WHEN the test suite and typecheck run
- THEN failing tests ≤ 7 and tsc errors ≤ 7

## Non-Goals

Out of scope for this change:

- Backfill of existing web-registered users (separate follow-up change; they remain broken until it lands)
- Desktop-side changes and `020_web_schema.sql` alignment (`plan_id` vs `plan`, grace columns, provider CHECKs)
- Stripe payment integration
- `send-reminders` cron `grace_end` bug (pre-existing, tracked separately)
