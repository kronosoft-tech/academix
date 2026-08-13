```yaml
schema: gentle-ai.verify-result/v1
evidence_revision: sha256:d0f3b12ffb53266c2f9eae6aecd3d5a2a37498b6cfdda79cce3f4145413a64bd
verdict: fail
blockers: 0
critical_findings: 0
requirements: 8/8
scenarios: 17/17
test_command: bun run test
test_exit_code: 1
test_output_hash: sha256:2f318c390fe65936414e3290b81d6610fbceeabdf953bf34bc81a42dca23ccc4
build_command: bunx tsc --noEmit
build_exit_code: 2
build_output_hash: sha256:ad250f8b4f0cdb0c3c3e222412b9f816d84bdaba2ed9cd92b500f9afaea187a5
```

# Verification Report: web-registration-provisioning

**Change**: web-registration-provisioning
**Version**: delta spec `specs/user-provisioning/spec.md` (R1–R8, 17 scenarios)
**Mode**: Standard (Strict TDD not active)

## Completeness

| Metric | Value |
|--------|-------|
| Tasks total | 17 |
| Tasks complete | 17 |
| Tasks incomplete | 0 |

All tasks 1.1–4.3 checked in `tasks.md`/`apply-progress.md`; commits `8e0e89b 189ec5f 38f0c9f b66e7d5 f21285c 6d209bd`.

## Build & Tests Execution

**Typecheck** (`bunx tsc --noEmit`, web/): exit 2 — exactly the **7 pre-existing errors** (≤ 7 per R8), zero new. The former register.ts TS2345 is gone. Errors: `src/lib/db.ts(25,20) TS2352`, `src/test/payments.test.ts` TS2307 ×5 (missing `lib/payments/stripe` module), `src/test/payments/checkout-integration.test.ts(61,38) TS2367`.

**Tests** (`bun run test`, web/): **7 failed / 156 passed** (7 ≤ 7 per R8). All 7 failures are the pre-existing Stripe-era set, untouched by this change:
- `payments.test.ts` Stripe Webhook Signature ×5 (imports non-existent `lib/payments/stripe`)
- `checkout-integration.test.ts` ×2 (`geoToGateway('US')` expects `'stripe'`; gateway.ts intentionally routes to wompi/mercadopago only)

