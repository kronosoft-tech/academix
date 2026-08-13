/**
 * verify-mercadopago endpoint tests.
 *
 * Exercises GET /api/payments/verify-mercadopago (the dashboard-side payment
 * persistence fallback for Mercado Pago Checkout Pro redirects) through the
 * real APIRoute handler with mocked auth, MP API, and database dependencies.
 *
 * The webhook never passes the ownership guard, so dedicated tests exercise
 * the REAL activateApprovedPayment (via vi.importActual, with the mocked db)
 * to prove the guard changes nothing for webhook-style calls.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import type { CustomerJwtPayload } from '../../lib/auth';
import { GET } from '../../pages/api/payments/verify-mercadopago';

const mocks = vi.hoisted(() => ({
  getFullTokenPayload: vi.fn(),
  getPayment: vi.fn(),
  activateApprovedPayment: vi.fn(),
  execute: vi.fn<
    (query: { sql: string; args?: unknown[] }) => Promise<{
      rows: Array<Record<string, unknown>>;
    }>
  >(),
}));

vi.mock('../../lib/auth', () => ({
  getFullTokenPayload: mocks.getFullTokenPayload,
}));

vi.mock('../../lib/payments/mercadopago', () => ({
  getPayment: mocks.getPayment,
  activateApprovedPayment: mocks.activateApprovedPayment,
}));

vi.mock('../../lib/db', () => ({
  db: { execute: mocks.execute },
}));

// Reference format: {userId}-{planId}-{uuid}; userId is 5 UUID parts, plan is part 6.
const SUB = '11111111-2222-3333-4444-555555555555';
const FOREIGN_SUB = '99999999-8888-7777-6666-555555555555';
const EXTERNAL_REFERENCE = `${SUB}-pro-aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee`;
const FOREIGN_REFERENCE = `${FOREIGN_SUB}-pro-aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee`;
const PAYMENT_ID = '123456789';

const APPROVED_PAYMENT = {
  id: 123456789,
  status: 'approved',
  status_detail: 'accredited',
  external_reference: EXTERNAL_REFERENCE,
  transaction_amount: 89900,
  currency_id: 'COP',
  payer: { email: 'user@test.com' },
};

function makePayload(): CustomerJwtPayload {
  return {
    sub: SUB,
    email: 'user@test.com',
    role: 'customer',
    type: 'customer',
    dbUrl: 'libsql://academix.example.com',
    dbToken: 'test-token',
    academyName: 'Test Academy',
    iat: 1,
    exp: 2,
  };
}

function makeContext(query: string): Parameters<typeof GET>[0] {
  const url = `http://localhost/api/payments/verify-mercadopago${query}`;
  return {
    request: new Request(url),
    url: new URL(url),
    cookies: {},
  } as unknown as Parameters<typeof GET>[0];
}

const happyQuery = `?payment_id=${PAYMENT_ID}&external_reference=${EXTERNAL_REFERENCE}`;

function insertCalls(): Array<[{ sql: string; args?: unknown[] }]> {
  return mocks.execute.mock.calls.filter(([query]) =>
    query.sql.includes('INSERT INTO subscription_payments')
  );
}

beforeEach(() => {
  mocks.getFullTokenPayload.mockReset().mockResolvedValue(makePayload());
  mocks.getPayment.mockReset().mockResolvedValue(APPROVED_PAYMENT);
  mocks.activateApprovedPayment.mockReset().mockResolvedValue(undefined);
  mocks.execute.mockReset().mockResolvedValue({ rows: [] });
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe('GET verify-mercadopago — happy path', () => {
  it('returns 200 { success, plan, status } and activates with expectedUserId', async () => {
    const res = await GET(makeContext(happyQuery));

    expect(res.status).toBe(200);
    await expect(res.json()).resolves.toEqual({
      success: true,
      plan: 'pro',
      status: 'active',
    });
    expect(mocks.activateApprovedPayment).toHaveBeenCalledWith({
      paymentId: PAYMENT_ID,
      externalReference: EXTERNAL_REFERENCE,
      transactionAmount: 89900,
      currencyId: 'COP',
      expectedUserId: SUB,
    });
  });

  it('accepts collection_id as an alias for payment_id', async () => {
    const res = await GET(
      makeContext(
        `?collection_id=${PAYMENT_ID}&external_reference=${EXTERNAL_REFERENCE}`
      )
    );

    expect(res.status).toBe(200);
    expect(mocks.activateApprovedPayment).toHaveBeenCalledWith(
      expect.objectContaining({ paymentId: PAYMENT_ID, expectedUserId: SUB })
    );
  });
});

describe('GET verify-mercadopago — authentication', () => {
  it('returns 401 without a valid JWT and never touches MP or the DB', async () => {
    mocks.getFullTokenPayload.mockResolvedValue(null);

    const res = await GET(makeContext(happyQuery));

    expect(res.status).toBe(401);
    const body = await res.json();
    expect(body).toMatchObject({ success: false });
    expect(body.message).toBeTruthy();
    expect(mocks.getPayment).not.toHaveBeenCalled();
    expect(mocks.activateApprovedPayment).not.toHaveBeenCalled();
    expect(mocks.execute).not.toHaveBeenCalled();
  });

  it('returns 400 when payment_id is missing', async () => {
    const res = await GET(makeContext(`?external_reference=${EXTERNAL_REFERENCE}`));

    expect(res.status).toBe(400);
    expect(mocks.getPayment).not.toHaveBeenCalled();
    expect(mocks.activateApprovedPayment).not.toHaveBeenCalled();
  });

  it('returns 400 when external_reference is missing', async () => {
    const res = await GET(makeContext(`?payment_id=${PAYMENT_ID}`));

    expect(res.status).toBe(400);
    expect(mocks.getPayment).not.toHaveBeenCalled();
    expect(mocks.activateApprovedPayment).not.toHaveBeenCalled();
  });
});

describe('GET verify-mercadopago — ownership', () => {
  it('returns 403 when the reference does not start with the JWT sub', async () => {
    const res = await GET(
      makeContext(`?payment_id=${PAYMENT_ID}&external_reference=${FOREIGN_REFERENCE}`)
    );

    expect(res.status).toBe(403);
    const body = await res.json();
    expect(body).toMatchObject({ success: false });
    expect(body.message).toBeTruthy();
    expect(mocks.getPayment).not.toHaveBeenCalled();
    expect(mocks.activateApprovedPayment).not.toHaveBeenCalled();
    expect(mocks.execute).not.toHaveBeenCalled();
  });

  it('activateApprovedPayment early-returns when expectedUserId does not prefix the reference (real impl)', async () => {
    const real = await vi.importActual<
      typeof import('../../lib/payments/mercadopago')
    >('../../lib/payments/mercadopago');

    await real.activateApprovedPayment({
      paymentId: PAYMENT_ID,
      externalReference: FOREIGN_REFERENCE,
      transactionAmount: 89900,
      currencyId: 'COP',
      expectedUserId: SUB,
    });

    expect(mocks.execute).not.toHaveBeenCalled();
  });

  it('activateApprovedPayment proceeds normally when expectedUserId is omitted (webhook behavior, real impl)', async () => {
    mocks.execute.mockImplementation((query) => {
      if (query.sql.includes('SELECT id FROM subscriptions')) {
        return Promise.resolve({ rows: [{ id: 'sub-webhook' }] });
      }
      return Promise.resolve({ rows: [] });
    });
    const real = await vi.importActual<
      typeof import('../../lib/payments/mercadopago')
    >('../../lib/payments/mercadopago');

    await real.activateApprovedPayment({
      paymentId: PAYMENT_ID,
      externalReference: EXTERNAL_REFERENCE,
      transactionAmount: 89900,
      currencyId: 'COP',
    });

    expect(mocks.execute).toHaveBeenCalled();
    expect(insertCalls()).toHaveLength(1);
  });
});

describe('GET verify-mercadopago — idempotency', () => {
  it('returns 200 on repeated verify calls (dedupe is delegated to activation)', async () => {
    const first = await GET(makeContext(happyQuery));
    const second = await GET(makeContext(happyQuery));

    expect(first.status).toBe(200);
    expect(second.status).toBe(200);
    // The endpoint is safe to re-call; the replay guard inside
    // activateApprovedPayment prevents double persistence (covered below).
    expect(mocks.activateApprovedPayment).toHaveBeenCalledTimes(2);
  });

  it('persists the payment INSERT exactly once across repeated activation (real impl, mocked db)', async () => {
    let replaySelects = 0;
    mocks.execute.mockImplementation((query) => {
      if (query.sql.includes('SELECT id FROM subscription_payments')) {
        replaySelects += 1;
        // Call 1 runs two replay SELECTs (before activation and before the
        // INSERT). From the third replay SELECT on (call 2), report the
        // payment as already recorded so the real impl early-returns.
        return Promise.resolve(
          replaySelects >= 3 ? { rows: [{ id: 'pay-1' }] } : { rows: [] }
        );
      }
      if (query.sql.includes('SELECT id FROM subscriptions')) {
        return Promise.resolve({ rows: [{ id: 'sub-1' }] });
      }
      return Promise.resolve({ rows: [] });
    });
    const real = await vi.importActual<
      typeof import('../../lib/payments/mercadopago')
    >('../../lib/payments/mercadopago');
    mocks.activateApprovedPayment.mockImplementation(real.activateApprovedPayment);

    const first = await GET(makeContext(happyQuery));
    const second = await GET(makeContext(happyQuery));

    expect(first.status).toBe(200);
    expect(second.status).toBe(200);
    expect(insertCalls()).toHaveLength(1);
  });
});

describe('GET verify-mercadopago — MP API failure', () => {
  it('returns 502, logs, and persists nothing when getPayment fails', async () => {
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    mocks.getPayment.mockRejectedValue(
      new Error('MercadoPago getPayment failed: 500 - boom')
    );

    const res = await GET(makeContext(happyQuery));

    expect(res.status).toBe(502);
    const body = await res.json();
    expect(body).toMatchObject({ success: false });
    expect(body.message).toBeTruthy();
    expect(errorSpy).toHaveBeenCalled();
    expect(mocks.activateApprovedPayment).not.toHaveBeenCalled();
    expect(mocks.execute).not.toHaveBeenCalled();
  });

  it('returns 400 with the MP status when the payment is not approved', async () => {
    mocks.getPayment.mockResolvedValue({ ...APPROVED_PAYMENT, status: 'pending' });

    const res = await GET(makeContext(happyQuery));

    expect(res.status).toBe(400);
    const body = await res.json();
    expect(body).toMatchObject({ success: false, status: 'pending' });
    expect(mocks.activateApprovedPayment).not.toHaveBeenCalled();
    expect(mocks.execute).not.toHaveBeenCalled();
  });
});
