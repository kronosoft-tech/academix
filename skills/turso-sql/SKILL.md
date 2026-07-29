# Turso SQL Reference

Complete reference for libSQL/Turso SQL dialect: data types, statements, functions, extensions, pragmas, and SQLite compatibility.

**Trigger:** Turso SQL queries, libSQL syntax, SQLite compatibility, SQL functions, Turso data types, vector SQL, FTS SQL, JSON functions, PRAGMA statements, SQL statements (CREATE, SELECT, INSERT, UPDATE, DELETE).

---

## Data Types

Turso uses SQLite's dynamic type system with enhancements:

### Core Storage Classes

| Class | Description | Example |
|-------|-------------|---------|
| `INTEGER` | Signed integers | `42`, `-7` |
| `REAL` | Floating point | `3.14`, `-0.5` |
| `TEXT` | UTF-8 strings | `'hello'` |
| `BLOB` | Binary data | `x'48454c4c4f'` |
| `NULL` | Absence of value | `NULL` |

### STRICT Tables

Enforce type checking on column values:

```sql
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  email TEXT NOT NULL UNIQUE,
  age INTEGER
) STRICT;
```

In STRICT mode, inserting incompatible types raises an error instead of converting.

### Custom Types (Experimental)

Turso supports user-defined types with validation:

```sql
CREATE TYPE email AS TEXT CHECK (VALUE LIKE '%_@__%.__%');

CREATE TABLE accounts (
  id INTEGER PRIMARY KEY,
  contact email  -- Must match email format
);
```

### Vectors

```sql
CREATE TABLE embeddings (
  id INTEGER PRIMARY KEY,
  vector FLOAT32(1536)  -- 1536-dimensional vector
);
```

---

## Core SQL Statements

### CREATE TABLE

```sql
CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  email TEXT UNIQUE NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  role TEXT CHECK (role IN ('admin', 'user', 'viewer')) DEFAULT 'user'
);
```

### CREATE INDEX

```sql
-- Single column index
CREATE INDEX idx_users_email ON users(email);

-- Composite index
CREATE INDEX idx_users_role_created ON users(role, created_at);

-- Partial index
CREATE INDEX idx_active_users ON users(email) WHERE role = 'active';
```

### CREATE VIEW

```sql
CREATE VIEW user_summary AS
SELECT
  u.id,
  u.name,
  COUNT(o.id) AS order_count,
  SUM(o.total) AS total_spent
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
GROUP BY u.id;
```

### CREATE VIRTUAL TABLE (FTS5)

```sql
CREATE VIRTUAL TABLE articles_fts USING fts5(
  title,
  content,
  tokenize='unicode61'
);
```

### ATTACH / DETACH

```sql
-- Attach another database
ATTACH 'path/to/other.db' AS other;

-- Query across databases
SELECT * FROM main.users u
JOIN other.orders o ON u.id = o.user_id;

-- Detach
DETACH other;
```

When attaching a Turso Cloud database:
```sql
ATTACH 'libsql://other-db.turso.io' AS other
  USING token 'your-auth-token';
```

### ALTER TABLE

```sql
-- Rename table
ALTER TABLE old_name RENAME TO new_name;

-- Add column
ALTER TABLE users ADD COLUMN phone TEXT;

-- Rename column
ALTER TABLE users RENAME COLUMN phone TO mobile;

-- Drop column
ALTER TABLE users DROP COLUMN mobile;
```

### INSERT

```sql
-- Single row
INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com');

-- Multiple rows
INSERT INTO users (name, email) VALUES
  ('Alice', 'alice@example.com'),
  ('Bob', 'bob@example.com');

-- From SELECT
INSERT INTO archive_users SELECT * FROM users WHERE inactive = 1;
```

### UPSERT

```sql
INSERT INTO users (email, name)
VALUES ('alice@example.com', 'Alice Updated')
ON CONFLICT(email) DO UPDATE SET name = excluded.name, updated_at = CURRENT_TIMESTAMP;

-- Or do nothing on conflict
INSERT INTO users (email, name)
VALUES ('alice@example.com', 'Alice')
ON CONFLICT(email) DO NOTHING;
```

### SELECT

