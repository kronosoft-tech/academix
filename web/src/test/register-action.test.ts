import { describe, it, expect, vi, beforeEach } from 'vitest';
import { registerSchema, registerHandler } from '../actions/register';
import { ProvisioningError } from '../lib/provisioning';

/**
 * Action-level tests for the register action (Phase 3, task 3.6).
 *
 * The action's public surface (`register`) is the Astro-safe handler wrapper;
 * we test the exported `registerSchema` (R1) and `registerHandler` (R4/R5/R6)
 * directly. `astro:actions` is mocked because it is a virtual module; the
 * shared DB, auth helpers, provisioning library and @libsql/client are mocked
 * so no network or real Turso DB is touched.
 */

// --- Module mocks ---

const astroActions = vi.hoisted(() => {
  class ActionError extends Error {
    code: string;
    constructor(params: { code: string; message?: string }) {
      super(params.message);
      this.name = 'ActionError';
      this.code = params.code;
    }
  }
  return {
    ActionError,
    defineAction: (opts: unknown) => opts,
  };
});

vi.mock('astro:actions', () => astroActions);

const dbMocks = vi.hoisted(() => ({
  execute: vi.fn(),
}));

vi.mock('../lib/db', () => ({ db: { execute: dbMocks.execute } }));

const authMocks = vi.hoisted(() => ({
  hashPassword: vi.fn(),
  signToken: vi.fn(),
  setAuthCookie: vi.fn(),
}));

vi.mock('../lib/auth', () => authMocks);

const provisioningMocks = vi.hoisted(() => ({
  provisionUser: vi.fn(),
  deleteDatabase: vi.fn(),
  getTursoEnv: vi.fn(),
}));

vi.mock('../lib/provisioning', async (importOriginal) => {
  // Keep the real ProvisioningError class (handler relies on `instanceof`).
  const actual = await importOriginal<typeof import('../lib/provisioning')>();
  return { ...actual, ...provisioningMocks };
});

const lifecycleMocks = vi.hoisted(() => ({
  createTrialSubscription: vi.fn(),
}));

vi.mock('../lib/payments/lifecycle', () => lifecycleMocks);

const libsqlMocks = vi.hoisted(() => ({
  createClient: vi.fn(),
  perUserExecute: vi.fn(),
  perUserClose: vi.fn(),
}));

vi.mock('@libsql/client', () => ({ createClient: libsqlMocks.createClient }));

// --- Fixtures ---

const validInput = {
  name: 'Ana Gómez',
  email: 'ana@example.com',
  password: 'secret-pass-123',
  academyName: 'Music School',
  confirmPassword: 'secret-pass-123',
};

const provisionResult = {
  dbName: 'academy-music-school-abcd',
  dbUrl: 'libsql://academy-music-school-abcd.turso.io',
  dbToken: 'per-user-token',
  hostname: 'academy-music-school-abcd.turso.io',
};

// Records the exact R4 execution order across all mocked collaborators.
const order: string[] = [];

const handlerContext = {
  cookies: { set: vi.fn() },
} as unknown as Parameters<typeof registerHandler>[1];

beforeEach(() => {
  order.length = 0;

  for (const mock of [
    dbMocks.execute,
    authMocks.hashPassword,
    authMocks.signToken,
    authMocks.setAuthCookie,
    provisioningMocks.provisionUser,
    provisioningMocks.deleteDatabase,
    provisioningMocks.getTursoEnv,
    lifecycleMocks.createTrialSubscription,
    libsqlMocks.createClient,
    libsqlMocks.perUserExecute,
    libsqlMocks.perUserClose,
  ]) {
    mock.mockReset();
  }

  // Default implementations; individual tests override where needed.
  authMocks.hashPassword.mockImplementation(async () => {
    order.push('hash-password');
    return 'hashed-password';
  });
  authMocks.signToken.mockImplementation(async () => {
    order.push('sign-token');
    return 'jwt-token';
  });
  authMocks.setAuthCookie.mockImplementation(() => {
    order.push('set-cookie');
  });
  provisioningMocks.getTursoEnv.mockImplementation(() => ({
    apiToken: 'test-token',
    org: 'academix',
    group: 'default',
  }));
  provisioningMocks.deleteDatabase.mockImplementation(async () => {
    order.push('delete-database');
  });
  lifecycleMocks.createTrialSubscription.mockImplementation(async () => {
    order.push('create-trial');
  });
  libsqlMocks.perUserExecute.mockImplementation(async () => {
    order.push('per-user-users-insert');
    return { rows: [] };
  });
  libsqlMocks.perUserClose.mockImplementation(() => undefined);
  libsqlMocks.createClient.mockImplementation(() => ({
    execute: libsqlMocks.perUserExecute,
    close: libsqlMocks.perUserClose,
  }));
  dbMocks.execute.mockImplementation(async ({ sql }: { sql: string }) => {
    if (sql.startsWith('SELECT id FROM users')) {
      order.push('email-exists-check');
    } else if (sql.includes('user_databases')) {
      order.push('user-databases-upsert');
    } else {
      order.push('shared-users-insert');
    }
    return { rows: [] };
  });
});

