# Turso Platform API Reference

Programmatic management of databases, groups, organizations, locations, and tokens via REST API.

**Trigger:** Turso API, programmatic database management, Platform API endpoints, automation, CI/CD database provisioning.

---

## Base URL & Authentication

```
Base URL: https://api.turso.tech/v1
```

All requests require a Bearer token:
```
Authorization: Bearer <API_TOKEN>
```

Create API tokens:
- **CLI**: `turso auth api-tokens mint <name>`
- **API**: `POST /v1/auth/api-tokens`

---

## Authentication Endpoints

### Create API Token
```
POST /v1/auth/api-tokens
```

Returns a new API token. The token can be minted at three levels:
1. Unrestricted — Full platform access
2. Organization-scoped — Limited to specific org
3. Group-scoped — Limited to a group

### List API Tokens
```
GET /v1/auth/api-tokens
```

Returns all API tokens belonging to the user.

### List Organization Tokens
```
GET /v1/organizations/:orgSlug/tokens
```

Returns tokens scoped to this organization (org-scoped + group-scoped).

### Revoke API Token
```
DELETE /v1/auth/api-tokens/:token_id
```

### Validate API Token
```
POST /v1/auth/validate
```

---

## User Endpoints

### Get Current User
```
GET /v1/users/me
```

Returns information about the currently authenticated user.

---

## Organization Endpoints

### List Organizations
```
GET /v1/organizations
```

Returns all orgs the user owns or is a member of. Includes:
- `slug` — Used in URL paths
- `type` — Personal vs team
- `plan` — Current subscription plan

### Retrieve Organization
```
GET /v1/organizations/:orgSlug
```

### Update Organization
```
PATCH /v1/organizations/:orgSlug
```

### List Invoices
```
GET /v1/organizations/:orgSlug/invoices
```

### Current Subscription
```
GET /v1/organizations/:orgSlug/subscription
```

### Organization Usage
```
GET /v1/organizations/:orgSlug/usage
```

Fetch current billing cycle usage (rows read, storage, etc.).

### List Plans
```
GET /v1/organizations/plans
```

---

## Organization Members

### List Members
```
GET /v1/organizations/:orgSlug/members
```

### Retrieve Member
```
GET /v1/organizations/:orgSlug/members/:username
```

### Add Member
```
PUT /v1/organizations/:orgSlug/members/:username
```

Body:
```json
{ "role": "admin" }
```

### Update Member Role
```
PATCH /v1/organizations/:orgSlug/members/:username
```

Body:
```json
{ "role": "admin" }
```

### Remove Member
```
DELETE /v1/organizations/:orgSlug/members/:username
```

---

## Organization Invites

### List Invites
```
GET /v1/organizations/:orgSlug/invites
```

### Create Invite (v2)
```
POST /v1/organizations/:orgSlug/invites
```

Body:
```json
{ "email": "new-member@example.com" }
```

### Delete Invite (v2)
```
DELETE /v1/organizations/:orgSlug/invites/:email
```

---

## Database Endpoints

### List Databases
```
GET /v1/organizations/:orgSlug/databases
```

Returns:
- `name` — Database name
- `ID` — UUID
- `group` — Associated group
- `locations` — Deployed locations
- `primaryRegion` — Primary write region

### Retrieve Database
```
GET /v1/organizations/:orgSlug/databases/:name
```

### Create Database
```
POST /v1/organizations/:orgSlug/databases
```

Body:
```json
{
  "name": "my-app",
  "group": "default",
  "seed": {
    "type": "database",
    "name": "source-db"
  }
}
```

For point-in-time seeding:
```json
{
  "name": "snapshot",
  "group": "default",
  "seed": {
    "type": "database",
    "name": "production",
    "timestamp": "2024-01-15T00:00:00Z"
  }
}
```

### Delete Database
```
DELETE /v1/organizations/:orgSlug/databases/:name
```

### Retrieve Database Usage
```
GET /v1/organizations/:orgSlug/databases/:name/usage
```

### Retrieve Database Stats
```
GET /v1/organizations/:orgSlug/databases/:name/stats
```

Top queries by rows read/written.

### Update Database Configuration
```
PATCH /v1/organizations/:orgSlug/databases/:name/config
```

### Retrieve Database Configuration
```
GET /v1/organizations/:orgSlug/databases/:name/config
```

Returns database config including `allowedVpcEndpointIds`.

### Upload Database
```
POST /v1/organizations/:orgSlug/databases/:name/upload
```

Upload a database file to Turso.

