# AGENTS.md - Academix

Two-app repo: a **Tauri 2 + React 19 desktop app** at the repo root, plus an **Astro 7 web app** (marketing site + subscriptions/billing) under `web/`, deployed to Vercel. Both use **bun** and TypeScript. They are separate packages — never mix root and `web/` dependencies.

## Quick Reference

| Component | Location | Notes |
|-----------|----------|-------|
| Desktop frontend | `src/` | React 19, feature-based architecture |
| Desktop backend | `src-tauri/src/` | Rust, hexagonal architecture + Turso |
| Desktop migrations | `src-tauri/migrations/` | SQLite, files 001–020 (runner wired to 019) |
| Web app | `web/` | Astro 7 SSR on Vercel, subscriptions/payments |
| Web shared migrations | `web/migrations/` | Turso subscriptions schema, 001–002 |
| Web per-user migrations | `web/migrations/per-user/` | Mirrors of desktop 001–020 for per-user DBs |
| SDD artifacts | `openspec/` | Active changes + flattened specs (user-provisioning, payments) |
| Unit tests | `src/**/*.{test,spec}.ts(x)`, `web/src/**` | Vitest |
| E2E tests | `tests/e2e/` (root), `web/tests/e2e/` | Playwright |

Nested `AGENTS.md` files exist in `src/` and `src-tauri/` with per-area conventions (import order, naming, Context7 usage). No `opencode.json` / `CLAUDE.md` / `.cursorrules` at root. `.atl/` holds the auto-generated skill registry (`skill-registry.md` + cache, git-tracked) — regenerate via `gentle-ai skill-registry refresh`, not by hand.

---

## Commands

Run desktop commands at the repo root; web commands inside `web/`.

```bash
# --- Desktop (root) ---
bun install
bunx tsc --noEmit                # typecheck (ALWAYS before commit)
bun run build                    # = tsc && vite build → dist/ (Tauri's frontendDist)
bun run tauri dev                # full desktop dev (Vite :1420 + Rust backend)
bun run test                     # Vitest, jsdom, Tauri API mocked
bun run test -- src/features/foo/bar.test.ts   # single test
bun run test:e2e                 # Playwright (auto-starts Vite via webServer)
bun run test:e2e tests/e2e/navigation.spec.ts  # single spec

# --- Web (in web/) ---
bun install
bun run dev                      # Astro dev server on :4321
bun run build                    # astro build (type-safe check for web)
bun run test                     # Vitest, node env
bun run test -- src/lib/foo.test.ts             # single test
bun run test:e2e                 # Playwright against :4321
```

---

## Architecture

### Desktop frontend (`src/`)
- **Feature modules**: `src/features/{feature}/` with `components/`, `hooks/`, `types/`, `routes/`; only some features export an `index.ts`.
- **App shell**: `src/app/` holds layouts and global components. **`src/app/router.tsx` is dead code** — the live router is in `src/App.tsx`.
- **Router**: `HashRouter` (react-router-dom v7 component API, in `src/App.tsx`) — Tauri serves from `file://`, so hash routing is required.
- **IPC**: `invoke()` from `@tauri-apps/api/core` → Rust commands. **Do not** call `tauri-plugin-sql` from the frontend; `src/lib/database.ts` is legacy dead code (all DB access goes through Rust commands).
- **State**: Zustand v5. **Styling**: Tailwind v4 via `@tailwindcss/vite`.

### Desktop backend (`src-tauri/src/`)
- **Hexagonal architecture**: `domain/` → `application/` → `infrastructure/` → `commands/`; commands registered in `lib.rs` via `generate_handler![]`.
- **Data layer (Phase 4/5)**: auth/registration resolve via a **Turso control plane** → per-user **Turso** databases; repositories are `MemoryBuffer`-backed (reads from Turso). A local SQLite DB (opened with the `libsql` Turso SDK, **not** tauri-plugin-sql) is kept for backward-compatible reads.
- **Migrations** run at startup in `run_local_migrations()` (`lib.rs`): a `_schema_migrations` tracking table + `run_migration!` macro (`include_str!`). Files are **not** auto-discovered — a new `src-tauri/migrations/0XX_*.sql` must also be registered in `lib.rs`. Wired: 001, 003–019. Not wired: `002_seed_admin.sql`, `020_web_schema.sql`. Legacy DBs are seeded as migrated up to 018 (avoids re-running non-idempotent 010–016).
- **Env vars** (names; values never committed): desktop loads `.env` at startup via `dotenv`. `CONTROL_PLANE_DB_URL`, `CONTROL_PLANE_DB_TOKEN`, `TURSO_API_TOKEN`, `TURSO_ORG`, `TURSO_GROUP`, plus `ADMIN_EMAIL`/`ADMIN_PASSWORD_HASH` and `APP_IDENTIFIER`. Without the Turso vars the app runs degraded (Turso features disabled, local SQLite fallback). Admin auto-seeding in `lib.rs` is **disabled** — users register themselves.

