# Proposal: Turso SaaS Migration

## Intent

Transform Academix from a local desktop Tauri application into a SaaS platform where each academy tenant gets their own isolated Turso database. Currently, Academix uses a single local SQLite file — this change makes it cloud-native with per-user database provisioning via Turso Platform API, offline capability via Turso Embedded Replicas, and a superadmin account that manages all client databases centrally.

This enables subscription-based access, cloud backup, offline resilience, and zero-effort multi-tenancy without RLS complexity.

## Scope

### In Scope
- Per-user Turso database provisioning on registration (no RLS for tenant separation)
- Superadmin (master account) manages all client databases via Platform API
- Registration form with academy name field, triggers Turso DB creation
- Migration from `rusqlite` + `SqlitePool` (sync, Mutex) to `libsql` (async, per-user connections)
- Offline sync: local embedded replica + automatic push/pull on connectivity restore
- Control plane database mapping user → database URL + auth token
- Refactoring all Rust repositories for dynamic per-user database connections
- All 18 existing migrations converted to Turso-compatible format

### Out of Scope
- Superadmin dashboard UI (admin management panel) — deferred to a follow-up change
- Subscription/payments integration (Stripe, etc.) — deferred
- Multi-user collaboration within the same academy DB — single-user-per-DB for now
- Migration of existing local SQLite data to Turso — manual import by superadmin
- RLS-based multi-tenancy — explicitly not desired, per-user databases instead

## Approach

### Architecture Overview

```
┌─────────────────────────────────────────────┐
│           Turso Platform API                 │
│  (POST /v1/organizations/:org/databases)     │
└────────────────┬────────────────────────────┘
                 │
    ┌────────────┴────────────┐
    │   Control Plane DB      │
    │  (local SQLite or       │
    │   Turso management DB)  │
    │                         │
    │  users table:           │
    │  user_id | db_url       │
    │  | auth_token | org     │
    └────────────┬────────────┘
                 │
    ┌────────────┴────────────┐
    │  Connection Manager     │
    │  (per-user libsql pool) │
    └────────────┬────────────┘
                 │
     ┌───────────┴───────────┐
     │  User's Turso DB      │
     │  academy-{slug}       │
     │  (isolated tenant)    │
     └───────────────────────┘
```

### User Registration Flow
1. User fills registration form (email, password, academy_name)
2. Backend validates and creates Turso database via Platform API: `POST /v1/organizations/academix/databases` with name `academy-{slug}`
3. Database is created, migrations run against it
4. Auth token is generated and stored in control plane
5. User record created with mapping to their DB URL + token
6. Response includes session token + connection info

### Login Flow
1. User logs in with email + password
2. Auth service resolves user's Turso database from control plane
3. Creates a `libsql` connection to the user's database
4. Caches the connection in a pool manager
5. Returns session token

### Offline Sync (Embedded Replicas)
1. On first connection, create local SQLite file as embedded replica: `Database::open_with_remote_sync(path, url, token)`
2. App reads/writes to local replica (zero-latency, offline-safe)
3. On connectivity: auto-sync via `sync()` — pushes local changes, pulls remote changes
4. On disconnect: continues writing to local WAL
5. On reconnect: background `sync()` reconciles; no "cleanup" needed — local replica IS the working copy
6. For true offline-first: no cleanup DB strategy needed; the embedded replica handles this natively

### Implementation Order (Phase Plan)

**Phase 1: Foundation** (non-breaking, add libsql + provisioning)
- Add `libsql` and `reqwest` to Cargo.toml
- Create `TursoProvisioningService` (Platform API client)
- Create `ControlPlaneRepository` (maps user → DB)
- Control plane is a local SQLite file (managed by superadmin)

**Phase 2: Registration Flow**
- Add `academy_name` to RegisterForm (React)
- Modify register command: create Turso DB → run migrations → store mapping
- Handle loading/error states for 2-5s DB creation latency

**Phase 3: Dynamic Connections**
- Create `ConnectionManager` — holds `HashMap<UserId, libsql::Database>`
- Create `DynamicDbProvider` — resolves which DB to use per request
- Refactor `AppState` from `Arc<SqlitePool>` to `ConnectionManager`

