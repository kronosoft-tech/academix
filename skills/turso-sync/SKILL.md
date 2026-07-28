# Turso Sync & Replication

Complete reference for local-first sync, embedded replicas, conflict resolution, checkpointing, partial sync, and replication management.

**Trigger:** Turso sync, embedded replicas, local-first database, push/pull sync, conflict resolution, replication groups, multi-region deployment, sync server.

---

## Sync Architecture

Turso provides two sync mechanisms:

1. **Turso Sync** (Modern) — `@tursodatabase/sync`, `pyturso.sync`, `tursogo` with sync
   - Both reads AND writes are local
   - Explicit `push()` and `pull()` operations
   - CDC-based (logical change-data-capture)

2. **Embedded Replicas** (Traditional) — `@libsql/client` with `syncUrl`
   - Reads local, writes to cloud primary
   - Automatic propagation back to replica
   - Page-frame based sync

---

## Turso Sync (New)

### TypeScript

```typescript
import { connect } from "@tursodatabase/sync";

const db = await connect({
  path: "./app.db",
  url: process.env.TURSO_DATABASE_URL,
  authToken: process.env.TURSO_AUTH_TOKEN,
});

// First run: bootstrap local DB from remote automatically

// Push local changes to cloud
await db.push();

// Pull remote changes
const changed = await db.pull();
```

### Python

```python
import turso.sync

db = turso.sync.connect(
    "app.db",
    remote_url=os.environ["TURSO_DATABASE_URL"],
    auth_token=os.environ["TURSO_AUTH_TOKEN"],
)

# Push and pull
db.push()
db.pull()
```

### Go

```go
syncDb, _ := turso.NewTursoSyncDb(ctx, turso.TursoSyncDbConfig{
    Path:      "app.db",
    RemoteUrl: os.Getenv("TURSO_DATABASE_URL"),
    AuthToken: os.Getenv("TURSO_AUTH_TOKEN"),
})

db, _ := syncDb.Connect(ctx)

syncDb.Push(ctx)  // Push
syncDb.Pull(ctx)  // Pull
```

---

## Sync Operations

### Push

Sends local writes (WAL entries) to the Turso Cloud primary.

```typescript
await db.push();
```

- Only pushes un-synced local changes
- Returns immediately after sending
- Idempotent — safe to call multiple times

### Pull

Fetches remote changes and applies them locally.

```typescript
const changed = await db.pull();
```

Returns information about what changed:
```typescript
{
  revision: number,          // New revision number
  framesSynced: number,      // Number of frames applied
}
```

### Checkpoint

Compacts the local WAL file to bound disk usage.

```typescript
await db.checkpoint();
```

Run this periodically to prevent WAL from growing unboundedly.

### Stats

Get sync statistics:

```typescript
const stats = await db.stats();
console.log({
  cdcOperations: stats.cdcOperations,
  mainWalSize: stats.mainWalSize,
  networkReceivedBytes: stats.networkReceivedBytes,
  networkSentBytes: stats.networkSentBytes,
  lastPullUnixTime: stats.lastPullUnixTime,
  lastPushUnixTime: stats.lastPushUnixTime,
  revision: stats.revision,
});
```

---

## Sync Workflow Patterns

### Pattern 1: Periodic Sync

```typescript
// Push every 30s, pull every 10s
setInterval(async () => {
  await db.push();
}, 30_000);

setInterval(async () => {
  const changed = await db.pull();
  if (changed.framesSynced > 0) {
    console.log(`Synced ${changed.framesSynced} frames`);
  }
}, 10_000);
```

### Pattern 2: Event-Driven Sync

```typescript
// Push after every user action
async function handleUserAction(action: UserAction) {
  await db.exec(action.sql, action.params);
  await db.push();  // Immediately sync
}

// Pull on app focus / network reconnect
document.addEventListener("visibilitychange", async () => {
  if (document.visibilityState === "visible") {
    await db.pull();
  }
});
```

### Pattern 3: Background Sync (Service Worker)

```typescript
// In a service worker or background task
navigator.serviceWorker.addEventListener("message", async (event) => {
  if (event.data.type === "SYNC") {
    await db.push();
    const changed = await db.pull();
    event.source.postMessage({ type: "SYNCED", changed });
  }
});
```

---

## Conflict Resolution

When multiple clients sync concurrently, conflicts may arise. Turso resolves them using:

### Timestamp-Based Resolution
- **Last write wins** — Most recent modification takes precedence
- Timestamps come from the server (authoritative)

### Application-Level Resolution
For complex conflicts, implement merge logic:

