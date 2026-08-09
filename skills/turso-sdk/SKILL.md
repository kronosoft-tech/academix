# Turso SDKs — All Languages

Use when connecting applications to Turso databases: TypeScript, Python, Go, Rust/Tauri, and more. Covers package selection, quickstarts, sync, embedded replicas, transactions, and ORM integrations.

## Choosing the Right Package

### TypeScript

| Package | Use Case | Sync? | Engine | Concurrency |
|---------|----------|-------|--------|-------------|
| `@tursodatabase/database` | Local/Embedded (Node.js, Electron, IoT) | — | Turso Database | MVCC |
| `@tursodatabase/sync` | Local + Cloud Sync (recommended for offline apps) | push/pull | Turso Database | MVCC |
| `@tursodatabase/serverless` | Remote (servers, serverless, edge) | — | fetch only | — |
| `@libsql/client` | ORM support (Drizzle, Prisma), legacy codebases | Embedded Replicas | libSQL | Single-writer |
| `@libsql/client/web` | Edge runtimes (Cloudflare, Vercel Edge) | — | fetch only | — |

### Python

| Package | Use Case | Sync? | Interface |
|---------|----------|-------|-----------|
| `pyturso` | Local/Embedded | push/pull | sqlite3-compatible |
| `libsql` | Remote/over-the-wire, stateless servers | — | Python DB API 2.0 |
| `libsql.sync` (via turso.sync) | Local with sync | push/pull | sqlite3-compatible |

### Go

| Package | Use Case | Sync? | Interface |
|---------|----------|-------|-----------|
| `tursogo` | Local + Cloud Sync | Push/Pull | database/sql driver |
| `go-libsql` | Remote/over-the-wire, serverless | — | database/sql driver |
| `go-libsql` with Embedded Replicas | Local reads, cloud writes | sync() | database/sql driver |

### Rust / Tauri

| Package | Use Case |
|---------|----------|
| `libsql` | General Rust apps |
| `libsql` with remote sync | Tauri apps with embedded replicas |
| `tursodatabase/database` | Turso Database (newer) |

---

## TypeScript Quickstarts

### Recommended: `@tursodatabase/database` (Local/Embedded)

```bash
npm install @tursodatabase/database
```

```typescript
import { connect } from "@tursodatabase/database";

const db = await connect("app.db");

// Create table
await db.prepare(`
  CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL
  )
`).run();

// Insert
await db.prepare("INSERT INTO users (username) VALUES (?)").run("alice");

// Query
const stmt = db.prepare("SELECT * FROM users");
const users = await stmt.all();
console.log(users);
```

In-memory:
```typescript
const db = await connect(":memory:");
```

### Recommended: `@tursodatabase/sync` (Local + Cloud Sync)

```bash
npm install @tursodatabase/sync
```

```typescript
import { connect } from "@tursodatabase/sync";

const db = await connect({
  path: "./app.db",
  url: process.env.TURSO_DATABASE_URL,
  authToken: process.env.TURSO_AUTH_TOKEN,
});

// Local writes
await db.exec("INSERT INTO users (username) VALUES ('bob')");

// Push to cloud, pull changes from cloud
await db.push();
const changed = await db.pull();

// Checkpoint WAL to bound disk usage
await db.checkpoint();

// Stats
const stats = await db.stats();
```

### Recommended: `@tursodatabase/serverless` (Remote/Serverless)

```bash
npm install @tursodatabase/serverless
```

```typescript
import { connect } from "@tursodatabase/serverless";

const conn = connect({
  url: process.env.TURSO_DATABASE_URL,
  authToken: process.env.TURSO_AUTH_TOKEN,
});

// Prepared statements
const stmt = await conn.prepare("SELECT * FROM users WHERE id = ?");
const row = await stmt.get([1]);
```

`@libsql/client` compat:
```typescript
import { createClient } from "@tursodatabase/serverless/compat";

const client = createClient({
  url: process.env.TURSO_DATABASE_URL,
  authToken: process.env.TURSO_AUTH_TOKEN,
});
```

### ORM Integration: `@libsql/client`

```bash
npm install @libsql/client
```

```typescript
import { createClient } from "@libsql/client";

export const turso = createClient({
  url: process.env.TURSO_DATABASE_URL,
  authToken: process.env.TURSO_AUTH_TOKEN,
});

await turso.execute("SELECT * FROM users");
```

Runtime environments:
- Node.js 12+
- Deno
- CloudFlare Workers
- Netlify & Vercel Edge Functions

---

## Embedded Replicas (TypeScript)

For offline writes, bidirectional sync, multi-writer:

```typescript
import { createClient } from "@libsql/client";

const client = createClient({
  url: "file:path/to/db-file.db",
  syncUrl: "libsql://[databaseName]-[organizationSlug].turso.io",
  authToken: "...",
  syncInterval: 60,  // Auto-sync every 60s
});

// Manual sync
await client.sync();
```

For new projects needing sync, use `@tursodatabase/sync` — explicit `push()` / `pull()` with CDC.

---

## Transactions (@libsql/client)

### Transaction Modes

