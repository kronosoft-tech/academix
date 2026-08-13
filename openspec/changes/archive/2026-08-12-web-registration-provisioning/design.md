# Design: Web Registration Provisioning

## Technical Approach

TS port of desktop `TursoProvisioningService` (`src-tauri/src/infrastructure/turso/provisioning.rs`) + `run_migrations_on_db` (`connection_manager.rs`) into `web/src/lib/provisioning.ts`, wired into the register action (A1): provision per-user Turso DB (create → token → migrate 001–020) → per-user + shared rows → JWT with existing `dbUrl`/`dbToken`/`academyName` claims (D). Shared DB stays payment source of truth (B1). Fail-closed env gating, best-effort DB DELETE cleanup. Backfill, desktop changes, 020 alignment: Non-Goals.

## Architecture Decisions

| # | Choice | Alternatives | Rationale |
|---|---|---|---|
| D1 | Inline sync provisioning in register (A1) | A2 queued | Mirrors desktop; user provisioned before JWT signing; no new infra. Cold-start latency accepted |
| D2 | Shared DB only; no per-user subscription writes (B1) | B2 dual-write, B3 migrate | Per-user 020 tables have zero consumers; CHECKs reject `plan_id 'basico'`/`provider 'wompi'` (R7) |
| D3 | **Keep** `002_seed_admin.sql` | Exclude | R3 mandates copies 001–020; `INSERT OR IGNORE` swallows NOT NULL `id` violation → silent no-op; `_schema_migrations` stays byte-identical to desktop |
| D4 | `client.executeMultiple(sql)` per file | Split + `batch()` | 1:1 with desktop `execute_batch`; transactional; typed on `Client` in @libsql/client 0.14.0 |
| D5 | **Bare DB name** to `createAuthToken` | Copy `lib.rs:355` (URL) | `lib.rs:355` passes `libsql://{hostname}` — pre-existing bug; do NOT replicate. `register.rs:118` is the reference |
| D6 | Fail **closed** if Turso env missing | Degrade to shared-only | Silent degradation recreates today's broken state (R6) |

## Data Flow

```
register.astro ──POST──▶ register action
  zod: academyName trim.min(2) + confirmPassword refine
  │ email-exists (shared) ──CONFLICT──▶ error (no provision)   │ hash (bcrypt 12)
  ▼
provisionUser: getTursoEnv() ──missing──▶ fail closed
  createDatabase (409 ─▶ fresh 4-hex suffix, ≤3 retries)
  createAuthToken (bare name)  →  dbUrl = libsql://{hostname}
  runMigrationsOnDb ◀── web/migrations/per-user/001..020
  ▼
per-user users 'Admin' ─▶ shared users ─▶ user_databases INSERT OR REPLACE
  ▼
createTrialSubscription ─▶ signToken{dbUrl,dbToken,academyName} ─▶ cookie ─▶ /dashboard
catch ─▶ deleteDatabase (best-effort) ─▶ ActionError (no JWT)
```

## Interfaces / Contracts

```ts
// web/src/lib/provisioning.ts
export interface ProvisionResult { dbName: string; dbUrl: string; dbToken: string; hostname: string; }
export type ProvisioningErrorCode = 'MISSING_ENV'|'HTTP'|'RATE_LIMIT'|'CONFLICT'|'AUTH'|'MIGRATION';
export class ProvisioningError extends Error { constructor(code: ProvisioningErrorCode, message: string); }

export function generateDbSlug(academyName: string): string; // lowercase → [^a-z0-9-]→'-' → collapse → trim → 30 cap → `academy-{x}-{4hex}`
export async function createDatabase(org: string, name: string, group: string): Promise<{ name: string; hostname: string }>;
export async function createAuthToken(org: string, dbName: string): Promise<string>;   // bare name
export async function deleteDatabase(org: string, dbName: string): Promise<void>;        // 404 = ok
export async function runMigrationsOnDb(client: Client): Promise<void>;
export async function provisionUser(academyName: string): Promise<ProvisionResult>;      // orchestrates + env gate
```

