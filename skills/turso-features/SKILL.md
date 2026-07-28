# Turso Features Reference

Vector search, AI embeddings, SQLite extensions, Full-Text Search (FTS), branching, point-in-time recovery, embedded replicas, sync, and more.

**Trigger:** Turso vector search, AI embeddings, FTS, SQLite extensions, branching, point-in-time recovery, data sync, embedded replicas, content caching.

---

## Vector Search (AI & Embeddings)

Turso includes **native vector similarity search** — no extensions required. Perfect for semantic search, RAG workflows, and recommendation systems.

### Overview

1. Create a table with one or more `VECTOR` columns (`FLOAT32`)
2. Insert vector values (binary or text representation)
3. Query using `vector_distance_cosine()` or `vector_distance_l2()`

### Creating Vector Tables

```sql
-- Table with a 1536-dimensional vector column (e.g., OpenAI embeddings)
CREATE TABLE documents (
  id INTEGER PRIMARY KEY,
  content TEXT,
  embedding FLOAT32(1536)  -- Vector column
);

-- Create vector index for fast similarity search
CREATE INDEX idx_documents_embedding
  ON documents (libsql_vector_idx(embedding, 'metric=cosine'));
```

### Querying Vectors

```sql
-- Find the 5 most similar documents to a query vector
SELECT
  id,
  content,
  vector_distance_cosine(embedding, :query_vector) AS distance
FROM documents
ORDER BY distance ASC
LIMIT 5;

-- Using L2 distance instead
SELECT
  id,
  content,
  vector_distance_l2(embedding, :query_vector) AS distance
FROM documents
ORDER BY distance ASC
LIMIT 5;
```

### TypeScript Example (RAG)

```typescript
import { connect } from "@tursodatabase/serverless";
import { OpenAI } from "openai";

const db = connect({
  url: process.env.TURSO_DATABASE_URL,
  authToken: process.env.TURSO_AUTH_TOKEN,
});

// Generate embedding
const openai = new OpenAI({ apiKey: process.env.OPENAI_API_KEY });
const response = await openai.embeddings.create({
  model: "text-embedding-3-small",
  input: "What is machine learning?",
});
const queryVector = response.data[0].embedding;

// Query similar documents
const stmt = await db.prepare(`
  SELECT content, vector_distance_cosine(embedding, ?) AS distance
  FROM documents
  ORDER BY distance ASC
  LIMIT 5
`);
const results = await stmt.all([JSON.stringify(queryVector)]);
```

### Vector Functions

| Function | Description |
|----------|-------------|
| `vector_distance_cosine(a, b)` | Cosine similarity (1 - cosine) |
| `vector_distance_l2(a, b)` | Euclidean (L2) distance |
| `vector_from_json(json)` | Parse JSON array to vector |
| `vector_to_json(vector)` | Vector to JSON string |

### Inserting Vectors

```sql
INSERT INTO documents (content, embedding)
VALUES (
  'Machine learning is a subset of AI...',
  vector_from_json('[0.12, -0.34, 0.56, ...]')  -- 1536 values
);
```

---

## Full-Text Search (FTS)

Turso supports full-text search via Tantivy-powered FTS indexes with scoring and highlighting.

### Creating FTS Indexes

```sql
-- Create a virtual table for FTS
CREATE VIRTUAL TABLE articles_fts USING fts5(
  title,
  content,
  tokenize='unicode61'
);

-- Or attach an FTS index to an existing table
INSERT INTO articles_fts (rowid, title, content)
SELECT id, title, content FROM articles;
```

### Querying with FTS

```sql
-- Basic FTS5 query
SELECT * FROM articles_fts WHERE articles_fts MATCH 'machine learning';

-- With ranking
SELECT
  title,
  snippet(articles_fts, '<mark>', '</mark>', '...', -1, 50) as excerpt,
  rank
FROM articles_fts
WHERE articles_fts MATCH 'machine learning'
ORDER BY rank;
```

### FTS Operators

| Operator | Example | Description |
|----------|---------|-------------|
| AND | `'cat AND dog'` | Both terms required |
| OR | `'cat OR dog'` | Either term |
| NEAR | `NEAR(cat, dog, 3)` | Terms within 3 words |
| prefix | `'mach*'` | Prefix matching |
| Phrase | `'"machine learning"'` | Exact phrase |

---

## SQLite Extensions

Turso supports loading SQLite extensions for additional functionality.

### Enabling Extensions

#### Via CLI (at database creation)
```bash
turso db create mydb --enable-extensions
```

⚠️ This enables extensions for ALL databases in the group.

### Built-in Extensions

Turso ships with several built-in extensions:

| Extension | Description |
|-----------|-------------|
| `vsv` | CSV/TSV virtual table |
| `unicode` | Unicode functions |
| `regexp` | Regular expressions |
| `uuid` | UUID generation |
| `series` | Generate number series |
| `math` | Advanced math functions |
| `stats` | Statistical functions |
| `aggregate` | Custom aggregate functions |

### Loading Extensions at Runtime

```sql
-- Load a built-in extension
SELECT load_extension('vsv');

-- Create a virtual table from CSV
CREATE VIRTUAL TABLE data USING vsv(
  filename='data.csv',
  columns=5,
  header=yes
);

SELECT * FROM data;
```

### Registering Custom C Extensions (Turso Cloud)

Custom `.so` extensions can be registered for production deployments through Turso's platform API.

---

## Branching

Turso supports database branching — create isolated copies of a database for development/testing/snapshotting.

### Creating a Branch

Via CLI:
```bash
turso db create dev-branch --from-db production-db
```

