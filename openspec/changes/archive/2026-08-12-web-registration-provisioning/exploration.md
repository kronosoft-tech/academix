# Exploration: Web registration provisioning (per-user Turso DB)

## Context

Web users register at `web/src/pages/auth/register.astro` → `register` action (`web/src/actions/register.ts`). Today the action inserts the user into the **shared** users table and creates a trial subscription row in the **shared** subscriptions table — but never provisions the per-user Turso database that desktop users get. Result: after login the dashboard (`getUserDb`) cannot find a `user_databases` row and the web-registered user is effectively broken.

Desktop (`src-tauri/src/application/use_cases/register.rs`, `infrastructure/provisioning.rs`) resolves auth/registration via the Turso control plane: `ControlPlaneDb::create_user` → `provisioning.provision` (create DB → create auth token → run migrations) → `user_databases` mapping (INSERT OR REPLACE) → per-user `users` row (role `'Admin'`) → per-user subscription tables. `lib.rs` also computes `db_name = generate_db_slug(name)` and passes `config.turso_group` as the group.

## Current State

### Web (Astro 7 + @libsql/client)
- `web/src/lib/auth.ts` — JWT via jose HS256; `CustomerJwtPayload` already declares `sub, email, role, name, academyName, dbUrl, dbToken` (auth.ts:5-15). `signToken` sets `{dbUrl, dbToken, academyName}` only when a dbUrl is present.
- `web/src/lib/payments/user-db.ts` — `getUserDb(payload)` errors with `NOT_FOUND` when `payload.dbUrl/dbToken` are missing → dashboard breaks for web-registered users.
- `web/src/lib/db.ts` — shared Turso client (`TURSO_URL`/`TURSO_AUTH_TOKEN`); **no Turso REST API client exists in web/** (the REST client lives in desktop `infrastructure/provisioning.rs` only).
- `web/src/actions/register.ts` — inserts shared users row, hashes password (bcrypt cost 12), calls `createTrialSubscription`, signs JWT (TS2345: dbUrl/dbToken/academyName missing). `web/src/pages/auth/register.astro` — fields: `name, email, password`.
- Consumers of the extra JWT claims already exist: `UserNavbar.astro` (initials + academyName display), dashboard metrics via `getUserDb`.
- Cron note (pre-existing): `web/src/pages/api/cron/send-reminders.ts` selects `s.grace_end` (column does not exist in shared schema; only per-user 020 has `grace_end`) and uses `u.name as academy_name` instead of the academy name — out of scope, flagged in Risks.

### Shared vs per-user schema divergence (critical decision point)
| Aspect | Shared (web 001/002 + `web/migrations/migrate.ts`) | Per-user (desktop `src-tauri/migrations/020_web_schema.sql`) |
|---|---|---|
| plan column | `plan_id` TEXT (`'basico'/'pro'/'premium'`) | `plan` TEXT CHECK (`'basic','pro','premium'`) |
| trial start | `trial_starts_at` | `trial_start` |
| grace | `grace_expires_at` | `grace_start`, `grace_end` |
| provider | `provider` (wompi/mercadopago), `provider_subscription_id`, `provider_customer_id`, `payment_source_token`, `updated_at` | absent; `subscription_payments.provider` CHECK is `('stripe','mercadopago','payu')` — **rejects 'wompi'** |

**Every existing consumer reads the shared DB**: subscription/payments pages, checkout routes, wompi/mercadopago webhooks (`activateSubscription`), crons, and the desktop control-plane guard (`get_subscription_status`). The per-user 020 subscription tables have **zero consumers** today.

### Test/typecheck baseline (pre-existing, must not grow)
- `bunx tsc --noEmit`: 8 errors — db.ts TS2352 + 5 Stripe-era imports (test files) + 1 checkout-integration comparison + 1 register.ts TS2345. Fixing register.ts reduces the count to 7.
- 7 failing tests, all Stripe-era (`stripe-checkout`, `checkout`, `checkout-integration`); `auth.test.ts` passes (signs full payloads incl. dbUrl/academyName — unaffected).

## Approaches

### A. Provisioning implementation

**A1. Inline in the register action (recommended)** — new `web/src/lib/provisioning.ts` (TS port of desktop `infrastructure/provisioning.rs`): `generateDbSlug(name)` (same algorithm: lowercase → hyphenate → cap 30 chars → `academy-` prefix → 4-char hex suffix), `createDatabase(org, name, group)` POST `https://api.turso.tech/v1/organizations/{org}/databases` (409 → retry ≤3 with new suffix), `createAuthToken(org, dbName)` POST `/v1/organizations/{org}/databases/{name}/auth/tokens` (body `{permission: "full-access"}`, note: **pass the bare db name, not the `libsql://` URL** — desktop lib.rs:353 passes the URL, a pre-existing bug), `dbUrl = libsql://{hostname}`, `runMigrationsOnDb(client)` (TS port of `run_migrations_on_db`: `_schema_migrations` tracking table + per-file `client.batch([...])`, idempotent).
  - Pros: mirrors desktop exactly; synchronous so the user is fully provisioned before the JWT is signed; no new infra.
  - Cons: adds latency to register (2 API calls + 21-file migration batch on a fresh remote DB); Vercel serverless timeout risk (see Risks).
  - Effort: Medium.

**A2. Queued/background provisioning** — register returns immediately, a job provisions asynchronously.
  - Pros: fast UX, no function-timeout risk.
  - Cons: JWT must be signed without dbUrl → dashboard still broken until the job completes; needs queue infra on Vercel (out of scope); duplicate complexity.
  - Effort: High.

### B. Where the subscription/payment lifecycle lives (what "payment persistence in scope" means)

**B1. Shared DB stays the source of truth (recommended)** — web-registered users already get a shared trial row (works for any user_id); wompi/mercadopago webhooks, pages, and crons already write/read shared. **Do not** write to per-user 020 subscription tables (zero consumers, and CHECK constraints reject `plan_id 'basico'`/`provider 'wompi'` → dual-write is impossible without a schema migration).
  - Pros: zero behavioral change to payments; smallest surface; consistent with every existing consumer.
  - Cons: none for this change; the 020 divergence stays latent (deferred).
  - Effort: Low (mostly a decision, not code).

**B2. Dual-write shared + per-user** — requires aligning per-user schema first (new desktop migration 021 renaming columns, widening CHECKs, adding provider columns) + replicating every lifecycle write.
  - Pros: per-user tables become "real".
  - Cons: big migration + write fan-out with no consumer to justify it; desktop would also need the alignment; high drift risk.
  - Effort: High.

**B3. Migrate payments fully to per-user DBs** — architecturally cleanest long-term, but every desktop/web consumer would need to read per-user DBs via dbToken, and web crons would need per-user loops. Large, separate change.
  - Effort: Very High.

### C. Form fields
- Add **academyName** (text, required, `z.string().trim().min(2)`; slug derived server-side via `generateDbSlug` — no client slug field) and **confirmPassword** (`z.object().refine(pw === confirm, { path: ['confirmPassword'] })`) to `register.astro` + the action input schema. Mirrors desktop fields.

### D. JWT claims
- Supply `dbUrl`, `dbToken`, `academyName` in the register action's `signToken` call (claims + consumers already exist — fixes TS2345). No middleware change (`locals.user` only carries sub/email/role; dashboard uses `getFullTokenPayload`). Cookie size grows to ~1.2–1.8 KB with the Turso token — under the ~4 KB cookie limit.

## Recommendation

1. **A1** — inline provisioning in `web/src/lib/provisioning.ts`, wired into the register action, with `TURSO_API_TOKEN`/`TURSO_ORG`/`TURSO_GROUP` from env (fail **closed** if missing — no degraded registration, or we recreate today's broken state).
2. **Per-user migrations**: copy `src-tauri/migrations/*.sql` (001–020, incl. 002_seed_admin) into `web/migrations/per-user/` + `_schema_migrations` tracking, mirroring `run_migrations_on_db`. Note: 002's INSERT lacks an `id` (PK TEXT NOT NULL) so it is a silent no-op on fresh DBs — harmless; keep for parity or exclude deliberately.
3. **B1** — shared DB remains the payment source of truth; no per-user subscription writes; 020 divergence is a deferred, separate schema-alignment change (with a consumer story).
4. **Ordering** in register: validate → email exists check → hash → provision+migrate → per-user `users` row (role `'Admin'`) → shared users INSERT → `user_databases` INSERT OR REPLACE (with `academy_name`) → `createTrialSubscription` → signToken with dbUrl/dbToken/academyName. Wrap DB writes in try/catch with best-effort cleanup (DELETE the created DB via API on failure).
5. **Backfill is OUT of scope but urgent**: existing web-registered users (no `user_databases` row, no per-user DB) remain broken after this change → plan a separate backfill change (data migration + one-off provisioning job) immediately after.

## Risks

- **CRITICAL — Existing web-registered users stay broken** until a backfill change lands; register/login gives `NOT_FOUND`. Treat the backfill as the follow-up change.
- **CRITICAL — Env gating**: without `TURSO_API_TOKEN`/`TURSO_ORG`/`TURSO_GROUP` in `web/.env` (and Vercel env), registration must fail closed; silent degraded registration recreates the broken state.
- **WARNING — Vercel function timeout**: provisioning on a cold start may approach the default 10 s (Hobby). Monitor; if timeouts occur, enable Fluid compute or move to A2 later.
- **WARNING — Schema duplication**: `web/migrations/per-user/` diverges from `src-tauri/migrations/` over time; add a cross-reference note (or sync check). Future desktop migration 021+ must be copied too.
- **WARNING — Partial-failure orphans**: a retry after a mid-flow failure creates a new DB (random slug suffix); best-effort API DELETE in the catch block; accept residual orphans.
- **WARNING — dbToken in the app JWT**: full-access Turso token inside the httpOnly cookie for 7 d; matches the desktop control-plane pattern; consider shorter-lived/rotated db tokens later.
- **SUGGESTION (pre-existing, out of scope)**: `send-reminders.ts` cron queries non-existent shared column `s.grace_end` and uses `u.name as academy_name`; desktop lib.rs:353 passes the `libsql://` URL as the DB name to `create_auth_token` (web implementation must pass the bare name); desktop control-plane role `"user"` vs web `'Admin'` role divergence.

## Ready for Proposal

Yes — scope is clear: web-only provisioning (A1) + shared-payments decision (B1) + form fields (C) + JWT claims (D), with backfill explicitly deferred as its own change. Recommend the proposal phase next (`sdd-propose`, change: `web-registration-provisioning`).
