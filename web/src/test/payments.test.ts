import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the db module
vi.mock('../lib/db', () => ({
  db: {
    execute: vi.fn().mockResolvedValue({ rows: [] }),
  },
}));

describe('Stripe Webhook Signature', () => {
  it('should reject invalid signatures', async () => {
    // Mock stripe module to use constructEvent that throws on bad sig
    vi.stubEnv('STRIPE_SECRET_KEY', 'sk_test_fake');
    vi.stubEnv('STRIPE_WEBHOOK_SECRET', 'whsec_test_secret');

    const { verifyWebhookSignature } = await import('../lib/payments/stripe');

    expect(() => {
      verifyWebhookSignature('{"type":"test"}', 'invalid_sig');
    }).toThrow();
  });

  it('should normalize checkout.session.completed events', async () => {
    vi.stubEnv('STRIPE_SECRET_KEY', 'sk_test_fake');
    vi.stubEnv('STRIPE_WEBHOOK_SECRET', 'whsec_test_secret');

    const { normalizeEvent } = await import('../lib/payments/stripe');

    const mockEvent = {
      type: 'checkout.session.completed',
      data: {
        object: {
          subscription: 'sub_123',
          customer: 'cus_456',
          client_reference_id: 'user-789',
        },
      },
    } as any;

    const result = normalizeEvent(mockEvent);
    expect(result.type).toBe('checkout.session.completed');
    expect(result.data.subscriptionId).toBe('sub_123');
    expect(result.data.customerId).toBe('cus_456');
    expect(result.data.userId).toBe('user-789');
  });

  it('should normalize invoice.payment_succeeded events', async () => {
    vi.stubEnv('STRIPE_SECRET_KEY', 'sk_test_fake');

    const { normalizeEvent } = await import('../lib/payments/stripe');

    const mockEvent = {
      type: 'invoice.payment_succeeded',
      data: {
        object: {
          subscription: 'sub_123',
          customer: 'cus_456',
          id: 'inv_789',
          amount_paid: 2000,
          currency: 'usd',
        },
      },
    } as any;

    const result = normalizeEvent(mockEvent);
    expect(result.type).toBe('invoice.payment_succeeded');
    expect(result.data.amountPaid).toBe(2000);
    expect(result.data.currency).toBe('usd');
    expect(result.data.invoiceId).toBe('inv_789');
  });

  it('should normalize invoice.payment_failed events', async () => {
    vi.stubEnv('STRIPE_SECRET_KEY', 'sk_test_fake');

    const { normalizeEvent } = await import('../lib/payments/stripe');

    const mockEvent = {
      type: 'invoice.payment_failed',
      data: {
        object: {
          subscription: 'sub_fail',
          customer: 'cus_fail',
          id: 'inv_fail',
        },
      },
    } as any;

    const result = normalizeEvent(mockEvent);
    expect(result.type).toBe('invoice.payment_failed');
    expect(result.data.subscriptionId).toBe('sub_fail');
  });

  it('should normalize customer.subscription.deleted events', async () => {
    vi.stubEnv('STRIPE_SECRET_KEY', 'sk_test_fake');

    const { normalizeEvent } = await import('../lib/payments/stripe');

    const mockEvent = {
      type: 'customer.subscription.deleted',
      data: {
        object: {
          id: 'sub_del',
          customer: 'cus_del',
        },
      },
    } as any;

    const result = normalizeEvent(mockEvent);
    expect(result.type).toBe('customer.subscription.deleted');
    expect(result.data.subscriptionId).toBe('sub_del');
    expect(result.data.customerId).toBe('cus_del');
  });
});