| Mode | SQLite Command | Description |
|------|----------------|-------------|
| `write` | `BEGIN IMMEDIATE` | Read + write; forwarded to primary on replicas |
| `read` | `BEGIN TRANSACTION READONLY` | Select only; parallel on replicas |
| `deferred` | `BEGIN DEFERRED` | Start read, upgrade to write if needed |

### Interactive Transactions

```typescript
const transaction = await client.transaction("write");
try {
  await transaction.execute({
    sql: "UPDATE accounts SET balance = balance - ? WHERE userId = ?",
    args: [500, "user123"],
  });
  await transaction.commit();
} catch (e) {
  await transaction.rollback();
}
```

Interactive transactions lock the DB for writing with a 5-second timeout.

---

## Python Quickstarts

### Recommended: `pyturso` (Local/Embedded)

```bash
uv add pyturso
# or
pip install pyturso
```

```python
import turso

db = turso.connect("app.db")
# In-memory: db = turso.connect(":memory:")

db.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)")
db.execute("INSERT INTO users (name) VALUES (?)", ("Alice",))
db.commit()

for row in db.execute("SELECT * FROM users"):
    print(row)
```

### Python Sync

```python
import turso.sync

db = turso.sync.connect(
    "app.db",
    remote_url=os.environ["TURSO_DATABASE_URL"],
    auth_token=os.environ["TURSO_AUTH_TOKEN"],
)

db.push()   # Push to cloud
db.pull()   # Pull from cloud
```

### Remote Access (libsql)

```bash
pip install libsql
```

```python
import libsql

conn = libsql.connect(
    database=os.environ["TURSO_DATABASE_URL"],
    auth_token=os.environ["TURSO_AUTH_TOKEN"],
)

conn.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)")
conn.commit()

rows = conn.execute("SELECT * FROM users").fetchall()
```

---

## Go Quickstarts

### Recommended: `tursogo` (Local + Cloud Sync)

```bash
go get turso.tech/database/tursogo
```

```go
package main

import (
    "database/sql"
    "fmt"
    _ "turso.tech/database/tursogo"
)

func main() {
    db, _ := sql.Open("turso", "app.db")
    defer db.Close()

    db.Exec(`CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL
    )`)

    db.Exec("INSERT INTO users (name) VALUES (?)", "Alice")

    rows, _ := db.Query("SELECT * FROM users")
    defer rows.Close()
    for rows.Next() {
        var id int
        var name string
        rows.Scan(&id, &name)
        fmt.Printf("User: %d %s\n", id, name)
    }
}
```

### Go Sync

```go
package main

import (
    "context"
    "os"
    turso "turso.tech/database/tursogo"
)

func main() {
    ctx := context.Background()

    syncDb, _ := turso.NewTursoSyncDb(ctx, turso.TursoSyncDbConfig{
        Path:      "app.db",
        RemoteUrl: os.Getenv("TURSO_DATABASE_URL"),
        AuthToken: os.Getenv("TURSO_AUTH_TOKEN"),
    })

    db, _ := syncDb.Connect(ctx)
    defer db.Close()

    db.ExecContext(ctx, "INSERT INTO users (name) VALUES (?)", "Bob")

    syncDb.Push(ctx)   // Push to cloud
    syncDb.Pull(ctx)   // Pull from cloud
}
```

### Remote Access (go-libsql)

```bash
go get github.com/tursodatabase/libsql-client-go/libsql
```

```go
package main

import (
    "database/sql"
    "fmt"
    "os"
    _ "github.com/tursodatabase/libsql-client-go/libsql"
)

func main() {
    url := os.Getenv("TURSO_DATABASE_URL") + "?authToken=" + os.Getenv("TURSO_AUTH_TOKEN")
    db, _ := sql.Open("libsql", url)
    defer db.Close()

    rows, _ := db.Query("SELECT * FROM users")
    defer rows.Close()
    for rows.Next() {
        var id int
        var name string
        rows.Scan(&id, &name)
        fmt.Printf("User: %d %s\n", id, name)
    }
}
```

---

## Rust / Tauri Integration

### Tauri + Turso Setup

In `src-tauri/Cargo.toml`:
```toml
[dependencies]
libsql = { git = "https://github.com/tursodatabase/libsql" }
serde_json = "1.0"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
dotenvy = "0.15.7"
```

In `src-tauri/.env`:
```
TURSO_SYNC_URL="libsql://mydb-orgslug.turso.io"
TURSO_AUTH_TOKEN="your-token"
DB_PATH=./my-data.db
```

Rust code:
```rust
use dotenvy::dotenv;
use libsql::{params, Database};
use serde::{Deserialize, Serialize};
use std::env;

#[tauri::command]
async fn get_all_items() -> Result<Vec<Item>, Error> {
    dotenv().expect(".env file not found");

    let db_path = env::var("DB_PATH").unwrap();
    let sync_url = env::var("TURSO_SYNC_URL").unwrap();
    let auth_token = env::var("TURSO_AUTH_TOKEN").unwrap();

    let db = Database::open_with_remote_sync(db_path, sync_url, auth_token).await?;
    let conn = db.connect()?;

    let mut results = conn.query("SELECT * FROM table_name", ()).await?;

    let mut items: Vec<Item> = Vec::new();
    while let Some(row) = results.next()? {
        items.push(Item {
            id: row.get(0)?,
            foo: row.get(1)?,
            bar: row.get(2)?,
        });
    }

    Ok(items)
}
```

