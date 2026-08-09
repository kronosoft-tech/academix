# AGENTS.md - Academix

Desktop academic management system. **Tauri 2 + React 19 + Vite + TypeScript**.

## Quick Reference

| Component | Location | Notes |
|-----------|----------|-------|
| Frontend | `src/` | React 19, feature-based architecture |
| Backend | `src-tauri/src/` | Rust, hexagonal architecture |
| Tests (unit) | `src/**/*.test.ts` | Vitest + jsdom |
| Tests (E2E) | `tests/e2e/` | Playwright |
| Migrations | `src-tauri/migrations/` | SQLite, 001-017 |

---

## Commands

```bash
# Install (use bun, not npm)
bun install

# Type check (ALWAYS before commit)
bunx tsc --noEmit

# Build frontend
bun run build

# Full Tauri dev (starts Vite on :1420 + Rust backend)
bun run tauri dev

# Unit tests
bun run test
bun run test:watch

# E2E tests (requires dev server running)
bun run test:e2e
```

---

## Architecture

### Frontend (`src/`)
- **Feature modules**: `src/features/{feature}/` — each has `components/`, `hooks/`, `types/`, `routes/`, `index.ts`
- **Shared**: `src/shared/ui/components/` — reusable UI (Spinner, etc.)
- **Router**: `HashRouter` (required for Tauri file:// protocol)
- **IPC**: `invoke()` from `@tauri-apps/api/core` → Rust commands
- **State**: Zustand v5
- **Styling**: Tailwind CSS v4 (`@tailwindcss/vite` plugin)

### Backend (`src-tauri/src/`)
- **Hexagonal architecture**: `domain/` → `application/` → `infrastructure/` → `commands/`
- **Commands**: Registered in `lib.rs` via `generate_handler![]`
- **Database**: SQLite via `tauri-plugin-sql`, migrations run on startup
- **Migrations are idempotent**: Safe to re-run (checks for "duplicate column" errors)

### Communication
```
React component → invoke("command_name", { args }) → Tauri command → UseCase → Repository → SQLite
```

---

## Critical Gotchas

1. **Port 1420 is hardcoded** in `vite.config.ts` — `strictPort: true` means dev fails if port is busy
2. **HashRouter, not BrowserRouter** — Tauri serves from `file://`, BrowserRouter won't work
3. **Bun, not npm** — all scripts use `bun run`, `bunx`, etc.
4. **Migrations run on every startup** — new migrations must be idempotent or use the `run_migration!` macro pattern in `lib.rs`
5. **Admin seed**: Default admin credentials are hardcoded in `lib.rs` — override via `ADMIN_EMAIL` and `ADMIN_PASSWORD_HASH` env vars in production
6. **E2E tests need dev server** — Playwright config starts Vite automatically via `webServer.command: "bun run dev"`

---

## TypeScript

Strict mode enabled (`tsconfig.json`):
- `noUnusedLocals: true`
- `noUnusedParameters: true`
- `noFallthroughCasesInSwitch: true`

---

## Pre-commit Checklist

1. `bunx tsc --noEmit` passes
2. `bun run build` succeeds
3. No `console.log` in production code
4. Types are explicit (avoid `any`)

---

## Skills

| Context | Skill |
|---------|-------|
| React 19 | `react-19` |
| React + Tauri | `react-19-tauri` |
| TypeScript | `typescript` |
| Tauri 2 | `tauri-2` |
| Tauri SQL | `tauri-sql` |
| Zustand | `zustand-5` |
| Tailwind | `tailwind-4` |
| Testing | `playwright` |

### Turso Skills (Project-Local)

All Turso skills are located in `skills/` at the project root. Use these when working with Turso database features:

| Skill | When to Use |
|-------|-------------|
| `turso-cli` | Basic CLI commands: install, auth, db create/show/shell, groups, orgs |
| `turso-cli-advanced` | Advanced CLI flags, migrations, AWS migrations, config allow-rules |
| `turso-sdk` | TypeScript/Python/Go/Rust SDK patterns, ORM integration (Drizzle, Prisma) |
| `turso-security` | Auth tokens, RLS policies, JWT, JWKS, fine-grained permissions |
| `turso-features` | Vector search, AI embeddings, FTS5, SQLite extensions, branching, PITR |
| `turso-api` | Platform API REST endpoints, programmatic database management |
| `turso-sync` | Sync push/pull, embedded replicas, conflict resolution, local sync server |
| `turso-sql` | SQL reference: data types, statements, functions, JSON, vectors, pragmas |

**When to load these skills:**
- Working with Turso database setup or configuration → `turso-cli` or `turso-cli-advanced`
- Integrating Turso into frontend/backend code → `turso-sdk`
- Implementing security, authentication, or multi-tenant access → `turso-security`
- Building AI features, vector search, or full-text search → `turso-features`
- Automating database management via API → `turso-api`
- Setting up local-first sync or embedded replicas → `turso-sync`
- Writing complex SQL queries or using advanced SQLite features → `turso-sql`