```sql
-- Basic query
SELECT id, name, email FROM users WHERE active = 1 ORDER BY name;

-- With JOIN
SELECT u.name, COUNT(o.id) AS orders
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
GROUP BY u.id
HAVING orders > 5;

-- With LIMIT/OFFSET
SELECT * FROM users ORDER BY created_at DESC LIMIT 20 OFFSET 40;

-- With subquery
SELECT * FROM products
WHERE price > (SELECT AVG(price) FROM products);
```

### UPDATE

```sql
-- Simple update
UPDATE users SET email = 'new@example.com' WHERE id = 1;

-- With JOIN (via FROM clause)
UPDATE orders
SET status = 'cancelled'
FROM users
WHERE orders.user_id = users.id AND users.suspended = 1;
```

### DELETE

```sql
-- Simple delete
DELETE FROM users WHERE id = 1;

-- With subquery
DELETE FROM orders WHERE user_id IN (SELECT id FROM suspended_users);
```

### TRANSACTIONS

```sql
BEGIN TRANSACTION;

UPDATE accounts SET balance = balance - 100 WHERE id = 1;
UPDATE accounts SET balance = balance + 100 WHERE id = 2;

COMMIT;

-- Or rollback:
-- ROLLBACK;
```

Transaction modes:
| Mode | Description |
|------|-------------|
| `DEFERRED` (default) | Lock acquired on first write |
| `IMMEDIATE` | Exclusive lock acquired immediately |
| `EXCLUSIVE` | Prevents all other connections |

---

## Vector Functions

### Similarity Search

```sql
-- Cosine distance
SELECT id, content,
  vector_distance_cosine(embedding, :query_vector) AS distance
FROM documents
ORDER BY distance ASC
LIMIT 10;

-- L2 distance
SELECT id, content,
  vector_distance_l2(embedding, :query_vector) AS distance
FROM documents
ORDER BY distance ASC
LIMIT 10;
```

### Vector Utilities

```sql
-- Create vector from JSON array
vector_from_json('[0.1, 0.2, 0.3, ...]')

-- Convert vector to JSON
vector_to_json(embedding)

-- Create vector from blob
vector_from_blob(data)
```

---

## JSON Functions

```sql
-- Extract value
SELECT json_extract(data, '$.user.name') FROM records;

-- Extract array element
SELECT json_extract(data, '$.tags[0]') FROM records;

-- Create JSON object
SELECT json_object('id', 1, 'name', 'Alice');

-- Create JSON array
SELECT json_array('one', 'two', 'three');

-- Check if valid JSON
SELECT json_valid('{"key": "value"}');  -- Returns 1

-- Merge JSON objects
SELECT json_patch('{"a": 1}', '{"b": 2}');  -- {"a":1,"b":2}
```

### JSON Table

```sql
-- Query JSON as a virtual table
SELECT value FROM json_each('[1, 2, 3, 4]');

SELECT key, value FROM json_each('{"a": 1, "b": 2}');
```

---

## Full-Text Search Functions

```sql
-- FTS5 MATCH query
SELECT * FROM articles_fts
WHERE articles_fts MATCH 'machine learning';

-- With ranking
SELECT title, snippet(articles_fts) AS excerpt
FROM articles_fts
WHERE articles_fts MATCH 'tutorial'
ORDER BY rank;

-- Snippet with custom markers
SELECT snippet(articles_fts, '<mark>', '</mark>', '...', -1, 50)
FROM articles_fts
WHERE articles_fts MATCH 'turso';
```

---

## Date & Time Functions

```sql
-- Current time
SELECT datetime('now');

-- Format date
SELECT strftime('%Y-%m-%d %H:%M:%S', 'now');

-- Date arithmetic
SELECT date('now', '+30 days');
SELECT date('2024-01-15', '-1 month');

-- Parse date
SELECT julianday('2024-01-15');

-- Duration
SELECT julianday('now') - julianday('2024-01-01');
```

---

## Aggregate Functions

```sql
SELECT
  COUNT(*) AS total,
  AVG(price) AS avg_price,
  SUM(total) AS revenue,
  MAX(created_at) AS latest,
  MIN(price) AS cheapest,
  GROUP_CONCAT(name, ', ') AS names
FROM products;
```

---

## Window Functions

