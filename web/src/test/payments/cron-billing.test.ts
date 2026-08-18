/**
 * Cron Billing Tests
 *
 * Verifies the three billing crons against real schema columns:
 * - R1: charge-wompi selects `s.plan_id` with provider/status/period/token
 *   filters and charges the current `PLANS` price.
 * - R2: unknown plan id or a charge failure starts grace — no silent price.
 * - R3: send-reminders uses `s.grace_expires_at` and reports graceWarnings.
 * - R4: grace expiry → `cancelSubscription`; trial expiry → `expireSubscription`.
 * - R5: unexpected handler error → 500 + alert email; 401 → no email.
 * - R6: getPlanById resolves real PLANS prices; unknown → undefined.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// --- Mocks (mirror webhook-flows.test.ts pattern) ---
const mockExecute = vi.fn();
const mockCreateTransaction = vi.fn();
const mockStartGrace = vi.fn();
const mockCancel = vi.fn();
const mockExpire = vi.fn();
const mockGetExpiredGrace = vi.fn();
const mockGetExpiredTrials = vi.fn();
const mockSendTrialReminder = vi.fn();
const mockSendGraceWarning = vi.fn();
const mockSendEmail = vi.fn();

vi.mock('../../lib/db', () => ({
  db: { execute: (...args: unknown[]) => mockExecute(...args) },
}));

vi.mock('../../lib/payments/wompi', () => ({
  createTransaction: (...args: unknown[]) => mockCreateTransaction(...args),
}));

vi.mock('../../lib/payments/lifecycle', () => ({
  startGracePeriod: (...args: unknown[]) => mockStartGrace(...args),
  cancelSubscription: (...args: unknown[]) => mockCancel(...args),
  expireSubscription: (...args: unknown[]) => mockExpire(...args),
  getExpiredGraceSubscriptions: (...args: unknown[]) => mockGetExpiredGrace(...args),
  getExpiredTrials: (...args: unknown[]) => mockGetExpiredTrials(...args),
}));

vi.mock('../../lib/payments/email', () => ({
  sendTrialReminder: (...args: unknown[]) => mockSendTrialReminder(...args),
  sendGraceWarning: (...args: unknown[]) => mockSendGraceWarning(...args),
}));

vi.mock('../../lib/email', () => ({
  sendEmail: (...args: unknown[]) => mockSendEmail(...args),
}));

// --- Handlers under test (real modules; cron-alert NOT mocked so the R5
// email-spy assertion exercises the real alert path) ---
import { GET as chargeWompiGet } from '../../pages/api/cron/charge-wompi';
import { GET as sendRemindersGet } from '../../pages/api/cron/send-reminders';
import { GET as expireSubscriptionsGet } from '../../pages/api/cron/expire-subscriptions';
import { getPlanById, PLANS } from '../../data/plans';

type ApiContext = Parameters<typeof chargeWompiGet>[0];

function makeContext(headers: Record<string, string> = {}): ApiContext {
  return {
    request: new Request('http://localhost/api/cron/test', { headers }),
  } as unknown as ApiContext;
}

const authHeaders = { authorization: 'Bearer test-cron-secret' };

beforeEach(() => {
  vi.clearAllMocks();
  vi.stubEnv('CRON_SECRET', 'test-cron-secret');
  mockExecute.mockResolvedValue({ rows: [] });
  mockCreateTransaction.mockResolvedValue({});
  mockGetExpiredGrace.mockResolvedValue([]);
  mockGetExpiredTrials.mockResolvedValue([]);
  mockSendTrialReminder.mockResolvedValue(undefined);
  mockSendGraceWarning.mockResolvedValue(undefined);
  mockSendEmail.mockResolvedValue(true);
});

afterEach(() => {
  vi.unstubAllEnvs();
});

describe('R1 — charge-wompi: real plan_id + filters + PLANS price', () => {
  it('queries s.plan_id with the due-subscription filters', async () => {
    mockExecute.mockResolvedValue({ rows: [] });

    const res = await chargeWompiGet(makeContext(authHeaders));

    expect(res.status).toBe(200);
    const sql = mockExecute.mock.calls[0][0].sql as string;
    expect(sql).toContain('s.plan_id');
    expect(sql).toContain("s.provider = 'wompi'");
    expect(sql).toContain("s.status = 'active'");
    expect(sql).toContain('s.current_period_end <= ?');
    expect(sql).toContain('s.payment_source_token IS NOT NULL');
  });

  it('charges the current PLANS price for pro (149900 × 100)', async () => {
    mockExecute.mockResolvedValue({
      rows: [
        {
          id: 'sub-1',
          user_id: 'user-1',
          plan_id: 'pro',
          payment_source_token: '12345',
          email: 'academy@test.com',
        },
      ],
    });

    const res = await chargeWompiGet(makeContext(authHeaders));

    expect(res.status).toBe(200);
    expect(mockCreateTransaction).toHaveBeenCalledWith(
      149900 * 100,
      'COP',
      expect.stringMatching(/^renewal-sub-1-\d+$/),
      12345,
      'academy@test.com'
    );
    const body = await res.json();
    expect(body).toEqual({ charged: 1, failed: 0, total: 1 });
  });

  it('rejects a reference plan price drift against PLANS', () => {
    expect(getPlanById('basico')?.priceCOP).toBe(89900);
    expect(getPlanById('pro')?.priceCOP).toBe(149900);
    expect(getPlanById('premium')?.priceCOP).toBe(259900);
    expect(PLANS.map((p) => p.id)).toEqual(['basico', 'pro', 'premium']);
  });

  it('returns undefined for an unknown plan id', () => {
    expect(getPlanById('legacy-plan')).toBeUndefined();
  });
});

describe('R2 — charge-wompi: unknown plan / throw → grace, no silent price', () => {
  it('unknown plan_id enters grace and is not charged', async () => {
    mockExecute.mockResolvedValue({
      rows: [
        {
          id: 'sub-unknown',
          user_id: 'user-1',
          plan_id: 'legacy-plan',
          payment_source_token: '12345',
          email: 'academy@test.com',
        },
      ],
    });

    const res = await chargeWompiGet(makeContext(authHeaders));

    expect(res.status).toBe(200);
    expect(mockStartGrace).toHaveBeenCalledWith('sub-unknown');
    expect(mockCreateTransaction).not.toHaveBeenCalled();
    const body = await res.json();
    expect(body).toEqual({ charged: 0, failed: 1, total: 1 });
  });

  it('createTransaction throw starts grace and continues', async () => {
    mockExecute.mockResolvedValue({
      rows: [
        {
          id: 'sub-fail',
          user_id: 'user-1',
          plan_id: 'pro',
          payment_source_token: '12345',
          email: 'academy@test.com',
        },
      ],
    });
    mockCreateTransaction.mockRejectedValue(new Error('Wompi API unavailable'));

    const res = await chargeWompiGet(makeContext(authHeaders));

    expect(res.status).toBe(200);
    expect(mockStartGrace).toHaveBeenCalledWith('sub-fail');
    const body = await res.json();
    expect(body).toEqual({ charged: 0, failed: 1, total: 1 });
  });
});

describe('R3 — send-reminders: real grace_expires_at + counts', () => {
  it('queries s.grace_expires_at and sends a grace warning with days_left', async () => {
    const graceEnd = new Date(Date.now() + 3 * 24 * 60 * 60 * 1000).toISOString();
    mockExecute.mockImplementation(({ sql }: { sql: string }) => {
      if (sql.includes('s.trial_end')) return { rows: [] };
      if (sql.includes('s.grace_expires_at')) {
        return {
          rows: [
            {
              id: 'sub-grace',
              user_id: 'user-1',
              email: 'academy@test.com',
              academy_name: 'Mi Academia',
              grace_expires_at: graceEnd,
            },
          ],
        };
      }
      return { rows: [] };
    });

    const res = await sendRemindersGet(makeContext(authHeaders));

    expect(res.status).toBe(200);
    const graceSql = mockExecute.mock.calls.find((call) =>
      (call[0].sql as string).includes('s.grace_expires_at')
    )?.[0].sql as string;
    expect(graceSql).toContain('s.grace_expires_at');
    expect(graceSql).toContain("s.status = 'grace'");
    expect(graceSql).toContain('s.grace_expires_at > ?');
    expect(mockSendGraceWarning).toHaveBeenCalledWith('academy@test.com', 3, 'Mi Academia');
    const body = await res.json();
    expect(body).toEqual({ trialReminders: 0, graceWarnings: 1 });
  });

  it('no grace subscriptions → graceWarnings 0 and HTTP 200', async () => {
    const res = await sendRemindersGet(makeContext(authHeaders));

    expect(res.status).toBe(200);
    expect(mockSendGraceWarning).not.toHaveBeenCalled();
    const body = await res.json();
    expect(body).toEqual({ trialReminders: 0, graceWarnings: 0 });
  });
});

describe('R4 — expire-subscriptions: grace → cancelled, trial → expired', () => {
  it('routes grace-expired subs to cancelSubscription and trials to expireSubscription', async () => {
    mockGetExpiredGrace.mockResolvedValue([{ id: 'grace-1' }, { id: 'grace-2' }]);
    mockGetExpiredTrials.mockResolvedValue([{ id: 'trial-1' }]);

    const res = await expireSubscriptionsGet(makeContext(authHeaders));

    expect(res.status).toBe(200);
    expect(mockCancel).toHaveBeenCalledWith('grace-1');
    expect(mockCancel).toHaveBeenCalledWith('grace-2');
    expect(mockExpire).toHaveBeenCalledWith('trial-1');
    expect(mockCancel).not.toHaveBeenCalledWith('trial-1');
    const body = await res.json();
    expect(body).toEqual({ expiredGrace: 2, expiredTrials: 1 });
  });
});

describe('R5 — failure alerting', () => {
  it('db reject → 500 + alert email to SUPPORT_EMAIL', async () => {
    mockExecute.mockRejectedValue(new Error('Turso down'));

    const res = await chargeWompiGet(makeContext(authHeaders));

    expect(res.status).toBe(500);
    expect(mockSendEmail).toHaveBeenCalledWith(
      expect.objectContaining({
        to: 'support@academix.app',
        subject: expect.stringContaining('charge-wompi'),
      })
    );
  });

  it('401 bad auth → no alert email', async () => {
    const res = await chargeWompiGet(makeContext({ authorization: 'Bearer wrong' }));

    expect(res.status).toBe(401);
    expect(mockSendEmail).not.toHaveBeenCalled();
  });

  it('missing CRON_SECRET → 500 without alert email', async () => {
    vi.stubEnv('CRON_SECRET', '');
    const res = await chargeWompiGet(makeContext({ authorization: 'Bearer test-cron-secret' }));

    expect(res.status).toBe(500);
    expect(mockSendEmail).not.toHaveBeenCalled();
  });
});