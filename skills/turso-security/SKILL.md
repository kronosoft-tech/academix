# Turso Security & Authorization

Complete reference for Turso authentication, authorization, fine-grained permissions, JWT tokens, JWKS, Row-Level Security (RLS), and access control.

**Trigger:** Turso auth tokens, database access control, RLS policies, JWT-based permissions, external auth providers, fine-grained table permissions, token scoping, multi-tenant security.

---

## Authentication Model

Turso uses **scoped, JWT-based tokens** for database access. Every token can be restricted by:
- Database (or all databases in a group)
- Permission level (read, write, attach)
- Table-level and action-level restrictions
- Time-limited expiration (e.g., `7d`, `24h`)

---

## Token Scoping Levels (from broadest to narrowest)

| Level | Scope | How to Create |
|-------|-------|---------------|
| **API Token** | Platform API access (manage databases, groups) | `turso auth api-tokens mint <name>` |
| **Group Token** | Access all databases in a group | `turso group tokens create <group>` |
| **Database Token** | Access a single database | `turso db tokens create <database>` |
| **Read-only** | Queries only, no writes | Add `--read-only` flag |
| **Table + Action** | Specific tables and operations | Add `-p <table>:<actions>` flag |
| **Time-limited** | Auto-expires after duration | Add `--expiration <duration>` flag |

All levels can be combined:

```bash
# Read-only, single DB, expires in 7 days
turso db tokens create mydb --read-only --expiration 7d

# Only read from all tables, insert into comments
turso db tokens create mydb \
  -p all:data_read \
  -p comments:data_add
```

---

## Using Tokens in SDKs

All tokens are passed as `authToken` when creating a database client:

```typescript
import { createClient } from "@tursodatabase/serverless";

const db = createClient({
  url: "<your-database-url>",
  authToken: "<your-token>",
});
```

Get your database URL:
```bash
turso db show <database-name> --url
```

---

## Platform Tokens via CLI

### Database Tokens

```bash
# Full access
turso db tokens create my-db

# Read-only
turso db tokens create my-db --read-only

# Table-specific: read and add
turso db tokens create my-db -p users:data_read,data_add

# Table-specific: select, insert, update only on orders table
turso db tokens create my-db -p orders:select,insert,update

# Expire after 24 hours
turso db tokens create my-db --expiration 24h

# Allow ATTACH DATABASE operations
turso db tokens create my-db --allow-attach

# Invalidate all tokens for a database
turso db tokens invalidate my-db
```

### Group Tokens

```bash
# Creates a token that works across all databases in the group
turso group tokens create my-group

# Invalidate group tokens
turso group tokens invalidate my-group
```

### API Tokens (Platform API)

```bash
# Create a new API token
turso auth api-tokens mint my-token-name

# List all API tokens
turso auth api-tokens list

# Revoke an API token
turso auth api-tokens revoke my-token-id
```

---

## Platform API: Token Endpoints

### Create Token (User-level)

```
POST /v1/auth/api-tokens
```

Returns a new API token for the user. Tokens can be minted at three levels (increasing restriction):
1. **Unrestricted** — Full platform access
2. **Organization-scoped** — Limited to one org
3. **Group-scoped** — Limited to one group's databases

### Validate Token

```
POST /v1/auth/validate
```

Validates whether a token is active and returns its scope.

### Database Token Generation

```
POST /v1/organizations/:slug/databases/:name/auth/tokens
```

Generates a database-specific auth token.

### Revoke Tokens

```
DELETE /v1/auth/api-tokens/:token_id
```

---

## External Auth Providers (JWKS)

Let your authentication provider (Auth0, Clerk, Supabase Auth, custom) issue tokens using JWKS (JSON Web Key Sets).

### How it works

1. Configure your external auth provider to sign JWTs
2. Turso reads your JWKS endpoint to verify token signatures
3. Applications use the externally-issued JWT directly as the auth token
4. Tokens support the same fine-grained permissions as native tokens

