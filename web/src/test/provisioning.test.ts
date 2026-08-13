import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {
  generateDbSlug,
  createDatabase,
  createAuthToken,
  deleteDatabase,
  getTursoEnv,
  runMigrationsOnDb,
  provisionUser,
  ProvisioningError,
} from '../lib/provisioning';

// provisionUser creates a libsql client for the (mocked) remote DB URL. In
// tests we redirect that client to a real `file:` database so the migration
// runner executes against a real libsql client (integration-style), while the
// Turso Platform API itself stays mocked.
const mocks = vi.hoisted(() => ({
  createClientMock: vi.fn(),
  realCreateClient: null as unknown as typeof import('@libsql/client').createClient,
}));

vi.mock('@libsql/client', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@libsql/client')>();
  mocks.realCreateClient = actual.createClient;
  return { ...actual, createClient: mocks.createClientMock };
});

const tmpFiles: string[] = [];
const fixtureDirs: string[] = [];

function tempDbPath(): string {
  const p = path.join(
    os.tmpdir(),
    `provisioning-test-${Date.now()}-${Math.random().toString(36).slice(2)}.db`
  );
  tmpFiles.push(p);
  return p;
}

function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

function stubTursoEnv(): void {
  vi.stubEnv('TURSO_API_TOKEN', 'test-token');
  vi.stubEnv('TURSO_ORG', 'academix');
  vi.stubEnv('TURSO_GROUP', 'default');
}

async function createFixtureDir(files: Record<string, string>): Promise<string> {
  const dir = await mkdtemp(path.join(os.tmpdir(), 'provisioning-fixture-'));
  fixtureDirs.push(dir);
  for (const [name, content] of Object.entries(files)) {
    await writeFile(path.join(dir, name), content);
  }
  return dir;
}

async function migrationCount(
  client: Awaited<ReturnType<typeof import('@libsql/client').createClient>>
): Promise<number> {
  const result = await client.execute('SELECT COUNT(*) FROM _schema_migrations');
  return Number((result.rows[0] as Record<string, unknown>)['COUNT(*)']);
}

beforeEach(() => {
  mocks.createClientMock.mockReset();
});

afterEach(async () => {
  vi.unstubAllGlobals();
  vi.unstubAllEnvs();
  for (const p of tmpFiles) {
    await rm(p, { force: true });
    await rm(`${p}-wal`, { force: true });
    await rm(`${p}-shm`, { force: true });
  }
  tmpFiles.length = 0;
  for (const d of fixtureDirs) {
    await rm(d, { recursive: true, force: true });
  }
  fixtureDirs.length = 0;
});

describe('generateDbSlug', () => {
  it('lowercases the academy name', () => {
    const slug = generateDbSlug('Music School');
    expect(slug).toMatch(/^academy-music-school-[0-9a-f]{4}$/);
  });

  it('replaces spaces with hyphens', () => {
    const slug = generateDbSlug('My Academy Name');
    expect(slug).toMatch(/^academy-my-academy-name-[0-9a-f]{4}$/);
  });

  it('replaces special characters with hyphens', () => {
    const slug = generateDbSlug('Hello@World!#2024');
    expect(slug).toMatch(/^academy-hello-world-2024-[0-9a-f]{4}$/);
  });

  it('trims long names to 30 chars plus suffix', () => {
    const slug = generateDbSlug('A very long academy name that should be truncated significantly');
    expect(slug.length).toBeLessThanOrEqual('academy-'.length + 30 + 1 + 4);
    expect(slug.startsWith('academy-')).toBe(true);
  });

  it('handles underscores as hyphens', () => {
    const slug = generateDbSlug('music_school');
    expect(slug).toMatch(/^academy-music-school-[0-9a-f]{4}$/);
  });

  it('collapses consecutive hyphens', () => {
    const slug = generateDbSlug('My   Academy---Name');
    expect(slug).toMatch(/^academy-my-academy-name-[0-9a-f]{4}$/);
  });

  it('trims leading and trailing hyphens', () => {
    const slug = generateDbSlug('-Academy Name-');
    expect(slug).toMatch(/^academy-academy-name-[0-9a-f]{4}$/);
  });

  it('generates a unique 4-char hex suffix', () => {
    const slug1 = generateDbSlug('Test');
    const slug2 = generateDbSlug('Test');
    expect(slug1).not.toBe(slug2);
    expect(slug1.slice(-4)).toMatch(/^[0-9a-f]{4}$/);
  });
});

