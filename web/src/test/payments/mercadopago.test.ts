/**
 * Mercado Pago Webhook Signature & Handler Tests
 *
 * Unit + integration tests for MP webhook signature verification:
 * - verifyWebhookSignature: valid HMAC → true; missing/malformed headers → false (never throws)
 * - POST handler: missing secret → 500 (gate preserved); invalid signature → 401 (not 500)
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createHmac } from 'node:crypto';

// --- Mocks for real mercadopago.ts dependencies (used via vi.importActual) ---
vi.mock('../../lib/db', () => ({
  db: { execute: vi.fn().mockResolvedValue({ rows: [] }) },
}));

// --- Mocks for handler integration tests ---
const mocks = vi.hoisted(() => ({
  getWebhookSecret: vi.fn(),
  verifyWebhookSignature: vi.fn(),
  getPayment: vi.fn(),
  activateApprovedPayment: vi.fn(),
}));

vi.mock('../../lib/payments/mercadopago', () => ({
  getWebhookSecret: mocks.getWebhookSecret,
  verifyWebhookSignature: mocks.verifyWebhookSignature,
  getPayment: mocks.getPayment,
  activateApprovedPayment: mocks.activateApprovedPayment,
}));

// Import handler (resolves to the mocked mercadopago module)
import { POST } from '../../pages/api/webhooks/mercadopago';

// --- Test helpers ---

const TEST_SECRET = 'test-webhook-secret';
const DATA_ID = '123456789';
const X_REQUEST_ID = 'req-abc-123';
const TS = '1700000000';

/** Compute a valid x-signature for the given parameters. */
function computeValidSignature(
  dataId: string,
  xRequestId: string,
  ts: string,
  secret: string
): string {
  const manifest = `id:${dataId.toLowerCase()};request-id:${xRequestId};ts:${ts};`;
  const hmac = createHmac('sha256', secret).update(manifest).digest('hex');
  return `ts=${ts},v1=${hmac}`;
}

/** Build a minimal Request for the POST handler. */
function makeRequest(
  body: unknown,
  headers: Record<string, string> = {}
): Request {
  return new Request('https://localhost/api/webhooks/mercadopago', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...headers },
    body: JSON.stringify(body),
  });
}

type ApiRouteContext = Parameters<typeof POST>[0];

function makeContext(request: Request): ApiRouteContext {
  return { request } as unknown as ApiRouteContext;
}

const APPROVED_PAYMENT = {
  id: 123456789,
  status: 'approved',
  status_detail: 'accredited',
  external_reference:
    '11111111-2222-3333-4444-555555555555-pro-aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee',
  transaction_amount: 89900,
  currency_id: 'COP',
  payer: { email: 'test@test.com' },
};

beforeEach(() => {
  vi.clearAllMocks();
  // Default handler-level mocks
  mocks.getWebhookSecret.mockReturnValue(TEST_SECRET);
  mocks.verifyWebhookSignature.mockReturnValue(true);
  mocks.getPayment.mockResolvedValue(APPROVED_PAYMENT);
  mocks.activateApprovedPayment.mockResolvedValue(undefined);
});

// --- Unit tests: verifyWebhookSignature (real implementation via vi.importActual) ---

describe('verifyWebhookSignature', () => {
  it('returns true for a valid HMAC-SHA256 signature', async () => {
    const { verifyWebhookSignature } = (await vi.importActual(
      '../../lib/payments/mercadopago'
    )) as { verifyWebhookSignature: (
      xSignature: string,
      xRequestId: string,
      dataId: string,
      secret: string
    ) => boolean };

    const sig = computeValidSignature(DATA_ID, X_REQUEST_ID, TS, TEST_SECRET);
    expect(
      verifyWebhookSignature(sig, X_REQUEST_ID, DATA_ID, TEST_SECRET)
    ).toBe(true);
  });

  it('returns false for empty/missing x-signature (never throws)', async () => {
    const { verifyWebhookSignature } = (await vi.importActual(
      '../../lib/payments/mercadopago'
    )) as { verifyWebhookSignature: (
      xSignature: string,
      xRequestId: string,
      dataId: string,
      secret: string
    ) => boolean };

    expect(
      verifyWebhookSignature('', X_REQUEST_ID, DATA_ID, TEST_SECRET)
    ).toBe(false);
  });

  it('returns false for malformed non-hex v1 (never throws)', async () => {
    const { verifyWebhookSignature } = (await vi.importActual(
      '../../lib/payments/mercadopago'
    )) as { verifyWebhookSignature: (
      xSignature: string,
      xRequestId: string,
      dataId: string,
      secret: string
    ) => boolean };

    expect(
      verifyWebhookSignature(
        `ts=${TS},v1=zzzz`,
        X_REQUEST_ID,
        DATA_ID,
        TEST_SECRET
      )
    ).toBe(false);
  });
});

// --- Handler integration tests ---

describe('POST /api/webhooks/mercadopago', () => {
  it('returns 401 when x-signature is missing (not 500)', async () => {
    mocks.verifyWebhookSignature.mockReturnValue(false);

    const res = await POST(
      makeContext(
        makeRequest({ action: 'payment.created', data: { id: DATA_ID } })
      )
    );

    expect(res.status).toBe(401);
    const body = await res.json();
    expect(body.error).toBe('Invalid signature');
  });

  it('returns 401 for malformed non-hex v1 (not 500)', async () => {
    mocks.verifyWebhookSignature.mockReturnValue(false);

    const res = await POST(
      makeContext(
        makeRequest(
          { action: 'payment.created', data: { id: DATA_ID } },
          { 'x-signature': `ts=${TS},v1=zzzz` }
        )
      )
    );

    expect(res.status).toBe(401);
    const body = await res.json();
    expect(body.error).toBe('Invalid signature');
  });

  it('returns 500 when MP_WEBHOOK_SECRET is not configured (gate preserved)', async () => {
    mocks.getWebhookSecret.mockReturnValue('');

    const res = await POST(
      makeContext(
        makeRequest(
          { action: 'payment.created', data: { id: DATA_ID } },
          { 'x-signature': `ts=${TS},v1=abc` }
        )
      )
    );

    expect(res.status).toBe(500);
    const body = await res.json();
    expect(body.error).toContain('not configured');
  });

  it('returns 200 when signature is valid and payment is approved', async () => {
    mocks.verifyWebhookSignature.mockReturnValue(true);

    const res = await POST(
      makeContext(
        makeRequest({ action: 'payment.created', data: { id: DATA_ID } })
      )
    );

    expect(res.status).toBe(200);
    expect(mocks.getPayment).toHaveBeenCalledWith(DATA_ID);
    expect(mocks.activateApprovedPayment).toHaveBeenCalled();
  });
});