describe('registerSchema (R1)', () => {
  it('accepts a valid payload and trims academyName server-side', () => {
    const parsed = registerSchema.safeParse({
      ...validInput,
      academyName: '  Music School  ',
    });
    expect(parsed.success).toBe(true);
    if (parsed.success) {
      expect(parsed.data.academyName).toBe('Music School');
    }
  });

  it('rejects a whitespace-only academyName', () => {
    const parsed = registerSchema.safeParse({ ...validInput, academyName: '   ' });
    expect(parsed.success).toBe(false);
    if (!parsed.success) {
      expect(parsed.error.issues.some((issue) => issue.path[0] === 'academyName')).toBe(true);
    }
  });

  it('rejects a password confirmation mismatch on confirmPassword', () => {
    const parsed = registerSchema.safeParse({
      ...validInput,
      confirmPassword: 'different-pass-123',
    });
    expect(parsed.success).toBe(false);
    if (!parsed.success) {
      expect(parsed.error.issues.some((issue) => issue.path[0] === 'confirmPassword')).toBe(true);
    }
  });

  it('does not accept a client-supplied slug field', () => {
    const parsed = registerSchema.safeParse({ ...validInput, slug: 'client-slug' });
    expect(parsed.success).toBe(true);
    if (parsed.success) {
      expect(parsed.data).not.toHaveProperty('slug');
    }
  });
});

describe('registerHandler happy path (R4)', () => {
  it('provisions, writes all rows in order, signs the JWT and sets the cookie', async () => {
    provisioningMocks.provisionUser.mockImplementation(async () => {
      order.push('provision-db');
      return provisionResult;
    });

    const result = await registerHandler(validInput, handlerContext);

    expect(result).toEqual({ success: true });
    expect(provisioningMocks.provisionUser).toHaveBeenCalledWith('Music School');

    // Exact R4 ordering: email check → hash → provision → per-user → shared →
    // user_databases → trial → signToken → cookie.
    expect(order).toEqual([
      'email-exists-check',
      'hash-password',
      'provision-db',
      'per-user-users-insert',
      'shared-users-insert',
      'user-databases-upsert',
      'create-trial',
      'sign-token',
      'set-cookie',
    ]);

    // Per-user Admin row (R4).
    const perUserInsert = libsqlMocks.perUserExecute.mock.calls.find((call) =>
      (call[0].sql as string).includes('INSERT INTO users')
    );
    expect(perUserInsert).toBeDefined();
    expect(perUserInsert![0].sql).toContain("'Admin'");
    expect(perUserInsert![0].args[1]).toBe('ana@example.com');
    expect(perUserInsert![0].args[2]).toBe('hashed-password');
    expect(libsqlMocks.perUserClose).toHaveBeenCalled();

    // Shared users row (R4).
    const sharedInsert = dbMocks.execute.mock.calls.find((call) =>
      (call[0].sql as string).includes('INSERT INTO users')
    );
    expect(sharedInsert).toBeDefined();

    // user_databases upsert incl. academy_name (R4).
    const dbMappingCall = dbMocks.execute.mock.calls.find((call) =>
      (call[0].sql as string).includes('INSERT OR REPLACE INTO user_databases')
    );
    expect(dbMappingCall).toBeDefined();
    expect(dbMappingCall![0].args).toEqual([
      expect.any(String),
      'ana@example.com',
      'Music School',
      provisionResult.dbUrl,
      provisionResult.dbToken,
      'academix',
      expect.any(String),
    ]);

    // Shared trial only (R7): createTrialSubscription called on the shared DB
    // and the per-user DB never sees a subscription write.
    expect(lifecycleMocks.createTrialSubscription).toHaveBeenCalledWith(
      expect.any(String),
      'trial',
      null
    );
    const perUserSubscriptionWrites = libsqlMocks.perUserExecute.mock.calls.filter((call) =>
      (call[0].sql as string).toLowerCase().includes('subscriptions')
    );
    expect(perUserSubscriptionWrites).toHaveLength(0);

    // JWT claims (R4) — dbUrl/dbToken/academyName resolve the TS2345.
    expect(authMocks.signToken).toHaveBeenCalledWith(
      expect.objectContaining({
        sub: expect.any(String),
        email: 'ana@example.com',
        role: 'Admin',
        type: 'customer',
        dbUrl: provisionResult.dbUrl,
        dbToken: provisionResult.dbToken,
        academyName: 'Music School',
      })
    );

    // Cookie set with the signed JWT; no cleanup.
    expect(authMocks.setAuthCookie).toHaveBeenCalledWith(handlerContext.cookies, 'jwt-token');
    expect(provisioningMocks.deleteDatabase).not.toHaveBeenCalled();
  });
});