```typescript
const changed = await db.pull();
if (changed.hasConflicts) {
  // Handle conflicts:
  // - Notify user of conflicting changes
  // - Merge data programmatically
  // - Log conflict for reconciliation
}
```

### Conflict Types

| Type | Description | Default Resolution |
|------|-------------|-------------------|
| **Row update conflict** | Same row modified on two clients | Last write wins |
| **Insert conflict** | Same row inserted on two clients | Last write wins |
| **Delete conflict** | Row deleted locally, updated remotely | Delete wins |
| **Schema conflict** | Different schema changes | May require manual intervention |

---

## Embedded Replicas (libsql)

### TypeScript Setup

```typescript
import { createClient } from "@libsql/client";

const client = createClient({
  url: "file:path/to/local.db",
  syncUrl: "libsql://mydb-org.turso.io",
  authToken: process.env.TURSO_AUTH_TOKEN,
});
```

### Manual Sync

```typescript
await client.sync();
```

### Periodic Sync

```typescript
const client = createClient({
  url: "file:path/to/local.db",
  syncUrl: "libsql://mydb-org.turso.io",
  authToken: process.env.TURSO_AUTH_TOKEN,
  syncInterval: 60,     // Auto-sync every 60 seconds
});
```

### Encryption with Embedded Replicas

```typescript
const client = createClient({
  url: "file:encrypted.db",
  syncUrl: "libsql://mydb-org.turso.io",
  authToken: process.env.TURSO_AUTH_TOKEN,
  encryptionKey: process.env.ENCRYPTION_KEY,
});
```

### When to Use Embedded Replicas
- Stateless server environments where CDC is needed
- Writing goes to cloud, reads served locally (single-writer model)
- Existing `@libsql/client` codebase
- Need ORM compatibility beyond Drizzle (e.g., Prisma)

### When to Use Turso Sync
- New projects
- Both reads AND writes need to be offline-capable
- Multi-writer convergence
- Better sync performance (CDC vs page frames)

---

## Local Sync Server (Dev/Testing)

For development WITHOUT requiring a Turso Cloud account:

### Install Turso Database CLI

```bash
# Use tursodb as local sync server
curl -sSfL https://get.turso.tech | sh
```

### Start Local Server

```bash
# In-memory sync server
tursodb :memory: --sync-server 127.0.0.1:8080

# Persistent local sync server
tursodb ./local-sync.db --sync-server 127.0.0.1:8080
```

### Configure Client

```go
// Go
syncDb, _ := turso.NewTursoSyncDb(ctx, turso.TursoSyncDbConfig{
    Path:      "app.db",
    RemoteUrl: "http://127.0.0.1:8080",
    // No AuthToken needed for local server
})
```

```typescript
// TypeScript
const db = await connect({
  path: "./app.db",
  url: "http://127.0.0.1:8080",
  authToken: "",  // Not needed for local
});
```

---

## Partial Sync

For large databases, sync only what you need:

- **Faster cold starts** — Open the database before full download completes
- **Lower bandwidth** — Fetch only queried pages
- **Automatic** — Enabled by default in Turso Sync

When the app first opens, the sync engine:
1. Initializes connection and downloads metadata
2. Allows reads/writes immediately
3. Background syncs data pages on demand
4. Continues syncing remaining pages in background

This is particularly useful for:
- Mobile apps with large databases
- Desktop apps with cold-start requirements
- Offline-first apps that don't need full data immediately

---

## Replication Management

### Creating Replicas via CLI

```bash
# Add a replica location to an existing group
turso group update my-group --add-location hkg

# Remove a replica location from a group
turso group update my-group --remove-location waw
```

### Multi-Region Groups

```bash
# Create a group with replicas in multiple regions
turso group create global lhr pdx hkg sjo

# Now databases in this group sync across all 4 locations
```

### Replica Types

| Type | Description | Write Location |
|------|-------------|----------------|
| **Primary** | Accepts reads and writes | This location |
| **Replica** | Reads only, forwarded writes | Primary |

### Performance Considerations

- Reads from nearest replica = lowest latency
- Writes always forwarded to primary = higher latency
- Use embedded replicas for apps needing local writes

---

## Key URLs

- **Sync Usage**: https://docs.turso.tech/sync/usage
- **Conflict Resolution**: https://docs.turso.tech/sync/conflict-resolution
- **Checkpoint**: https://docs.turso.tech/sync/checkpoint
- **Partial Sync**: https://docs.turso.tech/sync/partial
- **Local Sync Server**: https://docs.turso.tech/sync/local-sync-server
- **Embedded Replicas Intro**: https://docs.turso.tech/features/embedded-replicas