Via Platform API:
```
POST /v1/organizations/:slug/databases
{
  "name": "dev-branch",
  "group": "default",
  "source": { "type": "database", "database": "production-db" }
}
```

### Point-in-Time Branching

```bash
# Branch from a specific timestamp
turso db create snapshot-2024-01-15 \
  --from-db production-db \
  --timestamp "2024-01-15T00:00:00Z"
```

### Branching Use Cases

| Use Case | Approach |
|----------|----------|
| **Feature development** | Branch from prod, work in isolation |
| **Testing migrations** | Branch, apply migration, verify, discard |
| **Data snapshots** | Branch at point-in-time for analysis |
| **Staging previews** | Branch with recent prod data for testing |
| **Blue-green deployments** | Branch during deployment, switch on success |

---

## Point-in-Time Recovery

Turso supports point-in-time recovery (PITR) — restore a database to any previous state.

### Creating a Point-in-Time Copy

```bash
turso db create recovery-copy \
  --from-db production-db \
  --timestamp "2024-01-15T14:30:00Z"
```

Timestamp must be RFC3339 format: `2024-01-15T14:30:00Z`

### Accessing Historical Data

You can query a database as it existed at any point in its history by creating a branch at that timestamp.

### PITR Requirements
- Available within retention period (varies by plan)
- Creates a NEW database copy (doesn't overwrite the original)

---

## Embedded Replicas & Sync

### Embedded Replicas

Traditional embedded replicas:
- Reads are local
- Writes go to cloud primary
- Changes reflected back automatically

```typescript
import { createClient } from "@libsql/client";

const client = createClient({
  url: "file:./local.db",
  syncUrl: "libsql://mydb-org.turso.io",
  authToken: "your-token",
});
```

### Turso Sync (Modern Approach)

New local-first sync engine:
- Both reads AND writes are local
- Explicit `push()` / `pull()` operations
- Conflict resolution built-in

```typescript
import { connect } from "@tursodatabase/sync";

const db = await connect({
  path: "./app.db",
  url: process.env.TURSO_DATABASE_URL,
  authToken: process.env.TURSO_AUTH_TOKEN,
});

// Push local changes to cloud
await db.push();

// Pull cloud changes locally
const changed = await db.pull();
```

### Checkpointing

Compact the local write-ahead log to bound disk usage:

```typescript
await db.checkpoint();
```

### Conflict Resolution

When conflicts arise from concurrent sync:
- **Timestamp-based** — Last write wins
- **Custom resolution** — Via application-level merge logic
- **Per-table strategies** — Configure sync behavior per table

### Local Sync Server (Dev/Testing)

For offline testing without Turso Cloud:
```bash
tursodb :memory: --sync-server 127.0.0.1:8080
```

Then configure clients with:
- `RemoteUrl: "http://127.0.0.1:8080"`
- No `AuthToken` needed locally

---

## Partial Sync

For apps that don't need the entire dataset:

- **Startup**: Open and query a database without downloading the full file
- **On-demand**: Fetch only the pages you need as you query
- **Benefit**: Faster cold starts, lower bandwidth

Enable partial sync in SDK configuration:
```typescript
const db = await connect({
  path: "./app.db",
  url: process.env.TURSO_DATABASE_URL,
  authToken: process.env.TURSO_AUTH_TOKEN,
  // Partially sync: download only what's needed
});
```

---

## Multi-DB Schemas (ATTACH DATABASE)

Access multiple Turso databases in a single connection:

```sql
-- Attach a secondary database
ATTACH 'libsql://other-db.turso.io?authToken=TOKEN' AS other;

-- Query across databases
SELECT u.name, o.total
FROM users u
JOIN other.orders o ON u.id = o.user_id;

-- Detach
DETACH other;
```

### Token Permissions for Attaching
- Create tokens with `--allow-attach` flag
- Both source and target databases must permit attach

⚠️ Deprecated in favor of proper data modeling.

---

## Durability & Encryption

### Durability Guarantees

Turso Cloud provides:
- **Synchronous replication** — Writes confirmed on multiple nodes
- **Automatic failover** — Primary replica failure is transparent
- **WAL durability** — Write-ahead logs survive crashes

### Cloud Encryption (BYOK)

Bring-your-own-key encryption for data at rest:
- Keys managed externally (AWS KMS, GCP KMS, etc.)
- Transparent encryption/decryption at the storage layer
- Compliant with enterprise data governance

---

## Content Caching

Turso supports caching patterns for content-heavy workloads:
- Materialized views for pre-computed results
- SQLite's built-in page cache for hot reads
- Application-level caching for API responses

```sql
-- Materialized view for dashboard data
CREATE MATERIALIZED VIEW dashboard_stats AS
SELECT
  COUNT(*) as total_users,
  SUM(revenue) as total_revenue,
  AVG(age) as avg_age
FROM users;
```

---

## Key URLs

- **Vector Search**: https://docs.turso.tech/guides/vector-search
- **AI & Embeddings**: https://docs.turso.tech/features/ai-and-embeddings
- **FTS**: https://docs.turso.tech/sql-reference/functions/fts
- **Extensions**: https://docs.turso.tech/features/sqlite-extensions
- **Branching**: https://docs.turso.tech/features/branching
- **Point-in-Time Recovery**: https://docs.turso.tech/features/point-in-time-recovery
- **Sync Usage**: https://docs.turso.tech/sync/usage
- **Partial Sync**: https://docs.turso.tech/sync/partial
- **Durability**: https://docs.turso.tech/cloud/durability
- **Cloud Encryption**: https://docs.turso.tech/cloud/encryption
