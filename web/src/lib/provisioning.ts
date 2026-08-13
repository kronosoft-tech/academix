import { createClient, type Client } from '@libsql/client';
import { readdir, readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

/**
 * Per-user Turso database provisioning for web registration.
 *
 * TypeScript port of the desktop `TursoProvisioningService`
 * (`src-tauri/src/infrastructure/turso/provisioning.rs`) plus the standalone
 * `run_migrations_on_db` (`src-tauri/src/infrastructure/turso/connection_manager.rs`).
 *
 * Flow: env gate → createDatabase (409 retried with fresh suffix) →
 * createAuthToken (bare DB name, design D5) → dbUrl = libsql://{hostname} →
 * runMigrationsOnDb (web/migrations/per-user/001..020). On any failure after
 * database creation, a best-effort DELETE is attempted and registration fails
 * closed (no partial account state, no silent degradation).
 */

export interface ProvisionResult {
  dbName: string;
  dbUrl: string;
  dbToken: string;
  hostname: string;
}

export type ProvisioningErrorCode =
  | 'MISSING_ENV'
  | 'HTTP'
  | 'RATE_LIMIT'
  | 'CONFLICT'
  | 'AUTH'
  | 'MIGRATION';

export class ProvisioningError extends Error {
  readonly code: ProvisioningErrorCode;

  constructor(code: ProvisioningErrorCode, message: string) {
    super(message);
    this.name = 'ProvisioningError';
    this.code = code;
  }
}

export interface TursoEnv {
  apiToken: string;
  org: string;
  group: string;
}

const TURSO_API_BASE = 'https://api.turso.tech/v1';
const MAX_CREATE_ATTEMPTS = 4; // initial attempt + up to 3 retries on 409

/**
 * Generate a unique database slug from an academy name.
 *
 * Format: `academy-{normalized-name}-{4-char-hex}`
 *
 * Rules (mirrors `provisioning.rs::generate_db_slug`):
 * - Lowercase
 * - Replace every non `[a-z0-9-]` character with `-`
 * - Collapse consecutive hyphens
 * - Trim leading/trailing hyphens, cap at 30 chars
 * - Append a random 4-char hex suffix for uniqueness
 */
export function generateDbSlug(academyName: string): string {
  const normalized = academyName
    .toLowerCase()
    .split('')
    .map((c) => (/[a-z0-9-]/.test(c) ? c : '-'))
    .join('');

  // Remove consecutive hyphens
  let clean = '';
  for (const c of normalized) {
    if (c === '-' && clean.endsWith('-')) {
      continue;
    }
    clean += c;
  }

  // Trim leading/trailing hyphens and limit to 30 chars
  const trimmed = clean.replace(/^-+|-+$/g, '').slice(0, 30);

  return `academy-${trimmed}-${randomHex4()}`;
}

/**
 * 4-char hex suffix. Mirrors the desktop behavior of taking the first 4 chars
 * of a v4 UUID (always hex digits).
 */
function randomHex4(): string {
  return globalThis.crypto.randomUUID().slice(0, 4);
}

/**
 * Read the Turso provisioning env vars. Fails closed (design D6): if any of
 * TURSO_API_TOKEN / TURSO_ORG / TURSO_GROUP is missing, provisioning must NOT
 * silently degrade — registration is rejected with an actionable error.
 */
export function getTursoEnv(): TursoEnv {
  const apiToken = import.meta.env.TURSO_API_TOKEN;
  const org = import.meta.env.TURSO_ORG;
  const group = import.meta.env.TURSO_GROUP;

  const missing: string[] = [];
  if (!apiToken) missing.push('TURSO_API_TOKEN');
  if (!org) missing.push('TURSO_ORG');
  if (!group) missing.push('TURSO_GROUP');

  if (missing.length > 0) {
    throw new ProvisioningError(
      'MISSING_ENV',
      `Turso provisioning requires ${missing.join(', ')}; registration fails closed`
    );
  }

  return { apiToken: String(apiToken), org: String(org), group: String(group) };
}

/**
 * Create a database in the Turso organization.
 *
 * `POST /v1/organizations/{org}/databases`
 *
 * On 409 (name conflict) retries up to 3 times, generating a fresh 4-char hex
 * suffix each retry (mirrors `provisioning.rs::create_database`: base name is
 * capped at 25 chars before the new suffix). If all attempts conflict, throws
 * CONFLICT.
 */
export async function createDatabase(
  org: string,
  name: string,
  group: string
): Promise<{ name: string; hostname: string }> {
  const { apiToken } = getTursoEnv();
  const url = `${TURSO_API_BASE}/organizations/${encodeURIComponent(org)}/databases`;

  let attemptName = name;
  for (let attempt = 0; attempt < MAX_CREATE_ATTEMPTS; attempt++) {
    const response = await apiRequest(url, {
      method: 'POST',
      apiToken,
      body: JSON.stringify({ name: attemptName, group }),
    });

    if (response.ok) {
      return parseCreateDatabaseResponse(response);
    }

    if (response.status === 409) {
      if (attempt < MAX_CREATE_ATTEMPTS - 1) {
        attemptName = `${name.slice(0, 25)}-${randomHex4()}`;
        continue;
      }
      throw new ProvisioningError(
        'CONFLICT',
        'Could not create database: all name attempts conflicted'
      );
    }

    throw mapStatusError(response.status, await errorBody(response));
  }

  throw new ProvisioningError(
    'CONFLICT',
    'Could not create database: all name attempts conflicted'
  );
}

/**
 * Create a full-access auth token for a database.
 *
 * `POST /v1/organizations/{org}/databases/{dbName}/auth/tokens`
 *
 * `dbName` MUST be the bare database name, not the `libsql://` URL (design D5 —
 * the desktop `lib.rs` passes the URL, which is a known bug; do not replicate).
 */
export async function createAuthToken(org: string, dbName: string): Promise<string> {
  const { apiToken } = getTursoEnv();
  const url =
    `${TURSO_API_BASE}/organizations/${encodeURIComponent(org)}/databases/` +
    `${encodeURIComponent(dbName)}/auth/tokens`;

  const response = await apiRequest(url, {
    method: 'POST',
    apiToken,
    body: JSON.stringify({ permission: 'full-access' }),
  });

  if (response.ok) {
    try {
      const data = (await response.json()) as { jwt?: string };
      if (typeof data?.jwt === 'string' && data.jwt.length > 0) {
        return data.jwt;
      }
    } catch {
      // fall through to the malformed-response error below
    }
    throw new ProvisioningError('HTTP', 'Malformed createAuthToken response: missing jwt');
  }

  throw mapStatusError(response.status, await errorBody(response));
}

/**
 * Delete a database. A 404 is treated as success (nothing to delete).
 *
 * `DELETE /v1/organizations/{org}/databases/{dbName}`
 */
export async function deleteDatabase(org: string, dbName: string): Promise<void> {
  const { apiToken } = getTursoEnv();
  const url =
    `${TURSO_API_BASE}/organizations/${encodeURIComponent(org)}/databases/` +
    `${encodeURIComponent(dbName)}`;

  const response = await apiRequest(url, { method: 'DELETE', apiToken });

  if (response.ok || response.status === 404) {
    return;
  }

  throw mapStatusError(response.status, await errorBody(response));
}

/**
 * Run the per-user migrations against a database.
 *
 * Mirrors the desktop `run_migrations_on_db`: creates a `_schema_migrations`
 * tracking table, reads `web/migrations/per-user/*.sql` sorted by filename,
 * and per file — skips it if already recorded, otherwise executes the whole
 * file (batched) and records the version. Re-running is a no-op. A failing
 * file aborts with a MIGRATION error.
 */
export async function runMigrationsOnDb(client: Client): Promise<void> {
  await client.execute(`CREATE TABLE IF NOT EXISTS _schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
  )`);

  const dir = migrationsDir();
  let entries: string[];
  try {
    entries = (await readdir(dir)).filter((f) => f.endsWith('.sql')).sort();
  } catch (err) {
    throw new ProvisioningError(
      'MIGRATION',
      `Failed to read migrations dir: ${errorMessage(err)}`
    );
  }

  for (const version of entries) {
    const applied = await client.execute({
      sql: 'SELECT version FROM _schema_migrations WHERE version = ?',
      args: [version],
    });

    if (applied.rows.length > 0) {
      continue;
    }

    let sql: string;
    try {
      sql = await readFile(path.join(dir, version), 'utf8');
    } catch (err) {
      throw new ProvisioningError('MIGRATION', `Failed to read ${version}: ${errorMessage(err)}`);
    }

    try {
      await client.executeMultiple(sql);
    } catch (err) {
      throw new ProvisioningError('MIGRATION', `Migration ${version} failed: ${errorMessage(err)}`);
    }

    try {
      await client.execute({
        sql: 'INSERT INTO _schema_migrations (version) VALUES (?)',
        args: [version],
      });
    } catch (err) {
      throw new ProvisioningError(
        'MIGRATION',
        `Failed to record migration ${version}: ${errorMessage(err)}`
      );
    }
  }
}

/**
 * Provision a per-user database for a newly registered academy.
 *
 * Orchestrates: env gate → createDatabase → createAuthToken (bare name) →
 * dbUrl = `libsql://{hostname}` → runMigrationsOnDb. On any failure after the
 * database was created, a best-effort DELETE is attempted and the error is
 * rethrown as a ProvisioningError (fail closed — no partial state).
 */
export async function provisionUser(academyName: string): Promise<ProvisionResult> {
  const env = getTursoEnv();
  const slug = generateDbSlug(academyName);

  // createDatabase throws on failure without creating a DB, so no cleanup is
  // needed before this point.
  const { name: dbName, hostname } = await createDatabase(env.org, slug, env.group);

  try {
    const dbToken = await createAuthToken(env.org, dbName);
    const dbUrl = `libsql://${hostname}`;
    const client = createClient({ url: dbUrl, authToken: dbToken });
    try {
      await runMigrationsOnDb(client);
    } finally {
      try {
        client.close();
      } catch {
        // Closing the migration connection is best-effort bookkeeping only.
      }
    }
    return { dbName, dbUrl, dbToken, hostname };
  } catch (err) {
    // The DB exists — try to clean it up, but never let cleanup mask the
    // original error (design R5: best-effort DELETE, fail closed).
    try {
      await deleteDatabase(env.org, dbName);
    } catch {
      // best-effort only
    }
    if (err instanceof ProvisioningError) {
      throw err;
    }
    throw new ProvisioningError(
      'HTTP',
      `Provisioning failed after database creation: ${errorMessage(err)}`
    );
  }
}

/** Directory holding the per-user migration copies. */
function migrationsDir(): string {
  const override = process.env.TURSO_MIGRATIONS_DIR;
  if (override) {
    return override;
  }
  // Dev/tests/typecheck: web/src/lib/provisioning.ts → web/migrations/per-user.
  return fileURLToPath(new URL('../../migrations/per-user/', import.meta.url));
}

async function apiRequest(
  url: string,
  init: { method: 'GET' | 'POST' | 'DELETE'; apiToken: string; body?: string }
): Promise<Response> {
  try {
    return await fetch(url, {
      method: init.method,
      headers: {
        Authorization: `Bearer ${init.apiToken}`,
        'Content-Type': 'application/json',
      },
      body: init.body,
    });
  } catch (err) {
    throw new ProvisioningError('HTTP', `Turso API request failed: ${errorMessage(err)}`);
  }
}

interface CreateDatabaseResponse {
  database?: {
    name?: string;
    hostname?: string;
    // Desktop serde expects capitalized keys; accept both shapes.
    Name?: string;
    Hostname?: string;
  };
}

async function parseCreateDatabaseResponse(
  response: Response
): Promise<{ name: string; hostname: string }> {
  let data: CreateDatabaseResponse;
  try {
    data = (await response.json()) as CreateDatabaseResponse;
  } catch (err) {
    throw new ProvisioningError('HTTP', `Malformed createDatabase response: ${errorMessage(err)}`);
  }
  const db = data?.database;
  const name = db?.name ?? db?.Name;
  const hostname = db?.hostname ?? db?.Hostname;
  if (!name || !hostname) {
    throw new ProvisioningError(
      'HTTP',
      'Malformed createDatabase response: missing name/hostname'
    );
  }
  return { name, hostname };
}

function mapStatusError(status: number, body: string): ProvisioningError {
  if (status === 401 || status === 403) {
    return new ProvisioningError('AUTH', `Turso API auth failed (${status}): ${body}`);
  }
  if (status === 429) {
    return new ProvisioningError('RATE_LIMIT', 'Turso API rate limit exceeded (429)');
  }
  return new ProvisioningError('HTTP', `Turso API HTTP ${status}: ${body}`);
}

async function errorBody(response: Response): Promise<string> {
  try {
    return await response.text();
  } catch {
    return '';
  }
}

function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}