### Web app (`web/`)
- Astro 7 SSR (`output: 'server'`) with the **Vercel adapter**; `vercel.json` schedules cron jobs (expire-subscriptions 06:00, send-reminders 07:00, charge-wompi 08:00 UTC).
- Purpose: marketing pages + account dashboard + **subscriptions/billing** for the desktop app's customers. Payment gateways: **Wompi and MercadoPago** only — checkouts under `web/src/pages/api/checkout/`, webhooks under `web/src/pages/api/webhooks/` (one module per gateway), plus dashboard-side persistence fallbacks under `web/src/pages/api/payments/` (`verify-wompi.ts`, `verify-mercadopago.ts`) for when webhooks fail.
- **Gateway routing**: `geoToGateway()` in `web/src/lib/payments/gateway.ts` — Colombia (`CO`) → Wompi, everything else (and `null` country) → MercadoPago. The `Gateway` union is `'wompi' | 'mercadopago'`; **Stripe is not implemented** (only a legacy `stripe_subscription_id` column and stale tests importing a non-existent `lib/payments/stripe` module — do not add Stripe without updating `gateway.ts`).
- **Auth**: JWT (jose, HS256) in an httpOnly cookie; `web/src/middleware.ts` splits `customer` vs `admin` roles and enforces route access. For registered users the JWT also carries per-user DB connection claims (`dbUrl`/`dbToken`/`academyName`).
- **Shared DB + per-user DBs**: the control-plane Turso database via `@libsql/client` (`TURSO_URL`/`TURSO_AUTH_TOKEN`) holds users, subscriptions, and payments. Every registered user ALSO gets their own Turso DB — see provisioning below. Email via nodemailer (Gmail); AI chat via groq/cerebras. `web/src/.env.example` is tracked but **stale** (still lists Stripe keys, missing provisioning vars) — trust `web/.env` and `web/docs/vercel-rollout.md` instead; never commit real `.env` values.
- **Per-user provisioning**: `web/src/lib/provisioning.ts` (slug, create DB, auth token, delete, migration runner) + `web/src/actions/register.ts`: email-conflict check → provision per-user DB → per-user `Admin` row → shared `users`/`user_databases` rows → trial subscription → JWT signed with the DB claims. `web/src/lib/user-db.ts` connects to the per-user DB from those claims. Fails closed without `TURSO_API_TOKEN`/`TURSO_ORG`/`TURSO_GROUP` ("Registro no disponible temporalmente"). Migrations run from `web/migrations/per-user/` (mirrors of desktop 001–020; **`002_seed_admin.sql` is intentionally a no-op — do NOT "fix" it**). Keep the mirror in sync when adding desktop migrations. Deploy order + rollback: `web/docs/vercel-rollout.md`. E2E: `web/tests/e2e/provisioning.spec.ts`.
- **Web migrations**: shared `web/migrations/` apply ad hoc — `web/src/lib/payments/migrate.ts` runs the 002 ALTERs, swallowing "duplicate column" errors (SQLite has no `IF NOT EXISTS` for ALTER). There is no full auto-runner wired into app code.
- **Webhook signatures**: Wompi uses SHA256 event signatures (see `wompi` skill); MercadoPago uses HMAC-SHA256 over `id:<data.id>;request-id:<x-request-id>;ts:<ts>;`.
- **Env vars (web)** beyond `TURSO_URL`/`TURSO_AUTH_TOKEN`/`JWT_SECRET`: `CRON_SECRET` (Bearer-gates `/api/cron/*`, 500s if unset), `SITE_URL`, `SUPPORT_EMAIL`, `WOMPI_PUBLIC_KEY`/`WOMPI_PRIVATE_KEY`/`WOMPI_EVENTS_SECRET`/`WOMPI_API_URL`, `MP_ACCESS_TOKEN`/`MP_API_URL`/`MP_WEBHOOK_SECRET`. Missing vars throw at **request time**, not at startup.