### JWT Claims

External JWTs must include claims compatible with Turso's permission model:
- `nbf` / `iat` — Not before / Issued at
- `exp` — Expiration (Turso enforces these)
- Custom permissions claims for fine-grained access

### Setting up JWKS

Configure your database or group to trust an external JWKS URL:
```bash
# Configure external auth for database
turso db config auth set <database> --jwks-url <url>

# View current JWKS settings
turso db config auth show <database>

# Clear JWKS config
turso db config auth clear <database>
```

---

## Fine-Grained Permissions (Table + Action Level)

Turso tokens can scope to specific tables and specific actions within those tables.

### Permission Syntax

```
-p <table>:<actions>
```

Available actions:
| Action | Description |
|--------|-------------|
| `read` / `data_read` | SELECT queries |
| `write` / `data_write` | INSERT, UPDATE, DELETE |
| `all` | All operations |

### Permission Examples

```bash
# Read-only: all tables
turso db tokens create mydb -p all:data_read

# Read + insert on comments table only
turso db tokens create mydb -p comments:data_read,data_add

# Read users, full CRUD on posts
turso db tokens create mydb \
  -p users:data_read \
  -p posts:data_read,data_add,data_write

# Read-only token for the entire database
turso db tokens create mydb --read-only
```

### Using Fine-Grained Permissions in Apps

```typescript
// This token can only READ and INSERT into comments
const db = createClient({
  url: process.env.TURSO_DATABASE_URL,
  authToken: process.env.TURSO_COMMENTS_TOKEN,
});

// ✅ This works:
await db.execute("SELECT * FROM comments");
await db.execute("INSERT INTO comments (text) VALUES ('hello')");

// ❌ This fails: permission denied
await db.execute("DELETE FROM comments WHERE id = 1");
await db.execute("UPDATE posts SET ...");  // Not in permissions
```

---

## Row-Level Security (RLS) Policies

RLS enforces data access at the row level based on the authenticated user's identity or context.

### Enabling RLS

RLS policies use SQLite expressions to filter which rows are visible for specific operations.

### Creating RLS Policies

```sql
-- Example: Users can only see their own posts
CREATE POLICY user_posts_select
  ON posts
  FOR SELECT
  USING (author_id = current_user_id());

-- Example: Users can only update their own posts
CREATE POLICY user_posts_update
  ON posts
  FOR UPDATE
  USING (author_id = current_user_id())
  WITH CHECK (author_id = current_user_id());

-- Example: Admins can read all posts
CREATE POLICY admin_read_all
  ON posts
  FOR SELECT
  USING (is_admin() = true);
```

### Policy Types

| Type | When it applies |
|------|----------------|
| `FOR SELECT` | Which rows can be read |
| `FOR INSERT` | Conditions for new rows (WITH CHECK) |
| `FOR UPDATE` | Which rows can be updated (USING), and allowed new values (WITH CHECK) |
| `FOR DELETE` | Which rows can be deleted |
| `FOR ALL` | Applies to all operations |

### Policy Clauses

- `USING(expression)` — Filter for existing rows
- `WITH CHECK(expression)` — Validation for new/modified rows

### Disabling RLS

```sql
-- Disable RLS on a table
ALTER TABLE posts DISABLE ROW LEVEL SECURITY;

-- Enable RLS on a table
ALTER TABLE posts ENABLE ROW LEVEL SECURITY;
```

---

## Multi-Tenant RLS Pattern

### Tenant-scoped data with RLS

