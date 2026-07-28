# Tasks: Turso SaaS Migration (v2 — In-Memory + Flush)

**Review Workload Forecast:**
- Estimated total changed lines: **~1,200**
- 800-line budget risk: **High** (but manageable with chained PRs)
- Chained PRs recommended: **Yes** (6 sequential PRs)

---

## Phase 1: Foundation — MemoryBuffer + Turso Infrastructure

> Zero behavioral change. All new code, no existing files modified (except Cargo.toml).

### 1.1 Add libsql and reqwest dependencies ✅

- **File**: `src-tauri/Cargo.toml`
- **Action**: Added `libsql = { version = "0.5", features = ["remote"] }` and `reqwest = { version = "0.12", features = ["json", "rustls-tls"] }`
- **Verification**: `cargo check` passes
- **Note**: `libsql` feature `rustls` does not exist on 0.5.0 — used `remote` instead. `rusqlite` stays until Phase 4.

### 1.2 Create Turso infrastructure module ✅

- **File**: `src-tauri/src/infrastructure/turso/mod.rs`
- **Action**: Module declaration exporting: `connection_manager`, `control_plane`, `flush_timer`, `memory_buffer`, `provisioning`

### 1.3 Implement MemoryBuffer ✅

- **File**: `src-tauri/src/infrastructure/turso/memory_buffer.rs`
- **Action**: Implemented `MemoryBuffer` with `pending_writes`, `cached_entities`, `last_write_at`. Includes `CachedEntity`, `BufferedOperation` enum, full method set plus unit tests.
- **Verification**: 7 unit tests covering buffer writes, idle timer, cache, clear

### 1.4 Implement FlushTimer ✅

- **File**: `src-tauri/src/infrastructure/turso/flush_timer.rs`
- **Action**: Background tokio task polling every 30s, 15-min idle timeout flush, `flush_on_close` with 5s timeout, `start_flush_timer` returns shutdown sender
- **Note**: SQL value binding uses empty params stub — actual binding in Phase 4.

### 1.5 Implement TursoProvisioningService ✅

- **File**: `src-tauri/src/infrastructure/turso/provisioning.rs`
- **Action**: Created with `api_token`, `org`, `reqwest::Client`. Methods: `create_database`, `create_auth_token`, `list_databases`, `delete_database`. Custom `ProvisioningError` enum. 6 unit tests for `generate_db_slug`.
- **Note**: 409 Conflict retry loop (3 attempts) + rate limit detection.

### 1.6 Implement ConnectionManager ✅

- **File**: `src-tauri/src/infrastructure/turso/connection_manager.rs`
- **Action**: `HashMap<String, CachedConnection>` with lazy init via `libsql::Builder::new_remote()`. Methods: `resolve_by_email`, `resolve_by_user_id`, `register_connection`, `run_migrations` (stub).
- **Note**: Used `libsql::Builder` pattern (new API) instead of deprecated `Database::open_remote`.

### 1.7 Create slug generator ✅

- **File**: `src-tauri/src/infrastructure/turso/provisioning.rs`
- **Action**: `fn generate_db_slug(academy_name: &str) -> String` — lowercase, spaces→hyphens, special chars removed, 30-char limit, 4-char random suffix
- **Verification**: 6 unit tests covering lowercase, spaces, special chars, long names, uniqueness

---

## Phase 2: Control Plane (Turso-backed)

> Prerequisite: superadmin creates `academix-control-plane` DB manually via turso-cli.

### 2.1 Implement ControlPlaneRepository (Turso-backed)

- **File**: `src-tauri/src/infrastructure/turso/control_plane.rs`
- **Action**: Connect to Turso DB via libsql, manage `user_databases` table
- **Schema**:
  ```sql
  CREATE TABLE user_databases (
    user_id TEXT PRIMARY KEY, email TEXT NOT NULL UNIQUE,
    academy_name TEXT NOT NULL, db_url TEXT NOT NULL,
    db_token TEXT NOT NULL, org TEXT DEFAULT 'academix',
    created_at TEXT NOT NULL
  );
  CREATE TABLE users (id, email, password_hash, name, role, is_active, created_at, updated_at);
  CREATE TABLE sessions (id, user_id, token, expires_at, created_at);
  ```
