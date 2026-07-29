# Design: Turso SaaS Migration

## Technical Approach

Transform Academix from a single local SQLite app into a cloud-native SaaS. Every piece of data lives in **Turso cloud** — no local database files. The architecture uses a **write-back cache pattern**: all data operations write to an in-memory buffer for instant response and Turso efficiency. After 15 consecutive minutes without any write activity, the buffer is flushed to Turso via batch writes. The **Control Plane** (mapping users to their database URLs) is also a Turso database, bootstrapped manually by the superadmin.

This minimizes Turso writes, eliminates local disk writes, and keeps reads at RAM speed. The tradeoff is a 15-minute window of potential data loss on crash — acceptable for a single-user desktop SaaS.

## Architecture Decisions

### Decision: Data Storage — In-Memory Write Buffer with 15-min Flush

**Choice**: A `MemoryBuffer` holds all pending writes in a `HashMap<String, Vec<PendingWrite>>` (keyed by SQL statement/user). A background timer monitors write activity. After 15 minutes of inactivity (no writes), all buffered writes are flushed to Turso in batch. On app close, the buffer flushes immediately.

| Operation | Behavior |
|-----------|----------|
| **Write** (create/update/delete) | Goes to in-memory buffer immediately. Timer resets. |
| **Read** | Checks in-memory buffer first (read-through), then Turso |
| **Flush trigger** | 15 min of inactivity since last write |
| **App close** | Immediate flush (blocking, with 5s timeout) |
| **App crash** | Data loss within the 15-min window (acceptable for desktop) |

**Alternatives considered**:
- Every write hits Turso immediately — too many network calls, higher latency for every operation
- Local SQLite + sync — writes files to disk, user explicitly wants no local files
- Embedded Replicas — requires local filesystem, user wants cloud-only

**Rationale**: The user explicitly wants:
1. No local storage
2. In-memory operations to save writes
3. Batched Turso writes after inactivity

This pattern is ideal for a single-user desktop app where the user is the only writer — no concurrency conflicts, the 15-min window is safe, and the UX is instant.

### Decision: Control Plane — Turso Database (Bootstrapped Manually)

**Choice**: The control plane is a Turso database named `academix-control-plane`, created manually by the superadmin via `turso-cli`. The superadmin's Turso API token and the control plane DB URL are configured as environment variables.

**Bootstrapping steps for superadmin:**
```bash
turso db create academix-control-plane
turso db show academix-control-plane --url  # Save as CONTROL_PLANE_DB_URL
turso db tokens create academix-control-plane  # Save as CONTROL_PLANE_DB_TOKEN
```

On app startup, the app connects to this Turso DB (via libsql) and reads/writes the `user_databases` mapping table. The superadmin's own credentials are seeded into the control plane on first use.

**Alternatives considered**:
- Local SQLite (was in v1 of this design) — user wants NO local files
- Env vars only — not scalable, no query capabilities

**Rationale**: The user wants everything in Turso cloud. The superadmin creates this one DB manually (one-time setup). After that, all user registrations create new databases via Platform API and record them in the control plane. The circular dependency concern (need Turso to resolve Turso) is resolved by the one-time manual bootstrap.

### Decision: Repository Pattern — MemoryBuffer Backend (not DbConnection)

**Choice**: Instead of the `DbConnection` trait wrapping rusqlite/libsql, repositories now write to a shared `MemoryBuffer` and read from it (read-through to Turso on cache miss). The `MemoryBuffer` replaces the `SqlitePool` at the infrastructure layer.

**Rationale**: Since ALL writes buffer in memory and ALL reads check memory first, the repository pattern shifts from "execute SQL on a database" to "buffer operations and flush as batch SQL." The turso flush step translates buffered operations into batch SQL executed against the user's Turso DB.

### Decision: Registration — In-Request DB Creation (Async/Await)

**Choice**: When a user registers, the backend:
1. Calls Turso Platform API to create the database (async/await, user waits 2-5s)
2. Runs migrations against the new DB
3. Creates the user account record (in-memory buffer, queued for flush)
4. Saves mapping to control plane Turso DB
5. Returns success