describe('Subscription Lifecycle', () => {
  let dbMock: { execute: ReturnType<typeof vi.fn> };

  beforeEach(async () => {
    vi.resetModules();
    dbMock = { execute: vi.fn().mockResolvedValue({ rows: [] }) };
    vi.doMock('../lib/db', () => ({ db: dbMock }));
  });

  it('should create a trial subscription', async () => {
    const { createTrialSubscription } = await import('../lib/payments/lifecycle');

    await createTrialSubscription('user-1', 'basic', 'sub_stripe_1');

    expect(dbMock.execute).toHaveBeenCalledWith(
      expect.objectContaining({
        sql: expect.stringContaining("INSERT INTO subscriptions"),
        args: expect.arrayContaining(['user-1', 'basic']),
      })
    );

    const args = dbMock.execute.mock.calls[0][0].args;
    // Check status is included via SQL literal 'trial'
    expect(dbMock.execute.mock.calls[0][0].sql).toContain("'trial'");
  });

  it('should activate a subscription clearing trial/grace fields', async () => {
    const { activateSubscription } = await import('../lib/payments/lifecycle');

    await activateSubscription('sub-id-1');

    expect(dbMock.execute).toHaveBeenCalledWith(
      expect.objectContaining({
        sql: expect.stringContaining("status = 'active'"),
        args: expect.arrayContaining(['sub-id-1']),
      })
    );

    expect(dbMock.execute.mock.calls[0][0].sql).toContain('trial_starts_at = NULL');
    expect(dbMock.execute.mock.calls[0][0].sql).toContain('grace_expires_at = NULL');
  });

  it('should start a grace period', async () => {
    const { startGracePeriod } = await import('../lib/payments/lifecycle');

    await startGracePeriod('sub-id-2');

    expect(dbMock.execute).toHaveBeenCalledWith(
      expect.objectContaining({
        sql: expect.stringContaining("status = 'grace'"),
        args: expect.arrayContaining(['sub-id-2']),
      })
    );
  });

  it('should expire a subscription', async () => {
    const { expireSubscription } = await import('../lib/payments/lifecycle');

    await expireSubscription('sub-id-3');

    expect(dbMock.execute).toHaveBeenCalledWith(
      expect.objectContaining({
        sql: expect.stringContaining("status = 'expired'"),
        args: ['sub-id-3'],
      })
    );
  });

  it('should cancel a subscription', async () => {
    const { cancelSubscription } = await import('../lib/payments/lifecycle');

    await cancelSubscription('sub-id-4');

    expect(dbMock.execute).toHaveBeenCalledWith(
      expect.objectContaining({
        sql: expect.stringContaining("status = 'cancelled'"),
        args: ['sub-id-4'],
      })
    );
  });

  it('should query expired grace subscriptions', async () => {
    dbMock.execute.mockResolvedValueOnce({
      rows: [
        {
          id: 'sub-expired',
          user_id: 'u1',
          plan: 'basic',
          status: 'grace',
          trial_start: null,
          trial_end: null,
          grace_start: '2024-01-01T00:00:00Z',
          grace_end: '2024-01-08T00:00:00Z',
          stripe_subscription_id: 'sub_stripe',
          current_period_start: null,
          current_period_end: null,
        },
      ],
    });

    const { getExpiredGraceSubscriptions } = await import('../lib/payments/lifecycle');
    const result = await getExpiredGraceSubscriptions();

    expect(result).toHaveLength(1);
    expect(result[0].id).toBe('sub-expired');
    expect(result[0].status).toBe('grace');

    expect(dbMock.execute).toHaveBeenCalledWith(
      expect.objectContaining({
        sql: expect.stringContaining("status = 'grace' AND grace_expires_at < ?"),
      })
    );
  });

  it('should transition trial → active → grace → expired correctly', async () => {
    const lifecycle = await import('../lib/payments/lifecycle');

    // Trial
    await lifecycle.createTrialSubscription('user-x', 'pro', 'sub_x');
    expect(dbMock.execute).toHaveBeenCalledTimes(1);

    // Active
    dbMock.execute.mockClear();
    await lifecycle.activateSubscription('sub-id-x');
    expect(dbMock.execute.mock.calls[0][0].sql).toContain("'active'");

    // Grace
    dbMock.execute.mockClear();
    await lifecycle.startGracePeriod('sub-id-x');
    expect(dbMock.execute.mock.calls[0][0].sql).toContain("'grace'");

    // Expired
    dbMock.execute.mockClear();
    await lifecycle.expireSubscription('sub-id-x');
    expect(dbMock.execute.mock.calls[0][0].sql).toContain("'expired'");
  });
});