**Focused suites** (this change's tests): `bun run test -- src/test/provisioning.test.ts` → **38 passed (38)**, exit 0; `bun run test -- src/test/register-action.test.ts` → **12 passed (12)**, exit 0.

**Coverage**: ➖ Not available (no coverage threshold configured in repo).

**E2E**: not re-run during verify (creates real Turso DBs + shared rows per run). apply-progress evidence (work-unit table): `bun run test:e2e` → **9 passed / 8 skipped / 0 failed**, ran green twice against the real Turso platform; provisioning spec asserts register → dashboard renders "Panel de Control" with zero "Error de conexión" blocks (JWT `dbUrl`/`dbToken` claims connect).

## Spec Compliance Matrix

| Requirement | Scenario | Test | Result |
|-------------|----------|------|--------|
| R1 | Valid registration with academy name | `register-action.test.ts > accepts a valid payload and trims academyName server-side`; `provisioning.test.ts > generateDbSlug (8 tests)` | ✅ COMPLIANT |
| R1 | Whitespace-only academy name | `register-action.test.ts > rejects a whitespace-only academyName` | ✅ COMPLIANT |
| R1 | Password confirmation mismatch | `register-action.test.ts > rejects a password confirmation mismatch on confirmPassword` | ✅ COMPLIANT |
| R2 | First-attempt provisioning | `provisioning.test.ts > creates a database on first attempt`; `creates a full-access token using the bare database name (design D5)`; `provisions a database end-to-end with a real libsql migration run` (dbUrl `libsql://{hostname}`) | ✅ COMPLIANT |
| R2 | Slug collision retries with fresh suffix | `provisioning.test.ts > retries on 409 with a fresh suffix, up to 3 retries`; `provisionUser > retries createDatabase on 409 with a fresh suffix` | ✅ COMPLIANT |
| R2 | Provisioning exhausted | `provisioning.test.ts > throws CONFLICT when all 4 attempts conflict`; `provisionUser > throws CONFLICT when all 4 create attempts conflict` (no token call) | ✅ COMPLIANT |
| R3 | Fresh database is migrated | `provisioning.test.ts > applies all 20 real migrations to a fresh database` (count=20, tables from 001/020 spot-checked) | ✅ COMPLIANT |
| R3 | Migration rerun is idempotent | `provisioning.test.ts > is a no-op on rerun (idempotent)`; `aborts with MIGRATION when a file fails, keeping prior versions recorded` | ✅ COMPLIANT |
| R4 | Full happy-path registration | `register-action.test.ts > provisions, writes all rows in order, signs the JWT and sets the cookie` (exact 9-step order log; claims dbUrl/dbToken/academyName; user_databases args incl. academy_name; trial on shared DB) | ✅ COMPLIANT |
| R4 | Duplicate email short-circuits | `register-action.test.ts > returns CONFLICT for a duplicate email without provisioning anything` (provisionUser not called; no rows; no JWT) | ✅ COMPLIANT |
| R5 | Failure after database creation | `provisioning.test.ts > deletes the created database best-effort when createAuthToken fails`; `...when migrations fail`; `surfaces the original error even when cleanup DELETE fails`; `register-action.test.ts > DELETEs the provisioned DB when a later shared write fails`; `...when signToken fails`; `never returns a JWT on any failure` | ✅ COMPLIANT |
| R5 | Failure before database creation | `provisioning.test.ts > fails closed with MISSING_ENV before any API call`; `register-action.test.ts > fails closed when Turso env vars are missing` (no DELETE, no JWT) | ✅ COMPLIANT |
| R6 | Missing Turso env vars | `provisioning.test.ts > getTursoEnv (4 tests: API_TOKEN/ORG/GROUP/all missing)`; `register-action.test.ts > fails closed when Turso env vars are missing` | ✅ COMPLIANT |
| R6 | All env vars present | `provisioning.test.ts > returns the env values when all vars are present`; provisionUser E2E (real env) per apply-progress | ✅ COMPLIANT |
| R7 | Trial row lands in shared DB only | `register-action.test.ts > happy path`: `createTrialSubscription(id,'trial',null)` on shared `db`; per-user client receives zero `subscriptions` writes (asserted) | ✅ COMPLIANT |
| R7 | Existing payment flows unchanged | Static: the 6 change commits touch only register/provisioning/form/tests/migrations/docs — no webhook/checkout/cron module modified; shared DB remains payment source of truth | ✅ COMPLIANT |
| R8 | Regression budget holds | `bun run test` → 7 failed ≤ 7; `bunx tsc --noEmit` → 7 errors ≤ 7; both exactly the pre-existing set | ✅ COMPLIANT |

**Compliance summary**: 17/17 scenarios compliant, 8/8 requirements.

## Correctness (Static Evidence)

| Requirement | Status | Notes |
|------------|--------|-------|
| R1 Form contract | ✅ Implemented | `web/src/actions/register.ts:20-34` — `academyName: z.string().trim().min(2)`, `.refine` on `confirmPassword` path, no slug field (unknown keys stripped by safeParse); `register.astro:81-112` renders both fields with per-field errors |
| R2 Provisioning | ✅ Implemented | `web/src/lib/provisioning.ts:66-86` slug (desktop parity vs `provisioning.rs:54-79` confirmed); `131-169` createDatabase POST + 409 retry ≤3 with fresh 4-hex suffix (MAX_CREATE_ATTEMPTS=4); `179-204` createAuthToken bare name + `{permission:"full-access"}`; `307` `dbUrl = libsql://{hostname}` |
| R3 Migration runner | ✅ Implemented | `provisioning.ts:235-287` — `_schema_migrations(version PK, applied_at)`; sorted `*.sql` read; skip-if-recorded (idempotent); per-file `client.executeMultiple(sql)` then record (desktop `execute_batch` parity, D4); 20 files byte-identical to `src-tauri/migrations/` (diff verified) |
| R4 Ordering/writes | ✅ Implemented | `register.ts:57-127` — email-exists CONFLICT → hash → provisionUser → per-user `users` 'Admin' → shared `users` → `user_databases` INSERT OR REPLACE (academy_name, org) → `createTrialSubscription` → `signToken{dbUrl,dbToken,academyName}` → cookie; TS2345 resolved |
| R5 Cleanup | ✅ Implemented | `provisioning.ts:319-334` best-effort DELETE + original error surfaced; `register.ts:130-153` handler DELETE when `provisioned`, no double-cleanup, fail closed (no JWT/cookie), Spanish messages per design matrix |
| R6 Env gating | ✅ Implemented | `provisioning.ts:101-119` — all 3 vars required; MISSING_ENV → "Registro no disponible temporalmente"; no silent shared-only degradation |
| R7 Payments shared-only | ✅ Implemented | Only per-user write is the `users` insert; `createTrialSubscription` runs against shared `db`; no per-user subscription writes |
| R8 Baseline | ✅ Implemented | 7 failing tests / 7 tsc errors, both ≤ 7 and identical to pre-change set |

## Coherence (Design)

| Decision | Followed? | Notes |
|----------|-----------|-------|
| D1 Inline sync provisioning in register | ✅ Yes | `provisionUser` called inline in `registerHandler` |
| D2 Shared DB only; no per-user subscription writes | ✅ Yes | R7 test asserts zero per-user `subscriptions` writes |
| D3 Keep `002_seed_admin.sql` | ✅ Yes | 002 copied byte-identical, annotated no-op in README |
| D4 `client.executeMultiple(sql)` per file | ✅ Yes | `provisioning.ts:270`; tested on real `file:` libsql client |
| D5 Bare DB name to `createAuthToken` | ✅ Yes | `provisioning.ts:179-204`; tests assert URL contains bare name, not `libsql://` |
| D6 Fail closed if Turso env missing | ✅ Yes | `getTursoEnv` throws MISSING_ENV; action maps to generic error |
| Error matrix | ✅ Yes | All rows implemented + tested (zod per-field, CONFLICT, MISSING_ENV, createDatabase, post-create) |
| Migration ordering 001→020 | ✅ Yes | 20 real files applied in order on fresh DB; 010–016 safe via `_schema_migrations` guard |

## Issues Found

**CRITICAL**: None.

**WARNING**:
- W1 (R4/R6): Vercel rollout **documented, not executed** — `TURSO_API_TOKEN`/`TURSO_ORG`/`TURSO_GROUP` must be set in Vercel production before deploy (`web/docs/vercel-rollout.md`); until then registration fails closed by design (D6). Deployment prerequisite, not a code defect.
- W2 (R8): E2E not re-run during this verify pass (each run creates a real Turso DB + shared rows, not auto-cleaned per apply-progress issue 5); relies on apply-progress evidence: 9 passed / 8 skipped / 0 failed, green twice.
- W3 (R2): `createDatabase` returns the server-reported `name`/`hostname` rather than the requested name — matches desktop parity; a misbehaving API response could in theory diverge, but the flow uses the response name consistently for token/migrate/DELETE.

**SUGGESTION**:
- S1 (R5): `registerHandler`'s catch re-calls `getTursoEnv()` (register.ts:136); capturing `env` from the happy path would avoid a second env read. Harmless today.
- S2 (R2): 4-hex suffix = 16 bits of entropy; acceptable because 409 retries absorb collisions — no change needed, noted for awareness.
- S3 (R8): E2E spec could tag test rows/DBs (e.g., email prefix) to ease manual cleanup; apply-progress issue 5 already suggests it.
- S4 (R8): Pre-existing Stripe-era failures (7 tests, 7 tsc errors) are untouched — consider a separate cleanup change to restore a green baseline.

## Verdict

**Envelope verdict: FAIL** — mechanical canonical-failure representation, NOT a requirement failure. `sdd-verify-validate` admits only a `fail` verdict when the declared commands exit non-zero (`test_exit_code: 1`, `build_exit_code: 2`), and the report format classifies command-exit evidence as a canonical failure: *valid and persistable but not archive-ready*. Both non-zero exits come exclusively from the **R8-approved pre-existing baseline** (7 Stripe-era test failures, 7 pre-existing tsc errors — both exactly ≤ budget, both zero new from this change). The focused suites for this change exit 0 (38 + 12).

**Human requirement assessment: PASS WITH WARNINGS** — All 8 requirements (R1–R8) and all 17 scenarios are COMPLIANT (see matrix); no CRITICAL findings; every requirement verified by code inspection plus passing tests. Warnings are deployment/process (Vercel vars documented, not yet set; E2E evidence from apply-progress, not re-run here).

**Archive readiness**: not mechanically archive-ready until the full suite and tsc exit 0. The only path to a green envelope is fixing the out-of-scope Stripe-era failures (spec Non-Goals); the gatekeeper may acknowledge this fail envelope as the R8-approved baseline and proceed.