- **Methods**: `save_user_db()`, `find_by_email()`, `find_by_user_id()`, `list_all_databases()`
- **Verification**: Integration test against real Turso DB (smoke test)

### 2.2 Add env vars for Turso configuration

- **File**: `src-tauri/src/env_loader.rs`
- **Action**: Add loading for `CONTROL_PLANE_DB_URL`, `CONTROL_PLANE_DB_TOKEN`, `TURSO_API_TOKEN`, `TURSO_ORG`

### 2.3 Seed superadmin in control plane on startup

- **File**: `src-tauri/src/lib.rs`
- **Action**: On app startup, connect to control plane Turso DB, seed superadmin user (same pattern as existing `seed_admin_user`)
- **Verification**: First startup creates superadmin

---

## Phase 3: Registration with Turso DB Creation

### 3.1 Add academy_name to RegisterUserRequest DTO

- **File**: `src-tauri/src/application/dto/user.rs`
- **Action**: Add `academy_name: String` field

### 3.2 Add academy_name to RegisterForm UI

- **File**: `src/features/auth/components/RegisterForm.tsx`
- **Action**: Add `academy_name` to `RegisterFormData`, add input field with label "Nombre de la academia", validation (3-100 chars, alphanumeric + spaces)

### 3.3 Pass academy_name in frontend invoke

- **File**: `src/features/auth/components/RegisterForm.tsx`
- **Action**: Include `academy_name` in the `invoke("register_user", ...)` payload

### 3.4 Modify RegisterUserUseCase with Turso provisioning

- **File**: `src-tauri/src/application/use_cases/register.rs`
- **Action**: Flow:
  1. Validate email + password
  2. Check email uniqueness (control plane)
  3. Generate DB slug from academy_name
  4. `TursoProvisioningService::create_database(slug)` — await (2-5s)
  5. `TursoProvisioningService::create_auth_token(slug)` — await
  6. Open libsql connection to new DB
  7. Run all 18 migrations against new DB
  8. Save user record → MemoryBuffer (queued for flush)
  9. Save mapping → ControlPlaneRepository (direct write)
  10. Return success

### 3.5 Update RegisterForm loading state

- **File**: `src/features/auth/components/RegisterForm.tsx`
- **Action**: Show "Creando tu academia..." with spinner while registering, disable button

### 3.6 Handle registration errors

- **File**: `src/features/auth/components/RegisterForm.tsx`
- **Action**: Error messages for: duplicate email, Turso API failure, slug conflict

---

## Phase 4: Login + MemoryBuffer Integration (HARD CUTOVER)

> This phase removes local SQLite. After this, the app is fully Turso-backed.

### 4.1 Modify AppState for ConnectionManager + MemoryBuffer ✅

- **File**: `src-tauri/src/commands/auth.rs`
- **Action**: `AppState` now holds `ConnectionManager`, `MemoryBuffer`, `ControlPlaneRepository`, `FlushTimer` (all optional for graceful fallback)
- **Action**: Remove old `AppState` that held `AuthService` with local pool
- **Note**: `control_plane` is `Option<Arc<ControlPlaneRepository>>` — login returns helpful error when Turso not configured. `flush_timer_sender: None` (timer not started — Phase 5 will wire it).

### 4.2 Modify login command for Turso DB resolution ✅

- **File**: `src-tauri/src/commands/auth.rs`
- **Action**: Login flow:
  1. Look up email in control plane Turso DB
  2. Resolve user's Turso DB via ConnectionManager
  3. Query `users` table in user's Turso DB
  4. Verify password (bcrypt)
  5. Buffer session creation → MemoryBuffer
- **Verification**: `cargo check` passes. Returns `Result<CommandLoginResponse, String>` (Tauri 2 convention for async commands with `State`).

### 4.3 Add resolve helper for authenticated commands ✅

- **File**: `src-tauri/src/commands/auth.rs`
- **Action**: `resolve_authenticated_user(token, cp, cm, mb)` → stub returning error with guidance. Full implementation deferred to Phase 5.