---

## ORM Integrations

### Drizzle + Turso (TypeScript)

```typescript
import { drizzle } from "drizzle-orm/libsql";
import { createClient } from "@libsql/client";

const client = createClient({
  url: process.env.TURSO_DATABASE_URL!,
  authToken: process.env.TURSO_AUTH_TOKEN!,
});

export const db = drizzle(client);
```

### Drizzle + `@tursodatabase/database` (beta)

```typescript
import { drizzle } from "drizzle-orm/tursodatabase";
import { connect } from "@tursodatabase/database";

const db = await connect("app.db");
export const orm = drizzle(db);
```

### Prisma + Turso

In `prisma/schema.prisma`:
```prisma
datasource db {
  provider = "sqlite"
  url      = env("TURSO_DATABASE_URL")
}
```

Use `@libsql/client`. Prisma treats Turso as SQLite-compatible.

### SQLAlchemy + Turso (Python)

```python
from sqlalchemy import create_engine

# For libsql-python remote connections
engine = create_engine(
    "sqlite+libsql://",
    connect_args={
        "database": os.environ["TURSO_DATABASE_URL"],
        "auth_token": os.environ["TURSO_AUTH_TOKEN"],
    }
)
```

### Ruby on Rails + Turso

In `config/database.yml`:
```yaml
development:
  adapter: sqlite3
  database: storage/development.sqlite3
```

### Drift + Flutter/Dart

```dart
import 'package:drift_libsql/drift_libsql.dart';
import 'package:libsql_dart/libsql_dart.dart';

final client = LibsqlClient(
  url: 'libsql://your-db.turso.io',
  authToken: 'your-token',
);
final database = LibsqlDatabase(client);
```

### Toasty + Rust (async ORM)

Toasty integrates with Turso for async database access following Tokio patterns.

### Doctrine DBAL + PHP

Connect to Turso as a SQLite-compatible driver through Doctrine DBAL.

---

## Encryption

### TypeScript (@tursodatabase/database)

```typescript
import { connect } from "@tursodatabase/database";

const db = await connect("encrypted.db", {
  encryption: {
    cipher: "aegis256",
    hexkey: "b1bbfda4f589dc9...",
  },
});
```

Supported ciphers: `aegis256`, `aegis256x2`, `aegis128l`, `aegis128x2`, `aegis128x4`, `aes256gcm`, `aes128gcm`.

### TypeScript (@libsql/client)

```typescript
const db = createClient({
  url: "file:encrypted.db",
  encryptionKey: process.env.ENCRYPTION_KEY,
});
```

---

## Attach Database (Multi-DB Sessions)

For reading across multiple databases in one connection:

```typescript
import { createClient } from "@libsql/client";

const client = createClient({
  url: process.env.TURSO_DATABASE_URL,
  authToken: process.env.TURSO_AUTH_TOKEN,
});

const txn = await db.transaction("read");
await txn.execute('ATTACH "<database-id>" AS attached');
const rs = await txn.execute("SELECT * FROM attached.users");
```

⚠️ Requires:
1. Attaching allowed on the target database (`turso db config attach allow set`)
2. Token with `--allow-attach` permission

---

## SQL over HTTP

For environments where SDKs aren't available (Edge Functions, serverless):

```bash
npm install @tursodatabase/serverless
```

```typescript
const response = await fetch(`${TURSO_DATABASE_URL}/v2/pipeline`, {
  method: "POST",
  headers: {
    "Authorization": `Bearer ${TURSO_AUTH_TOKEN}`,
    "Content-Type": "application/json",
  },
  body: JSON.stringify({
    baton: "",
    request: [
      { type: "execute", stmt: { sql: "SELECT * FROM users" } }
    ],
  }),
});
```

---

## Sync Concepts

### push() / pull() (Turso Sync)
- Local reads and writes against a local `.db` file
- `push()` sends local WAL to cloud
- `pull()` fetches remote changes
- Conflict resolution: Turso uses logical change-data-capture
- Checkpoint: Compacts WAL to bound disk usage

### Embedded Replicas (libsql)
- Reads served locally, writes forwarded to cloud primary
- Changes reflected back to replica automatically
- Single-writer model (writes go to cloud)

---

## Key URLs

- **Turso SDK Intro**: https://docs.turso.tech/sdk/introduction
- **TypeScript Reference**: https://docs.turso.tech/sdk/ts/reference
- **Python Reference**: https://docs.turso.tech/sdk/python/reference
- **Go Reference**: https://docs.turso.tech/sdk/go/reference
- **Rust Reference**: https://docs.turso.tech/sdk/rust/reference
- **Tauri Guide**: https://docs.turso.tech/sdk/rust/guides/tauri
- **Sync Usage**: https://docs.turso.tech/sync/usage