# Proposal: Web Registration Provisioning

## Intent

Web-registered users never get their per-user Turso database: the register action inserts into the shared users table and creates a shared trial row, but no `user_databases` row or per-user DB is provisioned. After login, `getUserDb` returns `NOT_FOUND` and the account is broken. This change provisions a per-user Turso DB at web registration (mirroring the desktop flow) so web-registered users become first-class.

## Scope

### In Scope
- `web/src/lib/provisioning.ts`: `generateDbSlug` (`academy-` + slug + 4-char hex), `createDatabase` (Turso REST, 409-retry ≤3 with new suffix), `createAuthToken` (bare DB name, `full-access`), `dbUrl = libsql://{hostname}`, `runMigrationsOnDb` (TS port: `_schema_migrations` + idempotent batches)
- `web/migrations/per-user/`: copies of desktop migrations 001–020
- Register form/action: academyName (trim ≥2), confirmPassword; env-gated provisioning (fail **closed** without `TURSO_API_TOKEN`/`TURSO_ORG`/`TURSO_GROUP`); per-user `users` row (role `'Admin'`); `user_databases` INSERT OR REPLACE (with `academy_name`); signToken with dbUrl/dbToken/academyName; best-effort DB DELETE cleanup on failure
- Ordering: validate → email-exists → hash → provision+migrate → per-user users → shared users → user_databases → createTrialSubscription → signToken
- Shared DB remains subscription/payment source of truth (B1)

### Out of Scope (Non-Goals)
- **Backfill of existing web-registered users** — separate follow-up change; they remain broken until it lands
- Desktop-side changes; 020 schema alignment (plan vs plan_id, grace cols, provider CHECKs); Stripe
- `send-reminders` cron `s.grace_end` bug (pre-existing)

## Capabilities

### New Capabilities
- `user-provisioning`: per-user Turso DB provisioning at web registration — slug generation, DB/token creation via Turso REST, migration runner, `user_databases` mapping, registration form contract, env gating, cleanup

### Modified Capabilities
- None — B1 keeps the lifecycle on the shared DB; existing `subscription-lifecycle` spec unchanged

## Approach

TS port of desktop `provisioning.rs` wired into the register action (A1); shared DB stays payment source of truth (B1); academyName/confirmPassword fields with server-side slug (C); JWT gains existing dbUrl/dbToken/academyName claims (D). Pre-existing baseline (7 failing tests; 7 tsc errors after fixing register.ts) must not grow.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `web/src/lib/provisioning.ts` | New | Turso REST provisioning + migration runner |
| `web/migrations/per-user/` | New | Copies of desktop 001–020 |
| `web/src/actions/register.ts` | Modified | Input schema, provisioning, ordering, cleanup |
| `web/src/pages/auth/register.astro` | Modified | academyName + confirmPassword fields |
| `web/src/lib/auth.ts` | Modified | signToken call sites (claims already declared) |
| `web/.env` + Vercel env | Modified | TURSO_API_TOKEN / TURSO_ORG / TURSO_GROUP |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Existing web users stay broken | Certain | Follow-up backfill change planned |
| Missing env vars → broken registration | Med | Fail closed at register |
| Vercel function timeout (cold start) | Med | Monitor; Fluid compute or A2 later |
| Per-user/desktop migration drift | Med | Sync note; copy future 021+ |
| Orphan DBs on partial failure | Med | Best-effort API DELETE; accept residuals |
| dbToken inside app JWT (7 d) | Low | Matches desktop pattern; rotation later |

## Rollback Plan

Revert register action to pre-change state (no provisioning); delete `provisioning.ts` + `web/migrations/per-user/`; remove form fields. No shared-DB schema change. Residual DBs removed via Turso API.

## Dependencies

- `TURSO_API_TOKEN` / `TURSO_ORG` / `TURSO_GROUP` in `web/.env` and Vercel
- Desktop `src-tauri/migrations/` 001–020 as migration source

## Success Criteria

- [ ] New web registration provisions per-user DB + `user_databases` row; dashboard loads
- [ ] Pre-existing failures not exceeded (7 failing tests; 7 tsc errors after register fix)
- [ ] Registration fails closed when Turso env vars missing
- [ ] Slug collision retries with fresh suffix (≤3)

## Open Questions

Exploration resolved the main decisions (A1, B1, C, D). Remaining items — backfill design and Vercel timeout tuning — are tracked as risks/follow-ups, not blockers.
