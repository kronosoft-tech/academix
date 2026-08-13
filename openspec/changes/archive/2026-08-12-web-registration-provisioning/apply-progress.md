# Apply Progress: Web Registration Provisioning (per-user Turso DB at web registration)

- Change: `web-registration-provisioning`
- Artifact store: hybrid (openspec file + engram observation)
- Batch: 3 of 3 (final) — Phase 4: verification & rollout (PR 4 slice; chain strategy: stacked-to-main)
- Date: 2026-08-12
- Mode: Standard (strict_tdd not enabled; test runner exists — Vitest — but no strict_tdd flag in openspec/config.yaml)
- Delivery strategy: auto-forecast; `Chain strategy: stacked-to-main`; batch 3 = PR 4 work unit (E2E spec + docs, well under 400 changed lines)
- Commits (local only, no PRs per orchestrator instruction):
  1. `8e0e89b` — `chore(web): mirror desktop migrations 001-020 into web/migrations/per-user`
  2. `189ec5f` — `feat(web): add Turso provisioning library (slug, REST client, migration runner)`
  3. `38f0c9f` — `feat(web): wire per-user provisioning into register action with env gating and cleanup`
  4. `b66e7d5` — `feat(web): add academyName and confirmPassword to registration form`
  5. `f21285c` — `test(web): e2e registration provisions per-user db and dashboard loads`
  6. `6d209bd` — `docs(web): document Vercel rollout env vars for provisioning`

## Completed Tasks (cumulative — ALL 17)

### Phase 1 — Foundation

- [x] **1.1** Env setup — `TURSO_API_TOKEN`/`TURSO_ORG`/`TURSO_GROUP` verified present in `web/.env` (gitignored; confirmed not committed). AC met.
- [x] **1.2** Copied `src-tauri/migrations/001..020_*.sql` → `web/migrations/per-user/` byte-identical (20 files, sorted 001–020; 002 kept per D3). Verified diff clean vs source.
- [x] **1.3** Created `web/migrations/per-user/README.md` — sync procedure for future 021+, D3 annotation for 002.

### Phase 2 — Provisioning Library

- [x] **2.1** `generateDbSlug` in `web/src/lib/provisioning.ts` — lowercase → hyphenate → collapse → 30-cap → `academy-{x}-{4hex}`; desktop parity verified against `provisioning.rs:54`. Tests: lowercase/spaces/special/long/underscore/unique-suffix.
- [x] **2.2** REST client — `createDatabase` (POST `/v1/organizations/{org}/databases`, 409-retry ≤3 fresh suffix), `createAuthToken` (bare name, `{permission:"full-access"}` per D5), `deleteDatabase` (404=ok). Error matrix CONFLICT/HTTP/AUTH mapped. Tests: fetch-mock.
- [x] **2.3** `ProvisioningError` codes (`SLUG`, `CONFLICT`, `HTTP`, `AUTH`, `MIGRATION`, `MISSING_ENV`) + `getTursoEnv` fail-closed (R6/D6). Tests: env-missing cases.
- [x] **2.4** `runMigrationsOnDb` — `_schema_migrations` tracking table, per-file `client.executeMultiple(sql)` then record, sorted file order (D4). Tests: real `file:` libsql client + 20 real files (fresh apply, rerun no-op, failing file aborts).
- [x] **2.5** `provisionUser` — env gate → create → token → migrate → `ProvisionResult{dbName, dbUrl, dbToken, hostname}`; best-effort DELETE cleanup on failure. Tests: mocked fetch + real libsql integration (409 retry, 409×4 exhaustion, dbUrl `libsql://{hostname}`).

### Phase 3 — Register Wiring

- [x] **3.1** `registerSchema` in `web/src/actions/register.ts` — `academyName: z.string().trim().min(2)`, `confirmPassword` refine (mismatch → error on `confirmPassword` path), no client slug field (verified: extra `slug` key is stripped by `safeParse`). Tests: valid+trim, whitespace-only rejected, mismatch path, slug-stripping.
- [x] **3.2** Handler ordering (R4) — email-exists check (CONFLICT, before any provisioning) → `hashPassword` → `provisionUser` → per-user `users` row (`'Admin'`) → shared `users` row → `user_databases` INSERT OR REPLACE (incl. `academy_name`, `org`) → `createTrialSubscription(id, 'trial', null)` (R7: shared only). Test asserts the exact 9-step order via a shared mock order log; per-user client never sees a `subscriptions` write; duplicate email → `provisionUser` never called.
- [x] **3.3** `signToken` with `dbUrl`/`dbToken`/`academyName` claims — resolves the pre-existing TS2345 (register.ts now typechecks). Test asserts `signToken` called with `expect.objectContaining({ sub, email, role: 'Admin', type: 'customer', dbUrl, dbToken, academyName })`.
- [x] **3.4** Cleanup — `catch` block: best-effort `deleteDatabase(org, provisioned.dbName)` when a DB was already created, then map to ActionError (Spanish messages per design error matrix); no JWT, no cookie on any failure. `mapProvisioningError` distinguishes MISSING_ENV / AUTH+MIGRATION / CONFLICT+HTTP+RATE_LIMIT rows. No double-cleanup: provisionUser's internal DELETEs are not repeated by the handler (tested). Tests: shared-write failure → DELETE called; signToken failure → DELETE called; per-user insert failure → DELETE called; env missing → fail closed, no DELETE, no JWT.
- [x] **3.5** `register.astro` — added `academyName` + `confirmPassword` fields with per-field error rendering via `isInputError` + `fieldErrors` map (R1). Build passes (`bun run build` complete).
- [x] **3.6** `web/src/test/register-action.test.ts` — 12 tests: schema (4) + happy path R4 ordering/claims/DB-write assertions (1) + failure matrix (7: duplicate email, MISSING_ENV, CONFLICT map, MIGRATION no-double-cleanup, shared-write failure → DELETE, signToken failure → DELETE, per-user failure → DELETE + no JWT). All 12 pass.