### Communication
```
React → invoke("command") → Tauri command → UseCase → Repository → SQLite (local libsql) / Turso (per-user DB)
```
Web: registration/dashboard → shared control-plane Turso (`db.ts`) + per-user Turso DBs (`provisioning.ts` → `user-db.ts`).

---

## Critical Gotchas

1. **Port 1420 is hardcoded** in `vite.config.ts` with `strictPort: true` — desktop dev fails if the port is busy. Web dev uses Astro's default **4321**.
2. **HashRouter, not BrowserRouter** — Tauri serves from `file://`. Live router is in `src/App.tsx`; `src/app/router.tsx` is dead code.
3. **Bun, not npm** — in both root and `web/`.
4. **Desktop migrations aren't auto-discovered** — register new `migrations/*.sql` in `lib.rs` or they never run.
5. **`tauri-plugin-sql` is a stale dependency** — the frontend talks to Rust commands; `src/lib/database.ts` is unused.
6. **`web/` is a separate package** — run its installs/scripts from inside `web/`; don't add web deps to the root `package.json`.
7. **Turso env vars gate features** — desktop runs degraded without them; web registration fails closed without `TURSO_API_TOKEN`/`TURSO_ORG`/`TURSO_GROUP`; missing web vars surface at request time, not startup.
8. **E2E auto-starts dev servers** — both Playwright configs use `webServer` (root: Vite `:1420`; web: Astro `:4321`).
9. **Releases**: `.github/workflows/tauri.yml` builds installers (mac/linux/win matrix) and publishes GitHub releases on `app-v*` tags (used by the updater plugin).
10. **Per-user migration mirror**: `web/migrations/per-user/` must track `src-tauri/migrations/` — when adding a desktop migration, mirror it (and keep `002_seed_admin.sql` as a no-op).
11. **Stale test/artifact traps**: `web/src/test/payments/checkout-integration.test.ts` still expects Stripe routes (`geoToGateway('US') === 'stripe'`) and will fail against current code — not a source of truth. Root `tasks.md` is a stale planning artifact; ignore it.

---

## TypeScript

Strict mode is enabled (`tsconfig.json`): `strict`, `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`. `web/` extends `astro/tsconfigs/strict`.

---

## Pre-commit Checklist

1. `bunx tsc --noEmit` passes (root); `bun run build` succeeds (root and `web/` if web changed)
2. No `console.log` in production code
3. Types are explicit (avoid `any`)
4. Env values never committed (`.env*` is gitignored; `.env.example` files ARE tracked — keep them secret-free)
5. New desktop migrations registered in `lib.rs` AND tested for idempotency
6. If desktop migrations changed, mirror them into `web/migrations/per-user/`

---

## Skills

| Context | Skill |
|---------|-------|
| React 19 | `react-19` |
| React + Tauri | `react-19-tauri` |
| TypeScript | `typescript` |
| Tauri 2 | `tauri-2` / repo-local `tauri`, `tauri-v2` (`.agents/skills/`, managed via `skills-lock.json`) |
| Tauri SQL | `tauri-sql` |
| Zustand | `zustand-5` |
| Tailwind | `tailwind-4` |
| Testing | `playwright` |

### Project-Local Skills (`skills/`)

| Skill | When to Use |
|-------|-------------|
| `turso-cli` / `turso-cli-advanced` | Turso setup, CLI, migrations |
| `turso-sdk` | libsql/Turso client integration in code |
| `turso-security` | Auth tokens, JWT, RLS, multi-tenant access |
| `turso-features` | FTS5, vectors, branching, PITR |
| `turso-api` | Turso platform REST API |
| `turso-sync` | Local-first sync, embedded replicas |
| `turso-sql` | SQLite/Turso SQL reference |
| `wompi` | Wompi (Colombia): widget/checkout integration, SHA256 event signature verification |
| `mercadopago` | Mercado Pago (Colombia): Checkout Pro/Bricks; Colombia does NOT support the preapproval/subscriptions API |
| `astro-7` | Work in `web/` (Astro 7) |