```sql
SELECT
  name,
  department,
  salary,
  RANK() OVER (PARTITION BY department ORDER BY salary DESC) as rank,
  AVG(salary) OVER (PARTITION BY department) as dept_avg,
  SUM(salary) OVER (ORDER BY hire_date) as running_total
FROM employees;
```

---

## Math Functions

```sql
SELECT
  abs(-42),           -- Absolute value
  round(3.14159, 2),  -- Round to 2 decimals
  max(1, 2, 3),       -- Max of values
  min(1, 2, 3),       -- Min of values
  random(),           -- Random integer
  pi(),               -- 3.14159265358979
  sqrt(16),           -- Square root
  sin(0.5),           -- Sine
  cos(0.5),           -- Cosine
  log(100),           -- Natural log
  log10(100),         -- Base-10 log
  exp(1);             -- e^1
```

---

## String Functions

```sql
SELECT
  length('hello'),           -- 5
  lower('HELLO'),            -- 'hello'
  upper('hello'),            -- 'HELLO'
  trim('  hello  '),         -- 'hello'
  ltrim('  hello  '),        -- 'hello  '
  rtrim('  hello  '),        -- '  hello'
  replace('hello world', 'world', 'Turso'),  -- 'hello Turso'
  substr('hello world', 7),  -- 'world'
  substr('hello world', 1, 5),  -- 'hello'
  instr('hello world', 'world'),  -- 7
  like('%world%', 'hello world');  -- 1 (true)
```

---

## PRAGMA Statements

```sql
-- Check database integrity
PRAGMA integrity_check;

-- Set journal mode
PRAGMA journal_mode = WAL;

-- Set synchronous mode
PRAGMA synchronous = NORMAL;

-- Check foreign keys
PRAGMA foreign_key_check;
PRAGMA foreign_keys = ON;

-- Table info
PRAGMA table_info(users);

-- Index list
PRAGMA index_list(users);

-- Database size
PRAGMA page_count;
PRAGMA page_size;
```

---

## SQLite Compatibility

Turso (libSQL) maintains full backward compatibility with SQLite:

| Feature | Supported |
|---------|-----------|
| Core SQL (CREATE, SELECT, INSERT, UPDATE, DELETE) | ✅ |
| Transactions | ✅ |
| Indexes | ✅ |
| Views | ✅ |
| Triggers | ✅ |
| FTS5 | ✅ (with enhancements) |
| JSON1 | ✅ (with enhancements) |
| RTREE | ✅ |
| VACUUM | ✅ |
| ATTACH/DETACH | ✅ |
| User-defined functions | ✅ |
| Concurrent writes | ✅ (Turso enhancement) |
| Vector search | ✅ (Turso enhancement) |
| Custom types | ✅ (experimental, Turso enhancement) |
| STRICT mode | ✅ (Turso enhancement) |

### Known Differences from SQLite

- Turso supports concurrent writes via MVCC (SQLite is single-writer)
- Vector functions are Turso-specific
- Some experimental features (custom types) are not in SQLite
- Cloud deployments use distributed architecture (not raw file access)

---

## Shell Commands

In `turso db shell`:

```sqlite
.tables                          -- List all tables
.schema                          -- Show all schemas
.schema tablename                -- Show schema for specific table
.mode column                     -- Column output mode
.mode csv                        -- CSV output mode
.headers on                      -- Show column headers
.dump                            -- Export entire database as SQL
.import file.csv tablename       -- Import CSV into table
.explain                         -- Show query execution plan
.quit                            -- Exit shell
.help                            -- Show all shell commands
```

---

## Key URLs

- **SQL Compatibility**: https://docs.turso.tech/sql-reference/compatibility
- **Data Types**: https://docs.turso.tech/sql-reference/data-types
- **Functions**: https://docs.turso.tech/sql-reference/functions/
- **PRAGMA**: https://docs.turso.tech/sql-reference/pragmas
- **Statements**: https://docs.turso.tech/sql-reference/statements/
- **Extensions**: https://docs.turso.tech/sql-reference/extensions
- **Vector Functions**: https://docs.turso.tech/sql-reference/functions/vector
- **FTS Functions**: https://docs.turso.tech/sql-reference/functions/fts