```sql
-- Create a tenants table
CREATE TABLE tenants (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL
);

-- Example resource table with tenant ownership
CREATE TABLE documents (
  id INTEGER PRIMARY KEY,
  title TEXT,
  content TEXT,
  tenant_id INTEGER NOT NULL REFERENCES tenants(id),
  created_by INTEGER NOT NULL
);

-- RLS: Only access rows matching current tenant
CREATE POLICY tenant_documents_select
  ON documents
  FOR SELECT
  USING (tenant_id = current_tenant_id());

CREATE POLICY tenant_documents_insert
  ON documents
  FOR INSERT
  WITH CHECK (tenant_id = current_tenant_id());

CREATE POLICY tenant_documents_update
  ON documents
  FOR UPDATE
  USING (tenant_id = current_tenant_id())
  WITH CHECK (tenant_id = current_tenant_id());

CREATE POLICY tenant_documents_delete
  ON documents
  FOR DELETE
  USING (tenant_id = current_tenant_id());
```

### Setting Context Variables

In application code, set the current tenant/user before executing queries:

```typescript
// Set tenant context before your queries
await db.execute("SELECT set_config('app.current_tenant_id', $1, false)", [tenantId]);

// Now RLS policies automatically filter all queries
const docs = await db.prepare("SELECT * FROM documents").all();
// Only returns documents where tenant_id matches the session variable
```

---

## Database Access Allow Rules (IP-based)

Restrict who can connect to your database at the network level.

```bash
# Show current allow rules
turso db config allow-rules show <database>

# Add allowed IP addresses or CIDR ranges
turso db config allow-rules set <database> "203.0.113.50"
turso db config allow-rules set <database> "10.0.0.0/8"

# Clear all allow rules (allow anyone with valid token)
turso db config allow-rules clear <database>
```

Supported formats:
- IPv4: `192.168.1.100`
- IPv6: `::1` or `2001:db8::/32`
- CIDR: `10.0.0.0/8` or `172.16.0.0/12`

---

## Private Endpoints (AWS VPC)

Configure private endpoints for your Turso Database on AWS VPC:

```bash
# Retrieve database configuration including VPC endpoint IDs
curl -H "Authorization: Bearer $TURSO_API_TOKEN" \
  "https://api.turso.tech/v1/organizations/$ORG/databases/$DB/config"
```

Database config includes `allowedVpcEndpointIds` — only connections from listed VPC endpoints are accepted.

---

## Encryption (Bring Your Own Key)

### Local Encryption

Encrypted databases can't be opened as standard SQLite files.

```typescript
import { connect } from "@tursodatabase/database";

const db = await connect("encrypted.db", {
  encryption: {
    cipher: "aegis256",
    hexkey: "b1bbfda4f589dc9daaf004fe21111e00dc00c98237102f5c7002a5669fc76327",
  },
});
```

### Cloud Encryption

Turso Cloud databases support bring-your-own-key (BYOK) encryption for data at rest. Configure via Platform API.

---

## Security Best Practices

1. **Use database-specific tokens** — Never use unrestricted API tokens for database access
2. **Set token expiration** — Always use `--expiration <duration>` for temporary access
3. **Apply least privilege** — Scope tokens to specific tables and actions
4. **Use read-only tokens** for public-facing read endpoints
5. **Invalidate tokens** when employees leave or access changes
6. **Allow rules** — Restrict by IP in production environments
7. **Separate tokens per environment** — Dev, staging, production each get their own tokens
8. **Use RLS for multi-tenant** — Enforce row-level isolation at the database layer
9. **Audit tokens regularly** — `turso auth api-tokens list`
10. **Use JWKS for SSO** — Centralize auth through your identity provider

---

## Key URLs

- **SDK Authorization**: https://docs.turso.tech/sdk/authorization
- **Platform Tokens**: https://docs.turso.tech/sdk/authorization/tokens
- **Fine-grained Permissions**: https://docs.turso.tech/sdk/authorization/fine-grained-permissions
- **External Auth (JWKS)**: https://docs.turso.tech/sdk/authorization/jwks
- **Private Endpoints**: https://docs.turso.tech/cloud/private-endpoints
- **Cloud Encryption**: https://docs.turso.tech/cloud/encryption