describe('getTursoEnv', () => {
  it('returns the env values when all vars are present', () => {
    stubTursoEnv();
    expect(getTursoEnv()).toEqual({ apiToken: 'test-token', org: 'academix', group: 'default' });
  });

  it('fails closed when TURSO_API_TOKEN is missing', () => {
    vi.stubEnv('TURSO_ORG', 'academix');
    vi.stubEnv('TURSO_GROUP', 'default');
    try {
      getTursoEnv();
      expect.unreachable('should have thrown');
    } catch (err) {
      expect(err).toBeInstanceOf(ProvisioningError);
      expect((err as ProvisioningError).code).toBe('MISSING_ENV');
      expect((err as ProvisioningError).message).toContain('TURSO_API_TOKEN');
    }
  });

  it('fails closed when TURSO_ORG is missing', () => {
    vi.stubEnv('TURSO_API_TOKEN', 'test-token');
    vi.stubEnv('TURSO_GROUP', 'default');
    try {
      getTursoEnv();
      expect.unreachable('should have thrown');
    } catch (err) {
      expect((err as ProvisioningError).code).toBe('MISSING_ENV');
      expect((err as ProvisioningError).message).toContain('TURSO_ORG');
    }
  });

  it('fails closed when TURSO_GROUP is missing', () => {
    vi.stubEnv('TURSO_API_TOKEN', 'test-token');
    vi.stubEnv('TURSO_ORG', 'academix');
    try {
      getTursoEnv();
      expect.unreachable('should have thrown');
    } catch (err) {
      expect((err as ProvisioningError).code).toBe('MISSING_ENV');
      expect((err as ProvisioningError).message).toContain('TURSO_GROUP');
    }
  });

  it('fails closed when all vars are missing', () => {
    try {
      getTursoEnv();
      expect.unreachable('should have thrown');
    } catch (err) {
      expect((err as ProvisioningError).code).toBe('MISSING_ENV');
    }
  });
});

describe('createDatabase', () => {
  it('creates a database on first attempt', async () => {
    stubTursoEnv();
    const fetchMock = vi.fn().mockResolvedValue(
      jsonResponse(200, {
        database: { name: 'academy-test-1a2b', hostname: 'academy-test-1a2b.x.turso.io' },
      })
    );
    vi.stubGlobal('fetch', fetchMock);

    const result = await createDatabase('academix', 'academy-test-1a2b', 'default');

    expect(result).toEqual({
      name: 'academy-test-1a2b',
      hostname: 'academy-test-1a2b.x.turso.io',
    });
    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(url).toBe('https://api.turso.tech/v1/organizations/academix/databases');
    expect(init.method).toBe('POST');
    expect((init.headers as Record<string, string>).Authorization).toBe('Bearer test-token');
    expect(JSON.parse(String(init.body))).toEqual({ name: 'academy-test-1a2b', group: 'default' });
  });

  it('accepts capitalized name/hostname keys (desktop parity)', async () => {
    stubTursoEnv();
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        jsonResponse(200, {
          database: { Name: 'academy-x-9f01', Hostname: 'academy-x-9f01.y.turso.io' },
        })
      )
    );

    const result = await createDatabase('academix', 'academy-x-9f01', 'default');
    expect(result).toEqual({
      name: 'academy-x-9f01',
      hostname: 'academy-x-9f01.y.turso.io',
    });
  });

  it('retries on 409 with a fresh suffix, up to 3 retries', async () => {
    stubTursoEnv();
    const fetchMock = vi.fn();
    fetchMock
      .mockResolvedValueOnce(jsonResponse(409, {}))
      .mockImplementation(async (input: string, init?: RequestInit) => {
        const body = JSON.parse(String(init?.body)) as { name: string };
        return jsonResponse(200, {
          database: { name: body.name, hostname: `${body.name}.x.turso.io` },
        });
      });
    vi.stubGlobal('fetch', fetchMock);

    const result = await createDatabase('academix', 'academy-music-school-ab12', 'default');

    expect(fetchMock).toHaveBeenCalledTimes(2);
    const firstBody = JSON.parse(String(fetchMock.mock.calls[0][1]?.body)) as { name: string };
    const secondBody = JSON.parse(String(fetchMock.mock.calls[1][1]?.body)) as { name: string };
    // Fresh 4-char hex suffix on the retried name, base capped at 25 chars.
    expect(secondBody.name).toMatch(/^academy-music-school-ab12-[0-9a-f]{4}$/);
    expect(secondBody.name).not.toBe(firstBody.name);
    expect(result.name).toBe(secondBody.name);
    expect(result.hostname).toBe(`${secondBody.name}.x.turso.io`);
  });

  it('throws CONFLICT when all 4 attempts conflict', async () => {
    stubTursoEnv();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse(409, {})));

    await expect(createDatabase('academix', 'academy-x-1234', 'default')).rejects.toMatchObject({
      name: 'ProvisioningError',
      code: 'CONFLICT',
    });
  });

  it('maps 429 to RATE_LIMIT', async () => {
    stubTursoEnv();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse(429, {})));

    await expect(createDatabase('academix', 'academy-x', 'default')).rejects.toMatchObject({
      code: 'RATE_LIMIT',
    });
  });

  it('maps 401 to AUTH', async () => {
    stubTursoEnv();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse(401, { error: 'unauthorized' })));

    await expect(createDatabase('academix', 'academy-x', 'default')).rejects.toMatchObject({
      code: 'AUTH',
    });
  });

  it('maps other non-2xx to HTTP', async () => {
    stubTursoEnv();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse(500, { error: 'boom' })));

    await expect(createDatabase('academix', 'academy-x', 'default')).rejects.toMatchObject({
      code: 'HTTP',
    });
  });

  it('maps network failures to HTTP', async () => {
    stubTursoEnv();
    vi.stubGlobal(
      'fetch',
      vi.fn().mockRejectedValue(new TypeError('Failed to fetch'))
    );

    await expect(createDatabase('academix', 'academy-x', 'default')).rejects.toMatchObject({
      code: 'HTTP',
    });
  });
});

