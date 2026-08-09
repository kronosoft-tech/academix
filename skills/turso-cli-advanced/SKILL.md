# Turso CLI Advanced Reference

Complete reference for every Turso CLI command including flags, examples, and output format.

**Trigger:** Turso CLI usage, database management, group operations, token management, org management, plan selection.

---

## Installation

### macOS
```bash
brew install tursodatabase/tap/turso
```

### Linux
```bash
curl -sSfL https://get.turso.tech/install.sh | bash
```

### Windows (WSL)
```bash
curl -sSfL https://get.turso.tech/install.sh | bash
```

### Update
```bash
turso update
```

### Headless Mode
```bash
turso --help                              # Show all available commands
turso auth login --headless               # CI/CD environments, no browser
```

---

## Auth Commands

### `turso auth login`
Authenticate with your Turso account.

| Flag | Description |
|------|-------------|
| `--headless` | For CI/CD or environments without a browser |

### `turso auth signup`
Register a new Turso account (opens browser or use `--headless`).

### `turso auth logout`
Remove stored credentials from local machine.

### `turso auth token`
Display the current API token being used for authentication.

| Flag | Description |
|------|-------------|
| `--plain` | Show only the token value (no formatting) |

### `turso auth whoami`
Show the currently authenticated user.

### `turso auth api-tokens list`
List all platform API tokens.

### `turso auth api-tokens mint <name>`
Create a new platform API token.

| Flag | Description |
|------|-------------|
| `--expiration <duration>` | Token expiration (e.g., `24h`, `7d`, `30d`) |

### `turso auth api-tokens revoke <token_id>`
Revoke a platform API token.

| Flag | Description |
|------|-------------|
| `--plain` | Output token ID only |

---

## Database Commands (`turso db`)

### `turso db create [database-name]`
Create a new database.

| Flag | Description |
|------|-------------|
| `--group <name>` | Create in specific group |
| `--size-limit <bytes>` | Max database size (e.g., `1gb`, `512mb`) |
| `--enable-extensions` | Enable experimental SQLite extensions |
| `--canary` | Use canary build |
| `-w`, `--wait` | Wait until database is ready |
| `--from-csv <path>` | Import from CSV file |
| `--csv-table-name <name>` | Table name for CSV import |
| `--from-db <name>` | Copy from existing Turso database |
| `--from-dump <path>` | Import from SQL dump file |
| `--from-dump-url <url>` | Import from remote SQL dump URL |
| `--from-file <path>` | Import from SQLite `.db` file (max 2GB) |
| `--timestamp <RFC3339>` | Point-in-time from existing DB |

**Examples:**
```bash
# Basic create
turso db create my-app

# From existing SQLite file
turso db create my-app --from-file ./production.db

# From SQL dump
turso db create my-app --from-dump ./backup.sql

# From CSV
turso db create my-app --from-csv ./users.csv --csv-table-name users

# Copy from another database
turso db create staging-db --from-db prod-db

# Point-in-time copy
turso db create snapshot --from-db prod-db --timestamp "2024-03-15T10:00:00Z"
```

### `turso db list`
List all databases.

| Flag | Description |
|------|-------------|
| `--group <name>` | Filter by group |

### `turso db show <name>`
Show database details: name, ID, group, size, location, libSQL version.

| Flag | Description |
|------|-------------|
| `--url` | Show libSQL connection URL |
| `--http-url` | Show HTTP API URL |

### `turso db shell <name>`
Open an interactive SQLite shell for the database.

Supports all standard SQLite commands:
- `.tables` — List all tables
- `.schema` — Show table structures
- `.mode` — Switch output mode
- `.dump` — Export SQL
- `.help` — Show shell commands

### `turso db destroy <name>`
Permanently delete a database.

| Flag | Description |
|------|-------------|
| `-y` | Skip confirmation prompt |

### `turso db locations`
List all available deployment locations/regions.

---

## Group Commands (`turso group`)

### `turso group list`
List all groups and their locations.

### `turso group create <name> [location...]`
Create a new group with replicas in specified locations.

**Examples:**
```bash
turso group create eu-west lhr waw
turso group create us-multi pdx lhr
turso group create production lhr pdx hkg
```

### `turso group update <name>`
Update group locations.

| Flag | Description |
|------|-------------|
| `--add-location <loc>` | Add a replica location |
| `--remove-location <loc>` | Remove a replica location |
| `--wait` | Wait for operation to complete |

**Examples:**
```bash
turso group update production --add-location hkg
turso group update production --remove-location waw --add-location ams
```

### `turso group destroy <name>`
Permanently delete a group and all its replicas.

| Flag | Description |
|------|-------------|
| `-y` | Skip confirmation |

### `turso group transfer <name> <org-slug>`
Transfer a group to another organization.

### `turso group unarchive <name>`
Restore an archived group.

### `turso group tokens create <name>`
Create an authentication token scoped to the entire group (works for all databases in the group).

| Flag | Description |
|------|-------------|
| `--read-only` | Queries only |
| `-p <table>:<actions>` | Fine-grained table permissions |
| `--expiration <duration>` | Token TTL |

### `turso group tokens invalidate <name>`
Invalidate all tokens for a group.

---

### `turso group aws-migration info <name>`
Check AWS VPC migration status.

### `turso group aws-migration start <name>`
Start migrating a group to AWS infrastructure.

### `turso group aws-migration abort <name>`
Abort an in-progress AWS migration.

---

## Organization Commands (`turso org`)

### `turso org list`
List all organizations you're a member of.

### `turso org create <name>`
Create a new organization.

### `turso org destroy <name>`
Delete an organization.

| Flag | Description |
|------|-------------|
| `-y` | Skip confirmation |

### `turso org switch <name>`
Switch the active organization context.

### `turso org billing`
Show billing information and usage.

### `turso org members list`
List organization members and their roles.

### `turso org members add <email-or-username>`
Add a user to the organization.

### `turso org members invite <email>`
Send an invitation to join.

### `turso org members rm <email-or-username>`
Remove a user.

---

## Plan Commands (`turso plan`)

### `turso plan show`
Show current plan details, quotas, and usage.

### `turso plan select <plan-name>`
Switch between available plans.

### `turso plan upgrade`
Upgrade to a higher tier plan.

### `turso plan overages enable`
Enable overage charging (allows exceeding plan limits with extra charges).

### `turso plan overages disable`
Disable overages (database stops accepting writes when quota is reached).

---

## Config Commands

### `turso db config allow-rules set <db> <ip-or-cidr>`
Add an allowed IP address or CIDR range.

### `turso db config allow-rules show <db>`
Display current IP allow rules.

### `turso db config allow-rules clear <db>`
Remove all IP allow rules (allow any authenticated connection).

---

## Contact Commands

### `turso contact bookmeeting`
Schedule a meeting with the Turso team.

### `turso contact feedback <message>`
Submit feedback or feature requests.

---

## Help Commands

### `turso --help`
Show top-level command help.

### `turso <command> --help`
Show help for a specific command.

### `turso <command> <subcommand> --help`
Show help for a subcommand.

---

## Quick CLI Cheatsheet

```bash
# Essential workflow
turso auth login
turso db show <name> --url          # Get URL for .env
turso db tokens create <name>       # Get auth token for .env
turso db shell <name>               # Run SQL interactively
turso db list                       # See all databases
turso group list                    # See all groups
turso group create <name> <loc>     # New region group
turso group update <name> --add<loc>  # Add replica
turso org list                      # Switch context
turso plan show                     # Check limits
```
