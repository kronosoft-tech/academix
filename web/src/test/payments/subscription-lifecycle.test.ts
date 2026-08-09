/**
 * Subscription Lifecycle Tests
 *
 * Verifica el flujo completo:
 * 1. Registro → Trial (15 días)
 * 2. Trial expira → expired (login bloqueado)
 * 3. Usuario paga → active (30 días de periodo)
 * 4. Pago falla → grace (7 días)
 * 5. Grace expira → expired
 * 6. Usuario paga de nuevo → active (reactivación)
 * 7. Usuario cancela → cancelled
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the db module
const mockExecute = vi.fn();
vi.mock('../../lib/db', () => ({
  db: { execute: (...args: unknown[]) => mockExecute(...args) },
}));

import {
  createTrialSubscription,
  activateSubscription,
  startGracePeriod,
  expireSubscription,
  cancelSubscription,
  getExpiredTrials,
  getExpiredGraceSubscriptions,
  findSubscriptionByStripeId,
  findByProviderSubId,
} from '../../lib/payments/lifecycle';

describe('Subscription Lifecycle', () => {
  beforeEach(() => {
    mockExecute.mockReset();
    mockExecute.mockResolvedValue({ rows: [] });
  });

  describe('1. Registro → Trial (15 días)', () => {
    it('crea una suscripción trial con 15 días de duración', async () => {
      await createTrialSubscription('user-123', 'trial', null);

      expect(mockExecute).toHaveBeenCalledTimes(1);
      const call = mockExecute.mock.calls[0][0];

      // Verifica el SQL de inserción
      expect(call.sql).toContain('INSERT INTO subscriptions');
      expect(call.sql).toContain("'trial'");

      // Verifica los argumentos
      const args = call.args;
      expect(args[1]).toBe('user-123'); // user_id
      expect(args[2]).toBe('trial'); // plan
      // trial_end debe ser ~15 días en el futuro
      const trialEnd = new Date(args[4] as string);
      const now = new Date();
      const diffDays = Math.round((trialEnd.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));
      expect(diffDays).toBe(15);
    });

    it('crea trial con provider Stripe cuando se especifica', async () => {
      await createTrialSubscription('user-123', 'basico', 'sub_stripe_123', 'stripe');

      const args = mockExecute.mock.calls[0][0].args;
      expect(args[5]).toBe('sub_stripe_123'); // stripe_subscription_id
      expect(args[7]).toBe('stripe'); // provider
    });
  });

  describe('2. Trial expira → expired', () => {
    it('getExpiredTrials devuelve trials vencidos', async () => {
      const pastDate = new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString();
      mockExecute.mockResolvedValue({
        rows: [{
          id: 'sub-1', user_id: 'user-123', plan: 'trial', status: 'trial',
          trial_start: pastDate, trial_end: pastDate,
          grace_start: null, grace_end: null,
          stripe_subscription_id: null,
          current_period_start: null, current_period_end: null,
          provider: null, provider_subscription_id: null,
          provider_customer_id: null, trial_starts_at: pastDate,
          grace_expires_at: null,
        }],
      });

      const expired = await getExpiredTrials();
      expect(expired).toHaveLength(1);
      expect(expired[0].status).toBe('trial');
      expect(expired[0].user_id).toBe('user-123');

      // Verifica que el query filtra por status='trial' AND trial_end < now
      const sql = mockExecute.mock.calls[0][0].sql;
      expect(sql).toContain("status = 'trial'");
      expect(sql).toContain('trial_end < ?');
    });

    it('expireSubscription cambia status a expired', async () => {
      await expireSubscription('sub-1');

      const call = mockExecute.mock.calls[0][0];
      expect(call.sql).toContain("SET status = 'expired'");
      expect(call.args[0]).toBe('sub-1');
    });
  });

  describe('3. Usuario paga → active', () => {
    it('activateSubscription cambia status a active con periodo de 30 días', async () => {
      await activateSubscription('sub-1');

      const call = mockExecute.mock.calls[0][0];
      expect(call.sql).toContain("SET status = 'active'");
      expect(call.sql).toContain('trial_start = NULL');
      expect(call.sql).toContain('trial_end = NULL');
      expect(call.sql).toContain('current_period_start = ?');
      expect(call.sql).toContain('current_period_end = ?');

      // current_period_end debe ser ~30 días en el futuro
      const periodEnd = new Date(call.args[1] as string);
      const now = new Date();
      const diffDays = Math.round((periodEnd.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));
      expect(diffDays).toBe(30);
    });

    it('al activar se limpian los campos de trial y grace', async () => {
      await activateSubscription('sub-1');

      const sql = mockExecute.mock.calls[0][0].sql;
      expect(sql).toContain('grace_start = NULL');
      expect(sql).toContain('grace_end = NULL');
    });
  });

  describe('4. Pago falla → grace (7 días)', () => {
    it('startGracePeriod cambia status a grace con 7 días', async () => {
      await startGracePeriod('sub-1');

      const call = mockExecute.mock.calls[0][0];
      expect(call.sql).toContain("SET status = 'grace'");
      expect(call.sql).toContain('grace_start = ?');
      expect(call.sql).toContain('grace_end = ?');

      // grace_end debe ser ~7 días en el futuro
      const graceEnd = new Date(call.args[1] as string);
      const now = new Date();
      const diffDays = Math.round((graceEnd.getTime() - now.getTime()) / (1000 * 60 * 60 * 24));
      expect(diffDays).toBe(7);
    });
  });

  describe('5. Grace expira → expired', () => {
    it('getExpiredGraceSubscriptions devuelve suscripciones con grace vencido', async () => {
      const pastDate = new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString();
      mockExecute.mockResolvedValue({
        rows: [{
          id: 'sub-1', user_id: 'user-123', plan: 'pro', status: 'grace',
          trial_start: null, trial_end: null,
          grace_start: pastDate, grace_end: pastDate,
          stripe_subscription_id: 'sub_stripe_1',
          current_period_start: null, current_period_end: null,
          provider: 'stripe', provider_subscription_id: 'sub_stripe_1',
          provider_customer_id: 'cus_1', trial_starts_at: null,
          grace_expires_at: pastDate,
        }],
      });

      const expired = await getExpiredGraceSubscriptions();
      expect(expired).toHaveLength(1);
      expect(expired[0].status).toBe('grace');

      const sql = mockExecute.mock.calls[0][0].sql;
      expect(sql).toContain("status = 'grace'");
      expect(sql).toContain('grace_end < ?');
    });
  });

  describe('6. Reactivación (pago después de expired)', () => {
    it('activateSubscription funciona sobre una suscripción expired', async () => {
      // Simula reactivación: mismo flujo que activar
      await activateSubscription('sub-expired-1');

      const call = mockExecute.mock.calls[0][0];
      expect(call.sql).toContain("SET status = 'active'");
      expect(call.args[2]).toBe('sub-expired-1');
    });
  });

  describe('7. Cancelación', () => {
    it('cancelSubscription cambia status a cancelled', async () => {
      await cancelSubscription('sub-1');

      const call = mockExecute.mock.calls[0][0];
      expect(call.sql).toContain("SET status = 'cancelled'");
      expect(call.args[0]).toBe('sub-1');
    });
  });

  describe('Búsqueda por provider', () => {
    it('findSubscriptionByStripeId encuentra por stripe_subscription_id', async () => {
      mockExecute.mockResolvedValue({
        rows: [{
          id: 'sub-1', user_id: 'user-123', plan: 'pro', status: 'active',
          trial_start: null, trial_end: null,
          grace_start: null, grace_end: null,
          stripe_subscription_id: 'sub_stripe_abc',
          current_period_start: '2026-08-01', current_period_end: '2026-08-31',
          provider: 'stripe', provider_subscription_id: 'sub_stripe_abc',
          provider_customer_id: 'cus_abc', trial_starts_at: null,
          grace_expires_at: null,
        }],
      });

      const sub = await findSubscriptionByStripeId('sub_stripe_abc');
      expect(sub).not.toBeNull();
      expect(sub!.status).toBe('active');
      expect(sub!.provider).toBe('stripe');
    });

    it('findByProviderSubId encuentra por provider_subscription_id', async () => {
      mockExecute.mockResolvedValue({
        rows: [{
          id: 'sub-2', user_id: 'user-456', plan: 'basico', status: 'active',
          trial_start: null, trial_end: null,
          grace_start: null, grace_end: null,
          stripe_subscription_id: null,
          current_period_start: '2026-08-01', current_period_end: '2026-08-31',
          provider: 'wompi', provider_subscription_id: 'wompi-ref-123',
          provider_customer_id: null, trial_starts_at: null,
          grace_expires_at: null,
        }],
      });

      const sub = await findByProviderSubId('wompi-ref-123');
      expect(sub).not.toBeNull();
      expect(sub!.provider).toBe('wompi');
    });

    it('devuelve null si no encuentra', async () => {
      mockExecute.mockResolvedValue({ rows: [] });

      const sub = await findSubscriptionByStripeId('nonexistent');
      expect(sub).toBeNull();
    });
  });
});
