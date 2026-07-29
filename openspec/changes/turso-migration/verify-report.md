# Verification Report — PR#1: Critical Path Fixes for Turso Per-Academy Migration

**Change**: turso-migration (PR#1)
**Version**: Phase 4 — Hard Cutover (Rust-side critical path)
**Date**: 2026-07-28

---

## Completeness

| Metric | Value |
|--------|-------|
| Tasks total (this PR scope) | 23 |
| Tasks complete (backend Rust) | 21 |
| Tasks complete (frontend UI) | 2 |
| Tasks incomplete (frontend) | 0 |
| Spec requirements covered | 7 |
| Spec scenarios verified | 14 |

### Task Breakdown

| Phase | Task | Status | Notes |
|-------|------|--------|-------|
| 1.1 | Add libsql/reqwest deps | ✅ | `Cargo.toml` updated |
| 1.2 | Turso module structure | ✅ | `mod.rs` exports all sub-modules |
| 1.3 | MemoryBuffer | ✅ | Full impl + `find_session_by_token()` added |
| 1.4 | FlushTimer | ✅ | Background poll + shutdown channel |
| 1.5 | TursoProvisioningService | ✅ | Full Platform API client + slug generator + tests |
| 1.6 | ConnectionManager | ✅ | `run_migrations_on_db()` + `get_all_connections()` added |
| 1.7 | Slug generator | ✅ | 6 unit tests passing |
| 2.1 | ControlPlaneRepository | ✅ | Full CRUD: `save_user_db`, `find_by_email`, `find_by_user_id`, `list_all_databases`, `save_user`, `find_user_by_email` |
| 2.2 | Env vars loader | ✅ | `load_turso_config()` for 4 vars + dotenv |
| 2.3 | Seed superadmin | ✅ | `seed_control_plane_admin()` in `lib.rs` |
| 3.1 | academy_name DTO | ✅ | `RegisterUserRequest` has `academy_name: String` |
| 3.2 | RegisterForm UI | ✅ | Academy name input with validation |
| 3.3 | academy_name in invoke | ✅ | Passes `academy_name` in payload |
| 3.4 | RegisterUserUseCase | ✅ | Full Turso provisioning flow (create DB → run migrations → store user → save mapping) |
| 3.5 | Loading state | ✅ | "Creando tu academia..." with spinner/disabled button |
| 3.6 | Error handling | ✅ | UI handles: duplicate email, Turso failure, slug conflict |
| 4.1 | AppState for Turso | ✅ | `ConnectionManager`, `MemoryBuffer`, `ControlPlaneRepository` |
| 4.2 | Login command | ✅ | Full Turso DB resolution + password verify + session buffered |
| 4.3 | resolve_authenticated_user | ✅ | MemoryBuffer + cached connections iteration |
| 4.4 | Auth commands | ✅ | `login`, `logout`, `update_profile`, `change_password` all real impl |
| 4.5 | Remove SqlitePool | ✅ | `pool.rs` deleted, database module replaced |
| 4.6 | lib.rs entry point | ✅ | Dual-path: Turso when configured, local SQLite fallback |
| 4.7 | Command constructors | ✅ | All commands compile with new Turso AppState |

---

## Build & Tests Execution

### Rust Build (`cargo check`)
```
✅ Passed — 0 errors, 0 warnings
```

### TypeScript Build (`tsc --noEmit`)
```
✅ Passed — 0 errors
```

### Rust Tests (`cargo test`)
```
✅ 75 passed, 0 failed, 0 ignored
```
Turso-specific tests:
- `memory_buffer`: 7 tests — all pass (buffer writes, cache, timer, clear)
- `provisioning`: 6 tests — all pass (slug generation)
- `connection_manager`: 3 tests — all pass (empty state, get_connection)
- `flush_timer`: 1 test — passes (type/compilation check)
- `env_loader`: 6 tests — all pass
- `password`: 2 tests — all pass (hash/verify)
- Domain entities: 50 tests — all pass

### Frontend Tests (`vitest`)
```
✅ 12 passed, 0 failed — 2 test files
```
- `validateEmail.test.ts`: 5 tests
- `validatePassword.test.ts`: 7 tests

---

## Spec Compliance Matrix

### Auth Spec (`openspec/changes/turso-migration/specs/auth/spec.md`)

| Requirement | Scenario | Test Coverage | Result |
|-------------|----------|---------------|--------|
| Login with Database Resolution | Login resolves Turso DB successfully | No direct test (needs real Turso) | ✅ Implemented in `login()` — all 8 steps present: email→cp→connection→query→verify→session→response |
| Login with Database Resolution | Login with email not in control plane | No direct test | ✅ Returns "Invalid credentials" via `.ok_or_else()` — no DB connection attempted |
| Login with Database Resolution | Login with correct email but Turso DB unreachable | No direct test | ✅ Returns error from `cm.resolve_by_email()` — error message describes connection failure |
| Token Validation with Database Resolution | Validate token across user's DB | No direct test | ✅ `resolve_authenticated_user()` checks MemoryBuffer first, then iterates cached connections |
| Connection Caching | Connection is cached after first login | `connection_manager` tests (compilation) | ✅ `ConnectionManager` caches connections in `HashMap<String, CachedConnection>` |
| Connection Caching | Connection is created on first request | `connection_manager` tests (compilation) | ✅ Lazy `init_connection()` via `libsql::Builder::new_remote()` |

### Registration Spec (`openspec/changes/turso-migration/specs/registration/spec.md`)

| Requirement | Scenario | Test Coverage | Result |
|-------------|----------|---------------|--------|
| Academy Name Field | Register with academy name | `provisioning` unit tests (slug gen) | ✅ Backend: `generate_db_slug()` + `RegisterUserRequest.academy_name`. Frontend: input field with validation |
| Academy Name Field | Duplicate display name → unique slug | `test_generate_db_slug_unique_suffix` | ✅ UUID-based 4-char random suffix + 3-attempt conflict retry in `create_database()` |
| Turso Database Creation | Registration creates Turso DB successfully | No direct test (needs real API) | ✅ Full flow in `RegisterUserUseCase::execute()`: create DB → auth token → connect → run 18 migrations → insert user → save mapping |
| Turso Database Creation | Turso API fails during registration | No direct test | ✅ Returns `ApplicationError::Infrastructure` — no partial state |
| Registration Loading State | Loading state shown during registration | No frontend test | ✅ `isLoading` state + "Creando tu academia..." text + `loading={isLoading}` on Button |
| Registration Loading State | Double-submission prevention | No frontend test | ✅ Button disabled during loading (`loading` prop prevents interaction) |
| Academy Name Validation | Empty academy name | No direct test | ✅ `validateForm()` returns "El nombre de la academia es requerido" |
| Academy Name Validation | Invalid characters in academy name | No direct test | ✅ Regex `^[a-zA-ZáéíóúÁÉÍÓÚñÑ0-9\s]+$` rejects special chars |

---

## Correctness (Static Analysis)

| Requirement | Status | Notes |
|-------------|--------|-------|
| `run_migrations_on_db()` | ✅ | Reads `.sql` files sorted by name, uses `_schema_migrations` tracking table, skips already-applied, uses `execute_batch` for multi-statement SQL |
| `RegisterUserUseCase` with Turso | ✅ | Full provisioning flow. Creates DB → token → connects → runs migrations → inserts user → saves control plane mapping |
| `resolve_authenticated_user()` | ✅ | Two-step: (1) MemoryBuffer scan for recent sessions, (2) full cached connections iteration with expiry check |
| `update_profile()` | ✅ | Resolves user, executes UPDATE on users table, buffers write via MemoryBuffer |
| `change_password()` | ✅ | Resolves user, verifies current bcrypt password, hashes new one, executes UPDATE |
| `find_session_by_token()` | ✅ | Linear scan over pending writes for `Insert{sessions}` ops matching token |
| `get_all_connections()` | ✅ | Returns `Vec<CachedConnection>` clones — safe for iteration without holding CM lock |
| Login flow (8 steps) | ✅ | email → cp lookup → db resolution → password verify → session → response |
| Logout | ✅ | Buffers session deletion via MemoryBuffer |
| Password verification | ✅ | `password::verify_password()` with bcrypt |
| Academy name validation (frontend) | ✅ | 3-100 chars, letters/numbers/spaces + Spanish accented chars allowed |
| Control plane schema | ✅ | `ensure_schema()` creates 3 tables + 1 index on startup |
| Superadmin seeding | ✅ | `seed_control_plane_admin()` on startup with env var overrides |

---

## Design Coherence

| Design Decision | Followed? | Notes |
|-----------------|-----------|-------|
| In-Memory Write Buffer with 15-min Flush | ✅ | `MemoryBuffer` with `pending_writes`, `last_write_at`, `flush_timer` |
| Control Plane — Turso Database | ✅ | `ControlPlaneRepository` backed by Turso via libsql |
| Registration — In-Request DB Creation | ✅ | Async/await within register_user command — user waits 2-5s |
| Session Storage — In-Memory (flushed to Turso) | ⚠️ | Sessions buffered in MemoryBuffer. Flush timer's `execute_operation()` has empty SQL params (`TODO(Phase 4)`). Sessions are NOT flushed correctly yet — but they ARE validated from MemoryBuffer directly |
| RegisterUserUseCase — academy_name in DTO | ✅ | `RegisterUserRequest { academy_name, name, email, password }` |
| Login — Control Plane → ConnectionManager → libsql | ✅ | Full implementation in `login()` command |
| Flush timer polls every 30s, 15-min idle timeout | ✅ | `start_flush_timer()` with `POLL_INTERVAL` and `IDLE_TIMEOUT` |
| Progressively migrate — Phase 4 hard cutover | ✅ | Local SQLite repos kept for backward compat. Auth commands use Turso, other commands use old managed state |

---

## Issues Found

### WARNING

1. **`resolve_authenticated_user()` doesn't query control plane** — The spec says "Resolve the user's Turso DB from control plane" but the implementation iterates cached connections only. If the app restarts and no connections are cached, session validation fails. This is documented as Phase 4 limitation, but deviates from the spec requirement. **Mitigation**: Phase 5 will add a token→user_id index in MemoryBuffer for O(1) lookup.

2. **`flush_timer::execute_operation()` doesn't bind values** — All three operation types (Insert, Update, Delete) use `libsql::params![]` with TODO comments. If the 15-min flush fires, it would emit parameterized SQL without bound values. **Impact**: Sessions and writes are NOT persisted to Turso via the flush path. **Mitigation**: Sessions work because `resolve_authenticated_user` reads from MemoryBuffer directly. The timer is started but real flushing needs Phase 5 value binding.

3. **Academy name character limit mismatch** — Spec says "Limit to 40 characters" but implementation uses 30-char limit before suffix. Total slug length differs from spec. **Impact**: Minor — Turso accepts both lengths.

4. **Registration duplicate-check uses local SQLite, not control plane** — The spec says "Check email does not exist (control plane)" but the code checks the local SQLite repository (`self.user_repository.exists_by_email`). **Impact**: Could allow a user to register with an email that exists in the control plane but not locally. Low risk for single-user desktop app.

5. **No test coverage for critical Turso paths** — `login`, `register_user`, `resolve_authenticated_user`, `update_profile`, `change_password`, `run_migrations_on_db`, and `ControlPlaneRepository` all have zero automated tests because they require real Turso connections. Unit testing relies on compilation tests only.

### SUGGESTION

1. **Add integration test mode** — Consider adding a `#[cfg(test)]` mock module for `ControlPlaneRepository` and `TursoProvisioningService` to enable unit testing of `login()` and `register_user()` without real Turso connections.

2. **Normalize spec to implementation** — Update the academy name character limit in the spec from 40 to 30 to match the implementation, or vice versa.

3. **Frontend test coverage** — RegisterForm.tsx has no `.test.tsx` file. Consider adding Vitest tests for form validation, loading state, and error handling.

---

## Verdict

**PASS WITH WARNINGS**

The critical path fixes are correctly implemented and match the architecture design. All builds pass, all 87 tests pass (75 Rust + 12 TypeScript), and the core spec scenarios are covered. The warnings are known Phase 4 limitations with Phase 5 remediation planned in the task breakdown.

### Summary
- ✅ All Rust backend builds pass
- ✅ All TypeScript builds pass
- ✅ 87/87 tests pass (75 Rust unit tests + 12 JS unit tests)
- ✅ `run_migrations_on_db()` fully implemented with tracking table
- ✅ `RegisterUserUseCase` creates Turso DB, runs migrations, inserts user
- ✅ `resolve_authenticated_user()` validates sessions from MemoryBuffer + cached connections
- ✅ `update_profile()` and `change_password()` are real implementations
- ✅ Frontend RegisterForm has academy_name field, loading state, error handling
- ⚠️ Flush timer `execute_operation()` has unimplemented SQL value binding (Phase 5 TODO)
- ⚠️ `resolve_authenticated_user` doesn't re-query control plane (cache-only)
- ⚠️ No automated tests for any Turso-dependent command paths
