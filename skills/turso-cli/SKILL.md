# Skill: turso-cli (Turso Cloud CLI)

Complete command-line reference for managing Turso Cloud databases: installation, authentication, databases, groups, replication, security (RLS/fine-grained), organizations, imports/exports, backups, and more.

**Trigger:** Using the Turso CLI (`turso`), managing databases/groups/organizations, token generation, shell access, branch management, replication, security policies.

## Installation

### macOS / Linux / WSL

```bash
brew install tursodatabase/tap/turso          # Homebrew
curl -fsSL https://get.tur.so | sh            # Install script
```

### Upgrading

```bash
turso update
```

---

## Authentication

```bash
turso auth signup             # Sign up (opens browser)
turso auth login              # Login (opens browser)
turso auth login --headless   # CI/CD, remote, headless environments
turso auth logout             # Remove credentials
turso auth whoami             # Show authenticated user
turso auth token              # Display current API/auth token
```

### Headless Mode (CI/CD)

Use `--headless` flag for environments without a browser (GitHub Actions, Codespaces, SSH).

### Minting API Tokens (for programmatic access)

```bash
turso auth api-tokens mint <name> [--expiration <duration>]    # Create new API token
turso auth api-tokens list                                     # List all API tokens
turso auth api-tokens revoke <token_id>                        # Revoke an API token
```

---

## Database Operations

### Creating Databases

```bash
turso db create [database-name] [--group <group-name>] [flags]
```

| Flag | Description |
|------|-------------|
| `--group <name>` | Create in specific group |
| `--size-limit <bytes>` | Max size (also accepts units: `1mb`, `256mb`, `1gb`) |
| `--enable-extensions` | Enable experimental SQLite extensions |
| `--canary` | Use canary build |
| `--wait` (`-w`) | Wait for DB to be ready |

### Importing Data

```bash
# From SQLite file (.db, not exceeding 2GB)
turso db create my-db --from-file ./path/to/file.db

# From SQL dump file
turso db create my-db --from-dump ./dump.sql

# From SQL dump URL
turso db create my-db --from-dump-url https://example.com/dump.sql

# From CSV file
turso db create my-db --from-csv ./data.csv --csv-table-name users

# Copy from another existing Turso database
turso db create my-db --from-db other-db

# Point-in-time restore (copy from another DB at a specific timestamp)
turso db create my-db --from-db other-db --timestamp "2024-01-15T10:30:00Z"
```

Point-in-time restore requires timestamps in RFC3339 format.

### Listing & Showing Databases

```bash
turso db list                           # List all databases
turso db list --group <name>            # Filter by group
turso db show <name>                    # Show database details
turso db show <name> --url              # Display HTTP API URL
turso db show <name> --http-url         # Display HTTP URL
turso db locations                      # Available deployment locations
```

### Database Shell

```bash
turso db shell my-db                     # Interactive SQLite shell
                                   # Inside shell: .quit to exit
# Shell supports all SQLite dot commands: .tables, .schema, .mode, etc.
```

### Inspecting & Exporting

```bash
turso db inspect <name>        # Analyze database schema/structure
turso db export <name>         # Export database
turso db import <name>         # Import into database
```

### Destorying Databases

```bash
turso db destroy <name>        # Permanently deletes a database
```

---

## Database Token Management

All SDK connections need an auth token scoped to a specific database or group.

```bash
# Create a database-specific token
turso db tokens create <database-name>                    # Full access
turso db tokens create <database-name> --read-only        # Queries only
turso db tokens create <database-name> -p table:actions   # Fine-grained permissions

# Invalidate all tokens for a database
turso db tokens invalidate <database-name>

# Fine-grained permission syntax
-p all:data_read                              # Read from all tables
-p posts:data_add                             # Insert into posts table only
-p users:data_read,data_add                   # Read + insert on users
```

Token options that combine with any scoping level:

| Flag | Description |
|------|-------------|
| `--read-only` | Read queries only, no writes |
| `-p <permission>` | Fine-grained: `table:actions` or `all:data_read` |
| `--expiration <duration>` | Auto-expire (e.g., `7d`, `24h`, `30m`) |

Examples:

```bash
# Read-only token expiring in 7 days
turso db tokens create mydb --read-only --expiration 7d

# Scoped to comments table, read + write, expires in 24h
turso db tokens create mydb -p comments:data_read,data_add --expiration 24h

# Allow attach operation (needed for multi-DB schemas feature)
turso db tokens create mydb --allow-attach
```

---

## Groups & Replication

Groups define where a database is replicated. Each group creates replicas in specified geographic locations.

### Listing & Showing Groups

```bash
turso group list                                    # List all groups
turso group show <group-name>                       # Show group details
```

### Creating Groups

```bash
turso group create <group-name> [locations...]      # Create with replica locations
                                                  # Locations: ams, atx, bhm, fra, gru, hkg, jnb, lhr, pdx, sjo, sin, snd, syd, waw, ord

# Example: US West + EU Europe
turso group create eu-west lhr pdx
```

Available location codes:

| Code | Location | Code | Location |
|------|----------|------|----------|
| `ams` | Amsterdam | `atx` | Austin |
| `bhm` | Birmingham AL | `fra` | Frankfurt |
| `gru` | Sao Paulo | `hkg` | Hong Kong |
| `jnb` | Johannesburg | `lhr` | London |
| `ord` | Chicago | `pdx` | Portland |
| `sjo` | San Jose CR | `sin` | Singapore |
| `snd` | Mumbai | `syd` | Sydney |
| `waw` | Warsaw | | |

### Updating & destroying Groups

```bash
turso group update <group-name> [--add-location <loc> --remove-location <loc>]
turso group destroy <group-name>            # Deletes group and all its replicas
turso group unarchive <group-name>          # Restore inactive group
turso group transfer <group-name> <org-slug>  # Transfer to another org
turso group tokens create <group-name>      # Group-level token (affects all DBs in group)
turso group tokens invalidate <group-name>  # Invalidate group tokens
```

### AWS Migration Commands

```bash
turso group aws-migration info <group-name>     # Check migration readiness
turso group aws-migration start <group-name>    # Begin AWS VPC migration
turso group aws-migration abort <group-name>    # Abort ongoing migration
```

---

## Organizations

```bash
turso org list                                      # List all accessible orgs
turso org create <name>                             # Create new organization
turso org switch <org-slug>                         # Switch active context
turso org billing                                   # View billing status
turso org members list                              # List members
turso org members invite <email>                    # Invite member
turso org members add <user>                         # Add existing user
turso org members rm <user>                         # Remove member
turso org destroy <name>                            # Destroy organization
```

---

## Plans & Billing

```bash
turso plan select <plan>                            # Select a plan
turso plan show                                     # View current plan quotas
turso plan upgrade                                  # Upgrade plan
turso plan overages enable                          # Enable overage charges
turso plan overages disable                         # Disable overages (DB stops accepting writes)
```

---

## Attach-Schema (Multi-Database Schemas — Legacy Feature)

Attach allows reading data from multiple Turso databases within a single session using the `ATTACH DATABASE` pattern.

### Enabling Attaching on a Database

```bash
turso db config attach allow set <database-name>           # Enable
turso db config attach allow clear <database-name>         # Disable
turso db config attach allow show <database-name>          # Show current state
```

### Usage in Shell

```sql
-- In `turso db shell my-db`:
ATTACH 'other-db-id' AS other;
SELECT * FROM other.users;
DETACH other;
```

### Token Permission for Attaching

When creating a token, include the proper privilege to attach:
```bash
turso db tokens create mydb --allow-attach
```

---

## Row-Level Security (RLS) / Database Access Control Rules

Control which database IPs are allowed to connect using IP-based allow rules.

```bash
turso db config allow-rules set <database> <ip-or-cidr>      # Add ip/CIDR rule
turso db config allow-rules show <database>                   # Show current rules
turso db config allow-rules clear <database>                  # Clear all rules
```

Supported formats:
- IPv4 address: `192.168.1.1`
- IPv6 address: `::1`
- CIDR range: `10.0.0.0/8`

---

## Local Development

Run a local libSQL server for development/testing without cloud dependencies.

```bash
turso dev :memory:                           # In-memory dev server
turso dev app.db                             # Local `.db` file dev server
                                           # Visit http://127.0.0.1:8080 to interact
```

For sync-enabled local testing:
```bash
# Start local sync server (local alternative to Turso Cloud)
tursodb :memory: --sync-server 127.0.0.1:8080

# Then configure client with RemoteUrl pointing to http://127.0.0.1:8080
# No AuthToken needed
```

---

## Contact & Feedback

```bash
turso contact feedback <message>               # Submit feedback
turso contact bookmeeting                       # Book a meeting with Turso team
```

---

## Quick Reference: Workflow Patterns

### Pattern 1: Standard Full-Stack App

```bash
# 1️⃣ Set up the database
turso auth login
turso db create myapp
turso db show myapp --url                       # → DATABASE_URL env var
turso db tokens create myapp                    # → AUTH_TOKEN env var

# 2️⃣ Connect your app with environment variables
export TURSO_DATABASE_URL="libsql://myapp-org.turso.io"
export TURSO_AUTH_TOKEN="<token>"

# 3️⃣ Manage production replicas
turso group add-location myapp pdx              # Add West coast replica
turso group remove-location myapp ord           # Remove central US replica
```

### Pattern 2: Multi-Domain Architecture

Each domain gets its own database, connected at runtime via `ATTACH`.

```bash
turso db create users-db
turso db create orders-db
turso db tokens create users-db --allow-attach
turso db tokens create orders-db --allow-attach
# App loads both tokens and uses ATTACH at runtime
```

### Pattern 3: Embedded Replica / Offline

Local-first with push/pull sync to cloud.

```bash
# Client connects locally, pushes when online
const db = connect("./app.db", { turbolink });   // TS example
await db.push();                                  // Push to cloud
await db.pull();                                  // Pull changes
```

---

## Key URLs

- **CLI Docs**: https://docs.turso.tech/cli/introduction
- **Quickstart**: https://docs.turso.tech/quickstart
- **Installation**: https://docs.turso.tech/cli/installation
- **Authentication**: https://docs.turso.tech/cli/authentication
- **Locations**: https://docs.turso.tech/cli/db/locations
- **Platform API**: https://docs.turso.tech/api-reference