The user sees a loading spinner "Creando tu academia..." for 2-5 seconds while the Turso DB is provisioned.

**Alternatives considered**: Background job (user can't log in immediately), pre-provision pool (wasteful)

**Rationale**: The user said "la creacion de la cuenta y la creacion de la base de datos sera asincrona y en la misma peticion se hara" — it's async within the request (the Tauri command is async, awaits the platform API call), but the user waits for completion. This is the right tradeoff: DB creation takes 2-5s once, and the user gets a clean onboarding experience.

### Decision: Session Storage — In-Memory (Flushed to User's Turso DB)

**Choice**: Sessions are stored in the in-memory buffer like all other data. On flush, they're written to the user's `sessions` table in their Turso DB. Token validation reads from memory first, then Turso.

**Rationale**: Sessions are data like any other — they follow the same in-memory → flush pattern. Since the 15-min inactivity window applies, active users with frequent writes will flush sessions promptly. On app restart, sessions are re-read from Turso.

### Decision: Port Interface Change — Add `academy_name` to Registration

**Choice**: Add `academy_name` field to `RegisterUserRequest` DTO and the frontend form. The academy name is used to derive the Turso database slug (e.g., `academy-music-school`).

**Alternatives considered**: Ask for academy name later (adds friction), generate name from user's email (impersonal)

**Rationale**: The academy name is essential for the superadmin to identify which database belongs to which tenant. It's a natural part of the registration flow for an academic management SaaS.

## Data Flow

### MemoryBuffer Architecture
```
┌─────────────────────────────────────────────────┐
│                  AppState                        │
│                                                   │
│  ┌───────────────────────────────────────────┐   │
│  │         MemoryBuffer                       │   │
│  │  ┌─────────────────────┐                   │   │
│  │  │ pending_writes:     │  HashMap<         │   │
│  │  │  user_id → Vec<Op>  │  buffered ops     │   │
│  │  └─────────────────────┘                   │   │
│  │  ┌─────────────────────┐                   │   │
│  │  │ cached_reads:       │  HashMap<         │   │
│  │  │  user_id → Entities │  read-through     │   │
│  │  └─────────────────────┘                   │   │
│  │  ┌─────────────────────┐                   │   │
│  │  │ last_write_at:      │  Instant          │   │
│  │  │ flush_timer:        │  15-min timeout   │   │
│  │  └─────────────────────┘                   │   │
│  └───────────────────────────────────────────┘   │
│                                                   │
│  ┌───────────────────────────────────────────┐   │
│  │  ConnectionManager                        │   │
│  │  user_id → (libsql::Db, UserDbMapping)    │   │
│  │  (lazy init, cached for session)          │   │
│  └───────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
         │
         ▼  (flush after 15min inactivity)
┌─────────────────────────────────────────────────┐
│               Turso Cloud                        │
│  ┌──────────────┐  ┌─────────────────────┐      │
│  │ Control Plane│  │ academy-{slug} DB   │      │
│  │ (managed)    │  │ (per user, auto)    │      │
│  └──────────────┘  └─────────────────────┘      │
└─────────────────────────────────────────────────┘
```

### Registration Flow
```
RegisterForm (academy_name, name, email, password)
  │
  ▼
invoke("register_user", { academy_name, name, email, password })
  │ async/await
  ▼
RegisterUserUseCase.execute(request)
  │
  ├── 1. Validate email + password
  ├── 2. Check if email exists (query control plane Turso DB)
  ├── 3. Hash password
  ├── 4. Generate DB slug from academy_name
  ├── 5. POST /v1/organizations/academix/databases { name: "academy-{slug}" }
  │     └── Response: { Name, Hostname, ... }
  ├── 6. Generate auth token for the new DB
  ├── 7. Connect to new DB via libsql (remote only, no local file)
  ├── 8. Run all 18 migrations against new DB (direct write, not buffered)
  ├── 9. Save user + academy record →Buffer (queued for flush)
  ├── 10. Save mapping to control plane Turso DB (direct write)
  └── 11. Return RegisterUserResponse { success, user }

Frontend: Loading spinner "Creando tu academia..." → success → redirect to login
```

### Login Flow
```
LoginForm (email, password)
  │
  ▼
invoke("login", { email, password })
  │
  ▼
LoginCommand
  │
  ├── 1. Look up email in CONTROL PLANE (Turso DB, direct query)
  │     └── Find: user_id, db_url, db_token
  ├── 2. Get or create libsql connection from ConnectionManager (remote only)
  ├── 3. Authenticate against USER'S Turso DB (direct query)
  ├── 4. Create session → Buffer (queued for flush)
  └── 5. Return LoginResponse { token, user, expires_at }

Subsequent commands:
  ├── 1. Validate session: check Buffer first, then Turso
  └── 2. Execute command → read from Buffer/Turso, write to Buffer
```

### Write-Back Flush Flow
```
User performs writes (create student, record payment, etc.)
  │
  ├── Write goes to MemoryBuffer.pending_writes[user_id]
  ├── Timer resets: last_write_at = now
  └── Response returns immediately (no network wait)

... 15 minutes pass without any write ...
  │
  ├── Background timer fires
  ├── Flush cycle starts:
  │   ├── For each user_id with pending writes:
  │   │   ├── Get the user's Turso DB connection
  │   │   ├── Build batch SQL from buffered operations
  │   │   ├── Execute batch against user's Turso DB
  │   │   └── Clear user's pending writes on success
  │   └── Clear cached reads (stale)
  └── Timer waits for next activity

App closes:
  ├── Immediate flush (with 5-second timeout)
  ├── Write all pending buffers to Turso
  └── App exits cleanly
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| **NEW** — `src-tauri/src/infrastructure/turso/mod.rs` | Create | Module exports for Turso infrastructure |
| **NEW** — `src-tauri/src/infrastructure/turso/provisioning.rs` | Create | `TursoProvisioningService` — Platform API client |
| **NEW** — `src-tauri/src/infrastructure/turso/control_plane.rs` | Create | `ControlPlaneRepository` — user→DB mapping (Turso-backed) |
| **NEW** — `src-tauri/src/infrastructure/turso/connection_manager.rs` | Create | `ConnectionManager` — remote-only libsql connections cache |
| **NEW** — `src-tauri/src/infrastructure/turso/memory_buffer.rs` | Create | `MemoryBuffer` — in-memory write cache with 15-min flush timer |
| **NEW** — `src-tauri/src/infrastructure/turso/flush_timer.rs` | Create | Background timer: monitors idle time, triggers flush |
| `src-tauri/Cargo.toml` | Modify | Add `libsql` + `reqwest` deps |
| `src-tauri/src/lib.rs` | Modify | Init Turso infrastructure + MemoryBuffer; seed control plane admin |
| `src-tauri/src/infrastructure/database/pool.rs` | Remove | No longer needed — no local SQLite |
| `src-tauri/src/infrastructure/database/mod.rs` | Modify | Remove pool re-export |
| `src-tauri/src/infrastructure/mod.rs` | Modify | Export turso module |
| `src-tauri/src/commands/register.rs` | Modify | Add Turso DB creation + academy_name in flow |
| `src-tauri/src/commands/auth.rs` | Modify | Login resolves via control plane Turso DB; MemoryBuffer for sessions |
| `src-tauri/src/commands/*.rs` (all commands) | Modify | Write to MemoryBuffer instead of SqlitePool |
| `src-tauri/src/application/dto/user.rs` | Modify | `RegisterUserRequest` gains `academy_name: String` |
| `src-tauri/src/application/ports/user.rs` | Modify | Repositories reference MemoryBuffer instead of pool |
| `src-tauri/src/application/ports/session.rs` | No change | Interface stays the same |
| `src-tauri/src/application/use_cases/register.rs` | Modify | Add Turso provisioning call + academy_name + MemoryBuffer |
| `src-tauri/src/application/use_cases/auth.rs` | Modify | Validate sessions via MemoryBuffer + Turso fallback |
| `src-tauri/src/infrastructure/repositories/sqlite/*.rs` (11 files) | Modify | Rewrite to use `MemoryBuffer` instead of rusqlite/SqlitePool |
| `src-tauri/src/infrastructure/repositories/mod.rs` | Modify | Update re-exports for memory-backed repos |
| `src/features/auth/components/RegisterForm.tsx` | Modify | Add `academy_name` input field + validation |
| `src-tauri/Cargo.toml` | Modify | Remove `rusqlite`, remove `printpdf` if unused |
| `src-tauri/src/domain/entities/` | No change | Domain entities stay unchanged |
| `src-tauri/src/application/ports/*.rs` | No change | Port interfaces stay unchanged |

## Interfaces / Contracts

### `RegisterUserRequest` DTO (Modified)
```rust
#[derive(Debug, Deserialize)]
pub struct RegisterUserRequest {
    pub academy_name: String,  // NEW
    pub name: String,
    pub email: String,
    pub password: String,
}
```

### `ControlPlaneRepository`
```rust
/// Maps user accounts to their Turso database connection details.
/// Backed by a Turso database (libsql), bootstrapped manually by superadmin.
pub struct ControlPlaneRepository {
    db: libsql::Database,
}

impl ControlPlaneRepository {
    pub async fn new(db_url: &str, db_token: &str) -> Self;
    pub async fn save_user_db(&self, mapping: UserDbMapping) -> Result<(), DomainError>;
    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserDbMapping>, DomainError>;
    pub async fn find_by_user_id(&self, user_id: &str) -> Result<Option<UserDbMapping>, DomainError>;
    pub async fn list_all_databases(&self) -> Result<Vec<UserDbMapping>, DomainError>;
}

pub struct UserDbMapping {
    pub user_id: String,
    pub email: String,
    pub academy_name: String,
    pub db_url: String,
    pub db_token: String,
    pub org: String,
    pub created_at: String,
}
```

### `TursoProvisioningService`
```rust
pub struct TursoProvisioningService {
    api_token: String,         // superadmin Platform API token
    org: String,               // Turso organization slug
    client: reqwest::Client,
}

impl TursoProvisioningService {
    pub fn new(api_token: String, org: String) -> Self;
    
    /// Create a new database in the organization.
    /// POST /v1/organizations/{org}/databases
    pub async fn create_database(&self, name: &str) -> Result<DatabaseInfo, ProvisioningError>;
    
    /// Generate an auth token for the database.
    /// POST /v1/organizations/{org}/databases/{name}/auth/tokens
    pub async fn create_auth_token(&self, db_name: &str) -> Result<String, ProvisioningError>;
    
    /// List all databases in the organization (for superadmin).
    /// GET /v1/organizations/{org}/databases
    pub async fn list_databases(&self) -> Result<Vec<DatabaseInfo>, ProvisioningError>;
    
    /// Delete a database (when user deletes account).
    /// DELETE /v1/organizations/{org}/databases/{name}
    pub async fn delete_database(&self, db_name: &str) -> Result<(), ProvisioningError>;
}

pub struct DatabaseInfo {
    pub name: String,
    pub hostname: String,   // e.g. academy-music-school.turso.io
}
```

### `MemoryBuffer`
```rust
/// In-memory write buffer with 15-minute inactivity flush.
/// All writes go through this buffer. Reads check buffer first, then Turso.
pub struct MemoryBuffer {
    pending_writes: HashMap<String, Vec<BufferedOperation>>,
    cached_entities: HashMap<String, Vec<Entity>>,
    last_write_at: Arc<Mutex<Instant>>,
}

impl MemoryBuffer {
    pub fn new() -> Self;
    
    /// Buffer a write operation (create/update/delete).
    pub fn buffer_write(&mut self, user_id: &str, op: BufferedOperation);
    
    /// Read from buffer or Turso (read-through).
    pub async fn read<T>(&self, user_id: &str, key: &str) -> Option<T>;
    
    /// Flush all pending writes to Turso.
    /// Called by the 15-min timer or on app close.
    pub async fn flush(&self, cm: &ConnectionManager) -> Result<(), FlushError>;
    
    /// Get time since last write (for timer).
    pub fn idle_duration(&self) -> Duration;
}

pub enum BufferedOperation {
    Insert { table: String, data: HashMap<String, Value> },
    Update { table: String, id: String, data: HashMap<String, Value> },
    Delete { table: String, id: String },
}
```

### `ConnectionManager`
```rust
/// Manages connections to users' Turso databases.
/// Remote-only connections (no local files).
pub struct ConnectionManager {
    connections: HashMap<String, (libsql::Database, UserDbMapping)>,
}

impl ConnectionManager {
    pub fn new() -> Self;
    
    pub async fn resolve_by_email(&self, cp: &ControlPlaneRepository, email: &str)
        -> Result<&libsql::Database, AppError>;
    
    pub async fn resolve_by_user_id(&self, user_id: &str)
        -> Result<&libsql::Database, AppError>;
    
    pub async fn register_connection(&mut self, mapping: UserDbMapping)
        -> Result<(), AppError>;
    
    pub async fn run_migrations(&self, db: &libsql::Database) -> Result<(), AppError>;
}
```

### Repository Interface Change
```rust
pub trait UserRepository: Send + Sync {
    // BEFORE: fn pool(&self) -> Arc<SqlitePool>;
    // AFTER:  buffer and flush through MemoryBuffer
    
    fn find_by_id(&self, id: &str) -> Result<Option<User>, DomainError>;
    fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;
    fn save(&self, user: &User) -> Result<(), DomainError>;
    fn update(&self, user: &User) -> Result<(), DomainError>;
    fn delete(&self, id: &str) -> Result<(), DomainError>;
    fn find_all(&self) -> Result<Vec<User>, DomainError>;
    fn exists_by_email(&self, email: &Email) -> Result<bool, DomainError>;
}

/// Concrete implementations hold a reference to MemoryBuffer
pub struct MemoryBackedUserRepository {
    buffer: Arc<Mutex<MemoryBuffer>>,
    user_id: String,  // which user's data this repo serves
}
```

### Control Plane Schema (Turso DB)
The control plane is a Turso database. It has one table for user→DB mappings:

```sql
CREATE TABLE IF NOT EXISTS user_databases (
    user_id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    academy_name TEXT NOT NULL,
    db_url TEXT NOT NULL,
    db_token TEXT NOT NULL,
    org TEXT NOT NULL DEFAULT 'academix',
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_user_databases_email ON user_databases(email);

-- Superadmin user for control plane login
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'Admin',
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    token TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit — `MemoryBuffer` | Buffer writes, read-through, flush builds correct SQL | Pure unit tests (no network), assert buffered operations |
| Unit — `flush_timer` | Timer fires after 15min idle, resets on write, flushes on close | Mock clock or test with short intervals |
| Unit — `TursoProvisioningService` | API calls, error handling, retry | Mock HTTP server (`wiremock`) |
| Unit — `ControlPlaneRepository` | CRUD on user→DB mappings against Turso | Integration test with test Turso DB |
| Unit — `ConnectionManager` | Caching, resolution, connection reuse | Mock dependencies |
| Integration — Registration | Full flow: validate → create DB → run migrations → store mapping | Needs Turso test org (smoke test) |
| Integration — Flush | Buffer 10 operations → trigger flush → verify in Turso DB | Integration test |
| Frontend | RegisterForm with academy_name, validation, loading state | Vitest + jsdom |
| E2E | Registration + login + CRUD + flush verification | Playwright with Turso test DB |

## Migration / Rollout

The migration follows **6 sequential phases**, each independently revertible. Since the architecture changes fundamentally (remove local SQLite), there is a hard cutover point in Phase 4.

### Phase 1: Foundation (non-breaking)
1. Add `libsql` and `reqwest` deps to `Cargo.toml`
2. Create `MemoryBuffer` with in-memory HashMap + flush logic
3. Create `flush_timer` background task (15-min inactivity → flush)
4. Create `TursoProvisioningService` (Platform API client)
5. Create `ConnectionManager` (remote-only libsql connections cache)
6. **Verification**: `cargo check` passes, Turso module compiles
7. **Rollback**: Revert commits — no behavioral change

### Phase 2: Control Plane (Turso-backed)
1. Superadmin creates `academix-control-plane` DB manually via `turso-cli`
2. Superadmin configures env vars: `CONTROL_PLANE_DB_URL`, `CONTROL_PLANE_DB_TOKEN`, `TURSO_API_TOKEN`
3. Create `ControlPlaneRepository` backed by Turso (not local SQLite)
4. Seed superadmin user in control plane on first startup
5. **Verification**: `cargo check` passes, control plane connects to Turso
6. **Rollback**: Revert — local SQLite app still works

### Phase 3: Registration (Turso DB creation)
1. Add `academy_name` field — `RegisterUserRequest` DTO + `RegisterForm.tsx`
2. Modify `RegisterUserUseCase`: validate → create Turso DB via Platform API → run migrations → save mapping in control plane → buffer user record
3. Add loading state "Creando tu academia..." with spinner
4. **Verification**: New registration creates isolated Turso DB
5. **Rollback**: Revert commits — fall back to local SQLite registration

### Phase 4: Login + Buffer Integration (HARD CUTOVER)
1. Modify `login` command: check control plane → resolve user's Turso DB → authenticate
2. Modify `AuthService` to work with MemoryBuffer + Turso fallback
3. Route all authenticated commands through MemoryBuffer instead of local pool
4. Remove `SqlitePool`, remove rusqlite dependency
5. **Verification**: Login works, all CRUD operations buffer to memory, flush on idle
6. **Rollback**: Full revert to last phase — app can't work without a DB provider

### Phase 5: Repository Rewrite (MemoryBuffer-backed)
1. Rewrite `MemoryBackedUserRepository` — implements `UserRepository` trait via `MemoryBuffer`
2. Rewrite `MemoryBackedSessionRepository`
3. Repeat for all 11 repositories
4. Remove old `Sqlite*Repository` files
5. **Verification**: All CRUD operations flow through MemoryBuffer, flush works end-to-end
6. **Rollback**: Revert individual repo rewrites (one at a time)

### Phase 6: Superadmin
1. Add `list_client_databases` Tauri command
2. Add role check (Admin role from control plane)
3. Register command in `lib.rs`
4. **Verification**: Superadmin can see all client databases

## Open Questions

- [ ] **Crash recovery**: If the app crashes before the 15-min flush, data is lost. Is this acceptable, or should we add a periodic intermediate flush (e.g., every 5 min regardless of activity)?
- [ ] **Token rotation**: How long do Turso auth tokens last before expiring? Need a background refresh mechanism. Check Turso docs.
- [ ] **Flush on read**: If the user reads data that's been buffered but not flushed, should we flush before reading from Turso to ensure consistency?
- [ ] **App close flush timeout**: If the app has 500 buffered operations, how long should we wait for flush (currently 5s)?

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| **Data loss on crash** — 15-min window of unsaved data | Medium | Acceptable for single-user desktop; add periodic intermediate flush if concerning |
| App close takes >5s due to large buffer flush | Medium | Progressively flush; if timeout, save buffer to a temp file for retry on next startup |
| Turso Platform API rate limits during batch registration | Low | 500 req/min per org on Hobby plan; fine for registration volume |
| Migrating 11 repos to MemoryBuffer simultaneously | High | Rewrite one repo at a time, verify each before next |
| libsql Rust crate API compatibility | Low | libsql is SQLite-compatible; main change is removing rusqlite entirely |
| Superadmin bootstrap requires manual turso-cli steps | Low | Document clearly in README; one-time setup |