describe('createAuthToken', () => {
  it('creates a full-access token using the bare database name (design D5)', async () => {
    stubTursoEnv();
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse(200, { jwt: 'jwt-abc-123' }));
    vi.stubGlobal('fetch', fetchMock);

    const token = await createAuthToken('academix', 'academy-music-school-1a2b');

    expect(token).toBe('jwt-abc-123');
    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(url).toBe(
      'https://api.turso.tech/v1/organizations/academix/databases/academy-music-school-1a2b/auth/tokens'
    );
    expect(url).not.toContain('libsql://');
    expect(JSON.parse(String(init.body))).toEqual({ permission: 'full-access' });
  });

  it('throws AUTH on 401', async () => {
    stubTursoEnv();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse(401, {})));

    await expect(createAuthToken('academix', 'academy-x-1234')).rejects.toMatchObject({
      code: 'AUTH',
    });
  });

  it('throws HTTP on malformed response (missing jwt)', async () => {
    stubTursoEnv();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse(200, { token: 'nope' })));

    await expect(createAuthToken('academix', 'academy-x-1234')).rejects.toMatchObject({
      code: 'HTTP',
    });
  });
});

describe('deleteDatabase', () => {
  it('resolves on success', async () => {
    stubTursoEnv();
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse(200, {}));
    vi.stubGlobal('fetch', fetchMock);

    await expect(deleteDatabase('academix', 'academy-x-1234')).resolves.toBeUndefined();
    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(url).toBe('https://api.turso.tech/v1/organizations/academix/databases/academy-x-1234');
    expect(init.method).toBe('DELETE');
  });

  it('treats 404 as success', async () => {
    stubTursoEnv();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse(404, {})));

    await expect(deleteDatabase('academix', 'academy-x-1234')).resolves.toBeUndefined();
  });

  it('throws AUTH on 403', async () => {
    stubTursoEnv();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse(403, {})));

    await expect(deleteDatabase('academix', 'academy-x-1234')).rejects.toMatchObject({
      code: 'AUTH',
    });
  });

  it('throws HTTP on other errors', async () => {
    stubTursoEnv();
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse(500, {})));

    await expect(deleteDatabase('academix', 'academy-x-1234')).rejects.toMatchObject({
      code: 'HTTP',
    });
  });
});