### Phase 4 — Verification & Rollout

- [x] **4.1** E2E — new `web/tests/e2e/provisioning.spec.ts`: register (academyName + confirmPassword) → provision per-user DB → dashboard loads via `getUserDb` (asserts "Panel de Control" heading + zero "Error de conexión" blocks). Gated on real Turso env (`test.skip(condition)` pattern matching the repo; spec self-loads `web/.env` because Playwright has no dotenv here). **Ran green against the real Turso platform twice** (`bun run test:e2e` → 9 passed / 8 skipped / 0 failed). `auth.spec.ts` kept passing — its `input[type="password"]` selector was fixed for the two-password form (batch 2's confirmPassword made it ambiguous).
- [x] **4.2** Baseline (R8) — `bun run test` → **7 failed / 156 passed** (7 ≤ 7 ✓; identical to pre-change set); `bunx tsc --noEmit` → **7 errors** (7 ≤ 7 ✓; exactly the pre-existing set: db.ts TS2352, payments.test.ts TS2307 ×5, checkout-integration TS2367). No pre-existing (Stripe-era) failures touched.
- [x] **4.3** Vercel rollout — **documented only** (see Issues: vercel CLI not installed locally → no credentialed access to set env vars; the exact commands the user must run are in `web/docs/vercel-rollout.md` and below). Deploy order requirement: set `TURSO_API_TOKEN` / `TURSO_ORG` / `TURSO_GROUP` in Vercel production **BEFORE** deploy (registration fails closed without them — D6); values already exist in `web/.env`.

## Work Unit Evidence (hard gate — every batch must produce this)

| Unit | Focused test command and exact result | Runtime harness command/scenario and exact result | Rollback boundary |
|------|---------------------------------------|---------------------------------------------------|-------------------|
| 1 — migration copies | `bunx tsc --noEmit`: passed with expected 8 pre-existing errors (unchanged vs baseline). No test command for SQL-only copies. | N/A — SQL files; runtime boundary is unit 2's runner tests against a real `file:` libsql client. | Delete `web/migrations/per-user/`; no code imports it yet. |
| 2 — provisioning library | `bun run test -- src/test/provisioning.test.ts` → **38 passed (38), 0 failed**. Full suite: **7 failed / 144 passed** — identical 7 pre-existing Stripe/gateway failures, zero new. | `bunx tsc --noEmit` → 8 errors, all pre-existing (incl. register.ts TS2345, Phase 3's target). | `git revert 189ec5f` — removes `provisioning.ts` + its test; no callers existed yet. |
| 3 — register wiring | `bun run test -- src/test/register-action.test.ts` → **12 passed (12), exit 0** (Vitest 3.2.7). Full suite regression: **7 failed / 156 passed** — same 7 pre-existing failures, zero new (+12). | `bunx tsc --noEmit` → **7 errors (was 8) — TS2345 gone; remaining 7 are exactly the pre-existing set** (db.ts TS2352, payments.test.ts TS2307 ×5, checkout-integration TS2367). `bun run build` (astro build + Vercel adapter) → **Complete** (server built; only pre-existing prerender-header warnings + local Node 26→24 runtime notice). | `git revert` the batch-2 commits — removes `register.ts` + `register.astro` + `register-action.test.ts`; `provisioning.ts` untouched; any orphan DBs from failed runs are best-effort DELETEd by the handler itself (R5). |
| 4 — E2E + R8 + rollout | `bun run test:e2e` (real env) → **9 passed / 8 skipped / 0 failed** (my provisioning test ran green twice; the 8 skips are the pre-existing env-gated DB tests). R8: `bun run test` → **7 failed / 156 passed**; `bunx tsc --noEmit` → **7 errors** — both ≤ 7 ✓. | Real browser flow: register form → POST action → per-user Turso DB created + 20 migrations applied → JWT cookie → `/dashboard` renders "Panel de Control" with per-user stats (no "Error de conexión") — proves `getUserDb` connects with the JWT `dbUrl`/`dbToken` claims (R4/R8). | `git revert f21285c 6d209bd` — removes the E2E spec + auth.spec selector fix + rollout doc; code units 1–3 untouched; Vercel vars removable via `vercel env rm <VAR> production`. |

## Design-Verified Decisions (cumulative)

- **R4 ordering** is enforced by a test asserting the exact 9-step sequence across all mocked collaborators — a regression here fails loudly. The E2E run then proves the same ordering end-to-end against the real platform.
- **R5/R6 fail-closed**: every failure path test asserts no JWT is signed and no cookie is set; `MISSING_ENV` returns "Registro no disponible temporalmente" (no silent shared-only degradation). E2E gate skips the flow when the vars are absent — never half-runs it.
- **R7**: shared-only subscription verified — the per-user `users` insert is the only write to the per-user client (asserted: zero `subscriptions` writes to it), and `createTrialSubscription` runs against the shared DB.
- **R8 baseline final**: failing tests 7 (≤7 ✓); tsc errors 7 (≤7 ✓) — both identical to the pre-existing set, zero new.
- **zod**: `astro/zod` re-exports `z` (bundles zod 4.4.3); `z.string().trim().min()` + `.refine({ path })` confirmed available. Verified `actions.register` is referenced only in `register.astro` (no other callers to update).

## Deviations from Design

1. (batch 3) `auth.spec.ts` was modified after all — the design said the E2E task must leave it untouched. The pre-existing `input[type="password"]` selector became ambiguous (strict-mode violation) the moment batch 2 added `confirmPassword` to the register form; task 4.1's AC "auth.spec.ts unaffected" was interpreted as "the suite must still pass", so the selector was fixed to `input[name="password"]` + `input[name="confirmPassword"]` + `input[name="academyName"]`. Minimal, no other test touched.
2. (batch 3) The repo's Playwright types (1.62.1) do not expose `describe.skipIf` — used `test.skip(condition, reason)` (the repo's own pattern) instead. First tsc run caught TS2551 from my own spec; fixed before commit (R8: 7 errors).
3. (batch 3) Playwright does not auto-load `web/.env` (no dotenv dep) — the spec's gate reads the three vars from `web/.env` itself so it matches what the Astro dev server sees.
4. (batch 2) `astro:actions` is a virtual module — vitest needs the factory mocks before `import { registerSchema }` resolves; happy-path order test uses a single shared `order[]` log.

## Issues Found

1. (batch 1) `vi.unstubAllGlobals()` does not restore `vi.stubEnv` stubs — leaked env vars poison later tests (fixed with `vi.unstubAllEnvs()` in `afterEach`).
2. (batch 1) Path-resolution gotcha: `new URL('../../migrations/per-user/', import.meta.url)` from `web/src/lib/` (not `../`).
3. (batch 2) Note for rollout (4.3): the handler's fail-closed behavior means registration returns "Registro no disponible temporalmente" until the 3 Turso vars are set in Vercel — deploy order matters.
4. (batch 3) **Vercel rollout is documented, NOT executed** — `vercel` CLI is not installed on this machine and no credentials are available (`vercel whoami` fails: command not found). Per task 4.3, deployment was NOT attempted. The user must run the steps in `web/docs/vercel-rollout.md`: `bunx vercel login` (if needed) → `bunx vercel link` (project not linked — no `web/.vercel/project.json`) → `bunx vercel env add TURSO_API_TOKEN|TURSO_ORG|TURSO_GROUP production` → verify with `bunx vercel env ls production` → THEN deploy `bunx vercel --prod`. Values come from `web/.env` (already present, gitignored).
5. (batch 3) E2E creates real artifacts per run: one per-user Turso DB + shared-DB rows (users, user_databases, trial) with unique email/academyName. Two successful runs during this batch left two test DBs; they are functional (not orphans) but the shared rows + DBs are not auto-cleaned — acceptable for a gated real-env test; manual cleanup possible via Turso API/CLI.
6. (batch 3) Local dev server on :4321 predates this batch (Astro 7 dev daemon); a stale-server race caused one `webServer exited early` — re-run reused the healthy server. Not a code issue; noted for CI where `reuseExistingServer` is off.

## Remaining Tasks

None — all 17 tasks complete (1.1–4.3). User action required (not an SDD task): set the 3 Vercel env vars and deploy per `web/docs/vercel-rollout.md`.

## Status

**17/17 tasks complete** (Phase 1: 3/3, Phase 2: 5/5, Phase 3: 6/6, Phase 4: 3/3). R8 gates hold: failing tests 7 ≤ 7 ✓, tsc errors 7 ≤ 7 ✓, E2E green (9/0/8). Ready for verify.

## Next Steps

- `sdd-verify` against spec R1–R8 and tasks 1.1–4.3.
- After verify: user runs the Vercel env-var steps (documented, not executed) and deploys.
- Known follow-ups (out of scope, tracked separately): backfill design for existing web-registered users; 020_web_schema alignment; Stripe-era test failures; `send-reminders` cron `grace_end` bug; Vercel Hobby 10 s cold-start timeout monitoring (open question in design).