`runMigrationsOnDb`: create `_schema_migrations(version TEXT PK, applied_at TEXT DEFAULT (datetime('now')))`; read `web/migrations/per-user/*.sql` sorted; per file — skip if recorded, else `client.executeMultiple(sql)` then record version.

Register action: + `academyName: z.string().trim().min(2)`, + `confirmPassword` via `z.object().refine(pw === confirm, { path: ['confirmPassword'] })`. No client slug field. `signToken` gains `dbUrl`/`dbToken`/`academyName` (resolves TS2345; claims already in `CustomerJwtPayload`). Cookie ~1.2–1.8 KB (< 4 KB).

## File Changes

| File | Action | Description |
|---|---|---|
| `web/src/lib/provisioning.ts` | Create | Slug, REST client, migration runner, `provisionUser` |
| `web/migrations/per-user/001..020_*.sql` | Create | Copies of desktop migrations (002 annotated no-op) |
| `web/migrations/per-user/README.md` | Create | Sync note: copy procedure for future 021+ |
| `web/src/actions/register.ts` | Modify | zod schema, provisioning ordering, cleanup, signToken claims |
| `web/src/pages/auth/register.astro` | Modify | academyName + confirmPassword fields |
| `web/.env` + Vercel env | Modify | `TURSO_API_TOKEN`/`TURSO_ORG`/`TURSO_GROUP` (never committed) |
| `web/src/lib/auth.ts` | No change | Call site only |

## Error Handling Matrix

| Stage | Error | Cleanup | User message (ES) |
|---|---|---|---|
| zod | field error | none | per-field |
| email exists | CONFLICT | none | Ya existe una cuenta con este email |
| env missing | MISSING_ENV | none | Registro no disponible temporalmente |
| createDatabase | CONFLICT/HTTP | none (no DB) | No se pudo crear tu academia, intenta de nuevo |
| createAuthToken / migrate / later writes / signToken | AUTH/MIGRATION/infra | DELETE db | No se pudo completar el registro, intenta de nuevo |

Fail closed: no JWT, no success, on every error.

## Migration Ordering

001 core → 002 seed admin (no-op) → 003 guardian/schedule → 004 enrollment → 005 course price → 006 group schedule → 007 start_date → 008 end_date → 009 due_date nullable → 010 accounting → 011 PUC seed → 012 liabilities/equity → 013 fixed assets → 014 fixed-asset accounts → 015 pasivo accounts → 016 simplified accounting → 017 app settings → 018 class duration/skipped dates → 019 payment_type → 020 web schema (additive). 010–016 non-idempotent by content — safe: each file runs once on a fresh DB, `_schema_migrations` guards reruns, a failing file aborts registration and the DB is deleted.

## Threat Matrix

N/A — no routing, shell, subprocess, VCS/PR automation, executable-file classification, or process-integration boundary. Outbound HTTPS with bearer tokens and SQL from static files are covered in Testing Strategy.

## Testing Strategy

| Layer | What | How |
|---|---|---|
| Unit | `generateDbSlug` (port desktop tests); rerun no-op | Vitest; file: libsql client |
| Integration | 409→fresh-suffix; 409×4 exhausted; token/DELETE; bare-name in URL; 20 real files | Vitest, mocked fetch; real libsql client |
| Action | happy path; duplicate email pre-provisioning; env fail-closed; post-create failure → DELETE; no JWT on failure | Vitest |
| E2E | register form → dashboard loads | Playwright; gated on real Turso env |
| Baseline | failing tests ≤ 7; tsc ≤ 7 (R8) | `vitest`, `tsc --noEmit` |

## Migration / Rollout

No shared-schema change, no data migration, no feature flag. Set the 3 Turso vars in Vercel **before** deploy (else registration fails closed — by design). Rollback: revert `register.ts` + `register.astro`, delete `provisioning.ts` + `web/migrations/per-user/`; residual orphan DBs removed via Turso API.

## Open Questions

- [ ] Backfill design for existing web-registered users (urgent follow-up; they stay broken until it lands)
- [ ] Vercel Hobby 10 s timeout on cold-start provisioning — monitor; Fluid compute or A2 later
- [ ] Final Spanish error copy for provisioning failures
