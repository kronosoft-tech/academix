import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the db module
vi.mock('../lib/db', () => ({
  db: {
    execute: vi.fn().mockResolvedValue({ rows: [] }),
  },
}));

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
          plan_id: 'pro',
          status: 'grace',
          trial_end: null,
          grace_expires_at: '2024-01-08T00:00:00Z',
          stripe_subscription_id: 'sub_stripe',
          current_period_start: null,
          current_period_end: null,
          provider: null,
          provider_subscription_id: null,
          provider_customer_id: null,
          trial_starts_at: null,
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