### 4.4 Route auth commands through Turso AppState ✅

- **File**: `src-tauri/src/commands/auth.rs`
- **Action**: `login`, `logout`, `update_profile`, `change_password` all use new Turso `AppState`. login/logout are fully functional. update_profile/change_password are stubs returning clear error messages.
- **Note**: Other command files (users, students, courses, etc.) still use old managed state. Full MemoryBuffer routing is Phase 5.

### 4.5 Remove SqlitePool (keep rusqlite) ✅

- **File**: `src-tauri/src/infrastructure/database/pool.rs`
- **Action**: DELETED — file removed
- **File**: `src-tauri/src/infrastructure/database/mod.rs`
- **Action**: Replaced pool module with `set_db_path()`/`open_connection()` globals
- **File**: `src-tauri/src/infrastructure/repositories/sqlite/*.rs` (10 files)
- **Action**: All repos rewritten to use `database::open_connection()` instead of `SqlitePool` parameter
- **File**: `src-tauri/src/application/ports/user.rs`
- **Action**: Removed `fn pool()` from `UserRepository` trait
- **DEVIATION**: `rusqlite` NOT removed from `Cargo.toml`. The sqlite repos still use sync rusqlite API. Full removal deferred to Phase 5. `cargo check` passes.
- **Verification**: All 10 repos compile without SqlitePool references.

### 4.6 Update lib.rs entry point ✅

- **File**: `src-tauri/src/lib.rs`
- **Action**: New `run()` flow:
  1. Load env vars for Turso config (`load_turso_config()`)
  2. Connect to control plane Turso DB (or None if not configured)
  3. Create `ControlPlaneRepository`, `ConnectionManager`, `MemoryBuffer`
  4. `run_local_migrations()` for backward-compat SQLite reads
  5. `seed_control_plane_admin()` on startup
  6. Register Turso AppState + old service states for backward compat
  7. Register all command handlers
- **Note**: Uses `tokio::sync::Mutex` for connection_manager and memory_buffer (required for `Send` across `.await`). Flush timer not started (set to None). Old service states kept until Phase 5.

### 4.7 Update commands using new constructors ✅

- **File**: `src-tauri/src/commands/register.rs`
- **Action**: Already uses `SqliteUserRepository::new()` (no pool argument). No changes needed.
- **File**: `src-tauri/src/commands/auth.rs`
- **Action**: All auth commands rewritten to use Turso `AppState` with `tokio::sync::Mutex` for thread-safe async.

---

## Phase 5: Repository Rewrite (MemoryBuffer-backed)

### 5.1 Rewrite UserRepository implementation

- **File**: `src-tauri/src/infrastructure/repositories/sqlite/user.rs`
- **Action**: Replace `SqliteUserRepository` with `MemoryBackedUserRepository`
  - Holds `Arc<Mutex<MemoryBuffer>>` + `user_id: String`
  - `find_by_id()` / `find_by_email()` → check buffer cache, if miss → read from Turso via ConnectionManager
  - `save()` / `update()` / `delete()` → `buffer_write()` to MemoryBuffer

### 5.2 Rewrite SessionRepository

- **File**: `src-tauri/src/infrastructure/repositories/sqlite/session.rs`
- **Action**: Same pattern — MemoryBuffer-backed

### 5.3 Rewrite StudentRepository

- **File**: `src-tauri/src/infrastructure/repositories/sqlite/student.rs`
- **Action**: MemoryBuffer + Turso read-through

### 5.4 Rewrite CourseRepository

- **File**: `src-tauri/src/infrastructure/repositories/sqlite/course.rs`
- **Action**: MemoryBuffer + Turso read-through

### 5.5 Rewrite GroupRepository

- **File**: `src-tauri/src/infrastructure/repositories/sqlite/group.rs`
- **Action**: MemoryBuffer + Turso read-through

### 5.6 Rewrite PaymentRepository

- **File**: `src-tauri/src/infrastructure/repositories/sqlite/payment.rs`
- **Action**: MemoryBuffer + Turso read-through

### 5.7 Rewrite AttendanceRepository

