# Tasks: Web Registration Provisioning (per-user Turso DB at web registration)

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~2,000 (1,126 migration copies + ~300 provisioning.ts + ~120 register.ts + ~45 register.astro + ~350 tests + ~40 docs) |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 → PR 2 → PR 3 → PR 4 |
| Delivery strategy | auto-forecast |
| Chain strategy | stacked-to-main |

```text
Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: stacked-to-main
400-line budget risk: High
```

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| 1 | Copy desktop migrations 001–020 + README | PR 1 | `bunx tsc --noEmit` (repo stays compiling) | N/A — SQL files; no runtime boundary until runner exists (verified by runner tests in unit 2) | Delete `web/migrations/per-user/`; no code imports it yet |
| 2 | `provisioning.ts` (slug, REST, token, runner, provisionUser) + tests | PR 2 | `bun run test -- src/test/provisioning.test.ts` | N/A — Turso API mocked via fetch mock; runner tested on `file:` libsql client, no real env | Delete `provisioning.ts` + test; no callers yet |
| 3 | Wire register action + form + action tests | PR 3 | `bun run test -- src/test/register-action.test.ts` | `bun run test:e2e web/tests/e2e/auth.spec.ts` gated on real Turso env in `web/.env` | Revert `register.ts` + `register.astro`; orphan DBs via Turso API |
| 4 | E2E register→dashboard + R8 baseline + Vercel env | PR 4 | `bun run test` (failing ≤7) + `bunx tsc --noEmit` (errors ≤7) | Real Turso register→dashboard in browser | Revert units 1–3; Vercel vars removable |

## Commit Plan (work units)

1. `chore(web): mirror desktop migrations 001-020 into web/migrations/per-user`
2. `feat(web): add Turso provisioning library (slug, REST client, migration runner)`
3. `feat(web): wire per-user provisioning into register action with env gating and cleanup`
4. `feat(web): add academyName and confirmPassword to registration form`
5. `test(web): e2e registration provisions per-user db and dashboard loads`

## Phase 1: Foundation

- [x] 1.1 Env setup — set `TURSO_API_TOKEN`/`TURSO_ORG`/`TURSO_GROUP` in `web/.env` (gitignored). AC: values present locally; never committed. Dep: —. Tests: none.
- [x] 1.2 Copy `src-tauri/migrations/001..020_*.sql` → `web/migrations/per-user/` byte-identical (keep 002 per D3). AC: 20 files sorted 001–020, diff clean vs source. Dep: —.
- [x] 1.3 Create `web/migrations/per-user/README.md` — sync note: copy procedure for future 021+, D3 002 annotation. AC: procedure documented. Dep: 1.2.

## Phase 2: Provisioning Library (`web/src/lib/provisioning.ts`)

- [x] 2.1 `generateDbSlug` — lowercase→hyphenate→collapse→30-cap→`academy-{x}-{4hex}`; port desktop tests (R2). AC: parity with `provisioning.rs:54`. Tests: unit (lowercase/spaces/special/long/underscore/unique suffix). Dep: —.
- [x] 2.2 REST client — `createDatabase` (POST `/v1/organizations/{org}/databases`, 409-retry ≤3 fresh suffix), `createAuthToken` (bare name, `{permission:"full-access"}`, D5), `deleteDatabase` (404=ok) (R2/R5). AC: error-matrix rows CONFLICT/HTTP/AUTH mapped. Tests: fetch-mock. Dep: 2.1.
- [x] 2.3 `ProvisioningError` codes + `getTursoEnv` — fail closed if any var missing (R6/D6). AC: MISSING_ENV thrown, no silent degradation. Tests: env-missing cases. Dep: 2.2.
- [x] 2.4 `runMigrationsOnDb` — `_schema_migrations` table, per-file `client.executeMultiple(sql)` then record (R3/D4). AC: fresh DB applies 001–020; rerun no-op; failing file aborts. Tests: `file:` libsql client + 20 real files. Dep: 1.2.
- [x] 2.5 `provisionUser` orchestrator — env gate → create → token → migrate → `ProvisionResult{dbName,dbUrl,dbToken,hostname}`; best-effort DELETE on failure (R2/R5). AC: 409→fresh suffix, 409×4→error, dbUrl=`libsql://{hostname}`. Tests: integration (mocked fetch + real libsql). Dep: 2.2–2.4.

## Phase 3: Register Wiring (`web/src/actions/register.ts` + `register.astro`)

- [x] 3.1 zod schema — `academyName: z.string().trim().min(2)` + `confirmPassword` refine; no client slug field (R1). AC: whitespace rejected; mismatch error on confirmPassword. Dep: —.
- [x] 3.2 Handler ordering (R4) — email-exists (CONFLICT pre-provisioning) → hash → `provisionUser` → per-user users `'Admin'` → shared users → `user_databases` INSERT OR REPLACE (academy_name) → `createTrialSubscription` (R4/R7). AC: duplicate email → no DB provisioned; trial row only in shared DB. Dep: 2.5, 3.1.
- [x] 3.3 `signToken` with `dbUrl`/`dbToken`/`academyName` — resolves TS2345 (R4/D). AC: claims present in payload; cookie ~1.2–1.8 KB (<4 KB). Dep: 3.2.
- [x] 3.4 Cleanup — catch → `deleteDatabase` best-effort → ActionError, no JWT (R5; matrix rows createAuthToken/migrate/writes/signToken). AC: post-create failure → DELETE attempted; fail closed. Dep: 3.2.
- [x] 3.5 `register.astro` — add academyName + confirmPassword fields (R1). AC: fields render; per-field errors shown. Dep: 3.1.
- [x] 3.6 Action tests — happy path, duplicate email, env fail-closed, post-create failure→DELETE, no JWT (R4/R5/R6 matrix). AC: all error-matrix rows covered. Dep: 3.4, 3.5.

## Phase 4: Verification & Rollout

- [x] 4.1 E2E — register (academyName + confirm) → dashboard loads via `getUserDb` (gated on real Turso env). AC: spec passes locally with env; `auth.spec.ts` unaffected. Dep: 3.6, 1.1.
- [x] 4.2 Baseline (R8) — `bun run test` failing ≤ 7; `bunx tsc --noEmit` errors ≤ 7 (8 → 7 after register fix). AC: counts ≤ 7. Dep: all.
- [x] 4.3 Vercel rollout — set 3 Turso vars in Vercel BEFORE deploy (fail-closed by design, D6). AC: vars set; deploy order documented. Dep: 4.2.

## Out of Scope (NOT tasks)

- Backfill of existing web-registered users (separate follow-up change; they remain broken until it lands)
- Desktop-side changes; `020_web_schema.sql` alignment (`plan_id` vs `plan`, grace cols, provider CHECKs); Stripe
- `send-reminders` cron `s.grace_end` bug (pre-existing)