**Phase 4: Repository Migration**
- Refactor every repository from `rusqlite` sync to `libsql` async
- Each repo receives a `&Connection` or `&Database` dynamically
- Migrate all 18 migrations to Turso-compatible SQL
- Update all Tauri commands to pass user context

**Phase 5: Offline Sync**
- Configure Embedded Replica for each user database
- Add auto-sync on app startup and connectivity change
- Add sync status indicator in UI

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/Cargo.toml` | Modified | Add `libsql`, `reqwest`, `tokio` deps |
| **NEW** — provisioning service | New | `src-tauri/src/infrastructure/turso/provisioning.rs` — Platform API client |
| **NEW** — control plane | New | `src-tauri/src/infrastructure/turso/control_plane.rs` — user→DB mapping |
| **NEW** — connection manager | New | `src-tauri/src/infrastructure/turso/connection_manager.rs` — dynamic pool |
| `pool.rs` | Modified | `Arc<SqlitePool>` → `ConnectionManager` with per-user DB resolution |
| `src-tauri/src/lib.rs` | Modified | Init control plane + connection manager instead of single pool |
| `register.rs` | Modified | Add Turso DB creation + academy_name field |
| `auth.rs` | Modified | Resolve user's DB on login |
| `auth.rs` (use case) | Modified | Dynamic DB resolution in AuthService |
| `src-tauri/src/infrastructure/repositories/*.rs` | Modified | All repos: rusqlite → libsql async, receive `&Connection` |
| `src-tauri/src/application/use_cases/*.rs` | Modified | Pass user context for DB resolution |
| `src/features/auth/components/RegisterForm.tsx` | Modified | Add "academy_name" input field |
| `src-tauri/migrations/` | Verified | All 18 migrations are Turso-compatible (SQLite subset) |
| `src-tauri/src/domain/` | Unchanged | Domain entities and ports stay the same |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Retrofit all repos to async libsql touches 15+ files | High | Do phase 4 as a systematic sweep, one repo at a time, with `libsql` examples |
| rusqlite sync → libsql async changes query API | High | `libsql` uses familiar SQLite syntax; main difference is async + connection pooling changes |
| Turso Platform API rate limits | Medium | Implement retry with exponential backoff; queue provisioning requests if needed |
| Database creation latency (2-5s) during registration | Medium | Show loading spinner with progress indication; consider async provisioning with polling |
| Auth token expiration for Turso databases | Medium | Store token + expiry in control plane; auto-rotate before expiry |
| Offline sync conflict with multiple devices per academy | Low | Single user per academy DB for now; LWW for embedded replica |
| All 18 migrations may need adjustment for Turso | Low | Turso is SQLite-compatible; mainly check for features that libsql doesn't support |
| Migration of existing users' local data | Low | Deferred to manual superadmin import process |

## Rollback Plan

1. **Per phase rollback**: Each phase can be reverted independently by reverting commits
2. **Full rollback**: Restore `main` branch to the last commit before Phase 1, restore single SQLite pool, all existing data remains intact locally
3. **Data safety**: No existing local SQLite data is modified or deleted during the migration — the old DB file remains untouched
4. **Feature flag**: Consider a `use_cloud_db` config flag that falls back to local SQLite if the Turso migration fails

## Dependencies

- Turso account with Organization created (`academix`)
- Turso Platform API token (superadmin)
- `libsql` crate (Rust) — Turso's client library
- `reqwest` crate — HTTP client for Platform API calls
- `turso-cli` installed for initial setup and debug

## Success Criteria

- [ ] New user registration creates an isolated Turso database and user can log in
- [ ] Existing login flow resolves to the correct user database
- [ ] All CRUD operations work against Turso databases (accounting, attendance, students, payments, etc.)
- [ ] Offline sync: app works without internet, data syncs on reconnect
- [ ] Superadmin can list all client databases via control plane
- [ ] All 18 migrations execute successfully on new Turso databases
- [ ] TypeScript compilation passes (`bunx tsc --noEmit`)
- [ ] Tauri build succeeds (`bun run build`)
- [ ] No existing local data is lost during transition