- **File**: `src-tauri/src/infrastructure/repositories/sqlite/attendance.rs`
- **Action**: MemoryBuffer + Turso read-through

### 5.8 Rewrite InvoiceRepository + InvoiceLineRepository

- **File**: `src-tauri/src/infrastructure/repositories/sqlite/invoice.rs`
- **Action**: MemoryBuffer + Turso read-through

### 5.9 Rewrite SettingsRepository

- **File**: `src-tauri/src/infrastructure/repositories/sqlite/settings.rs`
- **Action**: MemoryBuffer + Turso read-through

### 5.10 Rewrite AccountingEntryRepository

- **File**: `src-tauri/src/infrastructure/repositories/sqlite/accounting.rs`
- **Action**: MemoryBuffer + Turso read-through

### 5.11 Clean up old SQLite repository files

- **Files**: `src-tauri/src/infrastructure/repositories/sqlite/*.rs`
- **Action**: Rename to `memory_backed/*.rs` for clarity. Update module declarations.

### 5.12 Update port interfaces

- **File**: `src-tauri/src/application/ports/user.rs`
- **Action**: Remove `fn pool()` method (no longer relevant). Repositories no longer expose a pool.

---

## Phase 6: Superadmin

### 6.1 Add list_client_databases Tauri command

- **File**: `src-tauri/src/commands/admin.rs` (NEW)
- **Action**: `list_client_databases(token: String) -> Vec<ClientDatabaseInfo>`
  - Validate token + check role=Admin
  - Query `ControlPlaneRepository::list_all_databases()`
  - Return: email, academy_name, db_url, created_at

### 6.2 Add admin commands module

- **File**: `src-tauri/src/commands/mod.rs`
- **Action**: Add `pub mod admin;`

### 6.3 Register admin command in lib.rs

- **File**: `src-tauri/src/lib.rs`
- **Action**: Add `list_client_databases` to `generate_handler![]`

---

## Total Task Summary

| Phase | Tasks | Est. Lines | Description |
|-------|-------|-----------|-------------|
| 1 — Foundation | 7 | ~250 | MemoryBuffer, flush timer, provisioning, connection manager |
| 2 — Control Plane | 3 | ~120 | Turso-backed control plane, env vars, superadmin seed |
| 3 — Registration | 6 | ~150 | Academy name, Turso DB creation, loading states |
| 4 — Login + Cutover | 7 | ~300 | MemoryBuffer integration, remove local SQLite, lib.rs rewrite |
| 5 — Repository Rewrite | 12 | ~350 | 11 repos → MemoryBuffer-backed, port updates |
| 6 — Superadmin | 3 | ~50 | Admin command, role check |
| **Total** | **38** | **~1,220** | |

### Chained PR Strategy

```
feature/turso-migration ← acumula todo
  │
  ├── PR #1: Foundation     (~250 lines) → main  (new code, no behavior change)
  ├── PR #2: Control Plane  (~120 lines) → main  (env vars, schema)
  ├── PR #3: Registration   (~150 lines) → main  (new flow, dual-path)
  ├── PR #4: Login + Cutover (~300 lines) → main  (HARD CUTOVER)
  ├── PR #5: Repositories   (~350 lines, split 6+6) → main
  └── PR #6: Superadmin     (~50 lines)  → main
         ↓
  feature/turso-migration mergea a main al final
```

### Risk Priority

| Priority | Task | Why |
|----------|------|-----|
| 🔴 CRITICAL | 1.3 MemoryBuffer | Foundation — everything depends on this |
| 🔴 CRITICAL | 2.1 ControlPlaneRepository | Registration + login need it |
| 🔴 CRITICAL | 4.5 Remove SqlitePool | Hard cutover point — app stops working without it |
| 🟡 HIGH | 3.4 RegisterUserUseCase | Core registration with Turso provisioning |
| 🟡 HIGH | 4.2 Login resolves Turso DB | Core login flow |
| 🟡 HIGH | 5.1-5.10 Rewrite repos | Every feature depends on repos |
| 🟢 MEDIUM | 6.1-6.3 Superadmin | Add-on feature |