### List Database Instances
```
GET /v1/organizations/:orgSlug/databases/:name/instances
```

Returns individual instances (primary + replicas) in each region.

### Retrieve Database Instance
```
GET /v1/organizations/:orgSlug/databases/:name/instances/:location
```

### Generate Database Token
```
POST /v1/organizations/:orgSlug/databases/:name/auth/tokens
```

### Invalidate All Database Tokens
```
DELETE /v1/organizations/:orgSlug/databases/:name/auth/tokens
```

---

## Group Endpoints

### List Groups
```
GET /v1/organizations/:orgSlug/groups
```

### Retrieve Group
```
GET /v1/organizations/:orgSlug/groups/:name
```

### Create Group
```
POST /v1/organizations/:orgSlug/groups
```

Body:
```json
{
  "name": "production",
  "locations": ["lhr", "pdx"]
}
```

### Delete Group
```
DELETE /v1/organizations/:orgSlug/groups/:name
```

### Retrieve Group Configuration
```
GET /v1/organizations/:orgSlug/groups/:name/config
```

### Update Group Configuration
```
PATCH /v1/organizations/:orgSlug/groups/:name/config
```

### Transfer Group
```
POST /v1/organizations/:orgSlug/groups/:name/transfer
```

Body:
```json
{ "organization": "target-org-slug" }
```

### Unarchive Group
```
POST /v1/organizations/:orgSlug/groups/:name/unarchive
```

### Create Group Token
```
POST /v1/organizations/:orgSlug/groups/:name/auth/tokens
```

### Invalidate All Group Tokens
```
DELETE /v1/organizations/:orgSlug/groups/:name/auth/tokens
```

---

## Location Endpoints

### List Locations
```
GET /v1/locations
```

Returns available deployment regions with codes and friendly names.

### Closest Region
```
GET /v1/locations/closest-region
```

Returns the region closest to the requesting client.

---

## Audit Logs

### List Audit Logs
```
GET /v1/organizations/:orgSlug/audit-logs
```

Returns audit logs for the organization, ordered by `created_at` descending.

---

## cURL Examples

### Create Database

```bash
curl -X POST https://api.turso.tech/v1/organizations/my-org/databases \
  -H "Authorization: Bearer $TURSO_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "production-db",
    "group": "default"
  }'
```

### Get Database URL and Token

```bash
# Get database details
curl https://api.turso.tech/v1/organizations/my-org/databases/production-db \
  -H "Authorization: Bearer $TURSO_API_TOKEN"

# Generate token
curl -X POST https://api.turso.tech/v1/organizations/my-org/databases/production-db/auth/tokens \
  -H "Authorization: Bearer $TURSO_API_TOKEN"
```

### List All Databases

```bash
curl https://api.turso.tech/v1/organizations/my-org/databases \
  -H "Authorization: Bearer $TURSO_API_TOKEN"
```

### List All Groups

```bash
curl https://api.turso.tech/v1/organizations/my-org/groups \
  -H "Authorization: Bearer $TURSO_API_TOKEN"
```

### Create Group with Multi-Region

```bash
curl -X POST https://api.turso.tech/v1/organizations/my-org/groups \
  -H "Authorization: Bearer $TURSO_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "eu-group",
    "locations": ["lhr", "ams", "fra"]
  }'
```

---

## Platform API Quickstart

1. **Create a Platform API token**:
   ```bash
   turso auth api-tokens mint ci-cd-token
   ```

2. **Set it as an environment variable**:
   ```bash
   export TURSO_API_TOKEN=your-token-here
   ```

3. **Make API calls**:
   ```bash
   curl https://api.turso.tech/v1/organizations/my-org/databases \
     -H "Authorization: Bearer $TURSO_API_TOKEN"
   ```

---

## Error Codes

| Status | Meaning |
|--------|---------|
| `400` | Bad Request — Invalid input |
| `401` | Unauthorized — Invalid or missing token |
| `403` | Forbidden — Insufficient permissions |
| `404` | Not Found — Resource doesn't exist |
| `409` | Conflict — Database/group already exists |
| `422` | Unprocessable Entity — Validation errors |
| `429` | Rate Limitted — Too many requests |
| `500` | Internal Server Error |

---

## Rate Limits

The Turso Platform API has rate limits enforced per API token. Use `Retry-After` header when hitting 429 responses.

---

## Key URLs

- **API Reference**: https://docs.turso.tech/api-reference
- **API Quickstart**: https://docs.turso.tech/api-reference/quickstart
- **OpenAPI Spec**: https://docs.turso.tech/api-reference/openapi.json