describe('runMigrationsOnDb', () => {
  it('applies all 20 real migrations to a fresh database', async () => {
    const dbPath = tempDbPath();
    const client = mocks.realCreateClient({ url: `file:${dbPath}` });

    await runMigrationsOnDb(client);

    expect(await migrationCount(client)).toBe(20);

    // Spot-check tables from 001 and 020 actually exist.
    const tables = await client.execute(
      "SELECT name FROM sqlite_master WHERE type = 'table' AND name IN ('users', 'web_admins', 'subscriptions') ORDER BY name"
    );
    expect(tables.rows.map((r) => (r as Record<string, unknown>).name)).toEqual([
      'subscriptions',
      'users',
      'web_admins',
    ]);

    // Versions are recorded by filename.
    const versions = await client.execute(
      "SELECT version FROM _schema_migrations WHERE version = '001_initial_schema.sql' OR version = '020_web_schema.sql' ORDER BY version"
    );
    expect(versions.rows.map((r) => (r as Record<string, unknown>).version)).toEqual([
      '001_initial_schema.sql',
      '020_web_schema.sql',
    ]);
    client.close();
  });

  it('is a no-op on rerun (idempotent)', async () => {
    const dbPath = tempDbPath();
    const client = mocks.realCreateClient({ url: `file:${dbPath}` });

    await runMigrationsOnDb(client);
    await runMigrationsOnDb(client);

    expect(await migrationCount(client)).toBe(20);
    client.close();
  });

  it('aborts with MIGRATION when a file fails, keeping prior versions recorded', async () => {
    const fixtureDir = await createFixtureDir({
      '001_ok.sql': 'CREATE TABLE IF NOT EXISTS fixture_ok (id TEXT PRIMARY KEY);',
      '002_bad.sql': 'THIS IS NOT VALID SQL;',
    });
    vi.stubEnv('TURSO_MIGRATIONS_DIR', fixtureDir);

    const dbPath = tempDbPath();
    const client = mocks.realCreateClient({ url: `file:${dbPath}` });

    await expect(runMigrationsOnDb(client)).rejects.toMatchObject({
      name: 'ProvisioningError',
      code: 'MIGRATION',
      message: expect.stringContaining('002_bad.sql'),
    });

    // 001 was applied and recorded before the failure; 002 was not.
    expect(await migrationCount(client)).toBe(1);
    client.close();
  });
});