describe('registerHandler failure paths (R4/R5/R6)', () => {
  it('returns CONFLICT for a duplicate email without provisioning anything', async () => {
    dbMocks.execute.mockImplementation(async () => ({ rows: [{ id: 'existing-user' }] }));

    await expect(registerHandler(validInput, handlerContext)).rejects.toMatchObject({
      code: 'CONFLICT',
      message: 'Ya existe una cuenta con este email',
    });

    expect(provisioningMocks.provisionUser).not.toHaveBeenCalled();
    expect(libsqlMocks.createClient).not.toHaveBeenCalled();
    expect(lifecycleMocks.createTrialSubscription).not.toHaveBeenCalled();
    expect(authMocks.signToken).not.toHaveBeenCalled();
    expect(authMocks.setAuthCookie).not.toHaveBeenCalled();
    expect(provisioningMocks.deleteDatabase).not.toHaveBeenCalled();
  });

  it('fails closed when Turso env vars are missing (MISSING_ENV)', async () => {
    provisioningMocks.provisionUser.mockRejectedValue(
      new ProvisioningError('MISSING_ENV', 'Turso provisioning requires TURSO_API_TOKEN')
    );

    await expect(registerHandler(validInput, handlerContext)).rejects.toMatchObject({
      code: 'INTERNAL_SERVER_ERROR',
      message: 'Registro no disponible temporalmente',
    });

    expect(provisioningMocks.deleteDatabase).not.toHaveBeenCalled();
    expect(authMocks.signToken).not.toHaveBeenCalled();
    expect(authMocks.setAuthCookie).not.toHaveBeenCalled();
  });

  it('maps createDatabase exhaustion (CONFLICT) to a clear error with no cleanup', async () => {
    provisioningMocks.provisionUser.mockRejectedValue(
      new ProvisioningError('CONFLICT', 'all name attempts conflicted')
    );

    await expect(registerHandler(validInput, handlerContext)).rejects.toMatchObject({
      code: 'INTERNAL_SERVER_ERROR',
      message: 'No se pudo crear tu academia, intenta de nuevo',
    });

    expect(provisioningMocks.deleteDatabase).not.toHaveBeenCalled();
    expect(authMocks.signToken).not.toHaveBeenCalled();
  });

  it('maps post-create provision failures (MIGRATION) to the generic error; no double cleanup', async () => {
    provisioningMocks.provisionUser.mockRejectedValue(
      new ProvisioningError('MIGRATION', 'Migration 005 failed')
    );

    await expect(registerHandler(validInput, handlerContext)).rejects.toMatchObject({
      code: 'INTERNAL_SERVER_ERROR',
      message: 'No se pudo completar el registro, intenta de nuevo',
    });

    // provisionUser already DELETEd the DB internally — the handler must not
    // attempt a second DELETE (R5 best-effort cleanup semantics).
    expect(provisioningMocks.deleteDatabase).not.toHaveBeenCalled();
    expect(authMocks.signToken).not.toHaveBeenCalled();
  });

  it('DELETEs the provisioned DB when a later shared write fails and signs no JWT', async () => {
    provisioningMocks.provisionUser.mockResolvedValue(provisionResult);
    dbMocks.execute.mockImplementation(async ({ sql }: { sql: string }) => {
      if (sql.startsWith('SELECT id FROM users')) return { rows: [] };
      throw new Error('shared db unavailable');
    });

    await expect(registerHandler(validInput, handlerContext)).rejects.toMatchObject({
      code: 'INTERNAL_SERVER_ERROR',
      message: 'No se pudo completar el registro, intenta de nuevo',
    });

    expect(provisioningMocks.deleteDatabase).toHaveBeenCalledWith('academix', provisionResult.dbName);
    expect(authMocks.signToken).not.toHaveBeenCalled();
    expect(authMocks.setAuthCookie).not.toHaveBeenCalled();
  });

  it('DELETEs the provisioned DB when signToken fails and signs no JWT', async () => {
    provisioningMocks.provisionUser.mockResolvedValue(provisionResult);
    authMocks.signToken.mockRejectedValue(new Error('jwt signing failed'));

    await expect(registerHandler(validInput, handlerContext)).rejects.toMatchObject({
      code: 'INTERNAL_SERVER_ERROR',
      message: 'No se pudo completar el registro, intenta de nuevo',
    });

    expect(provisioningMocks.deleteDatabase).toHaveBeenCalledWith('academix', provisionResult.dbName);
    expect(authMocks.setAuthCookie).not.toHaveBeenCalled();
  });

  it('never returns a JWT on any failure (fail closed)', async () => {
    provisioningMocks.provisionUser.mockResolvedValue(provisionResult);
    // Fail at the per-user users insert, the first step after provisioning.
    libsqlMocks.perUserExecute.mockRejectedValue(new Error('per-user insert failed'));

    await expect(registerHandler(validInput, handlerContext)).rejects.toMatchObject({
      code: 'INTERNAL_SERVER_ERROR',
      message: 'No se pudo completar el registro, intenta de nuevo',
    });

    expect(provisioningMocks.deleteDatabase).toHaveBeenCalledWith('academix', provisionResult.dbName);
    expect(authMocks.signToken).not.toHaveBeenCalled();
    expect(authMocks.setAuthCookie).not.toHaveBeenCalled();
  });
});