describe('provisionUser', () => {
  it('provisions a database end-to-end with a real libsql migration run', async () => {
    stubTursoEnv();
    const dbPath = tempDbPath();
    const realFileClient = mocks.realCreateClient({ url: `file:${dbPath}` });
    mocks.createClientMock.mockReturnValue(realFileClient);

    const fetchMock = vi.fn();
    fetchMock
      // createDatabase → success
      .mockResolvedValueOnce(
        jsonResponse(200, {
          database: { name: 'academy-test-1a2b', hostname: 'academy-test-1a2b.x.turso.io' },
        })
      )
      // createAuthToken → success
      .mockResolvedValueOnce(jsonResponse(200, { jwt: 'jwt-full-access-xyz' }));
    vi.stubGlobal('fetch', fetchMock);

    const result = await provisionUser('Test');

    // dbUrl is built from the hostname (libsql://{hostname}).
    expect(result).toEqual({
      dbName: 'academy-test-1a2b',
      dbUrl: 'libsql://academy-test-1a2b.x.turso.io',
      dbToken: 'jwt-full-access-xyz',
      hostname: 'academy-test-1a2b.x.turso.io',
    });

    // createAuthToken received the bare name, not the libsql:// URL (D5).
    const tokenUrl = String(fetchMock.mock.calls[1][0]);
    expect(tokenUrl).toContain('/databases/academy-test-1a2b/auth/tokens');
    expect(tokenUrl).not.toContain('libsql://');

    // The real migration runner executed the 20 real files on a real client.
    const verifyClient = mocks.realCreateClient({ url: `file:${dbPath}` });
    expect(await migrationCount(verifyClient)).toBe(20);
    verifyClient.close();
  });

  it('retries createDatabase on 409 with a fresh suffix', async () => {
    stubTursoEnv();
    const dbPath = tempDbPath();
    mocks.createClientMock.mockReturnValue(mocks.realCreateClient({ url: `file:${dbPath}` }));

    let createCalls = 0;
    const fetchMock = vi.fn(async (input: string, init?: RequestInit) => {
      const url = String(input);
      if (url.includes('/auth/tokens')) {
        return jsonResponse(200, { jwt: 'jwt-abc' });
      }
      createCalls += 1;
      if (createCalls === 1) {
        return jsonResponse(409, {});
      }
      const body = JSON.parse(String(init?.body)) as { name: string };
      return jsonResponse(200, {
        database: { name: body.name, hostname: `${body.name}.x.turso.io` },
      });
    });
    vi.stubGlobal('fetch', fetchMock);

    const result = await provisionUser('Test');

    expect(createCalls).toBe(2);
    expect(result.dbName).toMatch(/^academy-test-[0-9a-f]{4}-[0-9a-f]{4}$/);
    expect(result.dbUrl).toBe(`libsql://${result.dbName}.x.turso.io`);
  });

  it('throws CONFLICT when all 4 create attempts conflict', async () => {
    stubTursoEnv();
    mocks.createClientMock.mockReturnValue(mocks.realCreateClient({ url: `file:${tempDbPath()}` }));
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse(409, {})));

    await expect(provisionUser('Test')).rejects.toMatchObject({ code: 'CONFLICT' });
    // No token request was ever made.
    const calls = (vi.mocked(fetch).mock.calls as [string][]) ?? [];
    expect(calls.some(([url]) => String(url).includes('/auth/tokens'))).toBe(false);
  });

  it('fails closed with MISSING_ENV before any API call', async () => {
    // No env stubs — all three vars absent.
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await expect(provisionUser('Test')).rejects.toMatchObject({ code: 'MISSING_ENV' });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('deletes the created database best-effort when createAuthToken fails', async () => {
    stubTursoEnv();
    mocks.createClientMock.mockReturnValue(mocks.realCreateClient({ url: `file:${tempDbPath()}` }));

    const fetchMock = vi.fn();
    fetchMock
      // createDatabase → success
      .mockResolvedValueOnce(
        jsonResponse(200, {
          database: { name: 'academy-test-1a2b', hostname: 'academy-test-1a2b.x.turso.io' },
        })
      )
      // createAuthToken → 401
      .mockResolvedValueOnce(jsonResponse(401, {}))
      // deleteDatabase → success
      .mockResolvedValueOnce(jsonResponse(200, {}));
    vi.stubGlobal('fetch', fetchMock);

    await expect(provisionUser('Test')).rejects.toMatchObject({ code: 'AUTH' });

    expect(fetchMock).toHaveBeenCalledTimes(3);
    const [delUrl, delInit] = fetchMock.mock.calls[2] as [string, RequestInit];
    expect(delInit.method).toBe('DELETE');
    expect(delUrl).toContain('/databases/academy-test-1a2b');
    // Migrations were never started (token failed before client creation).
    expect(mocks.createClientMock).not.toHaveBeenCalled();
  });

  it('deletes the created database best-effort when migrations fail', async () => {
    stubTursoEnv();
    const fixtureDir = await createFixtureDir({
      '001_ok.sql': 'CREATE TABLE IF NOT EXISTS fixture_ok (id TEXT PRIMARY KEY);',
      '002_bad.sql': 'THIS IS NOT VALID SQL;',
    });
    vi.stubEnv('TURSO_MIGRATIONS_DIR', fixtureDir);

    const dbPath = tempDbPath();
    const realFileClient = mocks.realCreateClient({ url: `file:${dbPath}` });
    mocks.createClientMock.mockReturnValue(realFileClient);

    const fetchMock = vi.fn();
    fetchMock
      .mockResolvedValueOnce(
        jsonResponse(200, {
          database: { name: 'academy-test-1a2b', hostname: 'academy-test-1a2b.x.turso.io' },
        })
      )
      .mockResolvedValueOnce(jsonResponse(200, { jwt: 'jwt-abc' }))
      .mockResolvedValueOnce(jsonResponse(200, {})); // deleteDatabase cleanup
    vi.stubGlobal('fetch', fetchMock);

    await expect(provisionUser('Test')).rejects.toMatchObject({ code: 'MIGRATION' });

    expect(fetchMock).toHaveBeenCalledTimes(3);
    const [delUrl, delInit] = fetchMock.mock.calls[2] as [string, RequestInit];
    expect(delInit.method).toBe('DELETE');
    expect(delUrl).toContain('/databases/academy-test-1a2b');
  });

  it('surfaces the original error even when cleanup DELETE fails', async () => {
    stubTursoEnv();
    mocks.createClientMock.mockReturnValue(mocks.realCreateClient({ url: `file:${tempDbPath()}` }));

    const fetchMock = vi.fn();
    fetchMock
      .mockResolvedValueOnce(
        jsonResponse(200, {
          database: { name: 'academy-test-1a2b', hostname: 'academy-test-1a2b.x.turso.io' },
        })
      )
      .mockResolvedValueOnce(jsonResponse(401, {})) // createAuthToken fails
      .mockResolvedValueOnce(jsonResponse(500, {})); // cleanup DELETE also fails
    vi.stubGlobal('fetch', fetchMock);

    await expect(provisionUser('Test')).rejects.toMatchObject({ code: 'AUTH' });
  });
});
