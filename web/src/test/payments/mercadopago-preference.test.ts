/**
 * Mercado Pago Checkout Pro — Preference body building tests.
 *
 * MP rejects `auto_return: "approved"` with invalid_auto_return when any
 * back_url is not https (e.g. local dev on http://localhost). These tests
 * verify buildPreferenceBody degrades gracefully without calling the MP API.
 */

import { describe, it, expect } from 'vitest';
import { buildPreferenceBody } from '../../lib/payments/mercadopago';

const httpsBackUrls = {
  success: 'https://academix.app/dashboard?payment=success&provider=mercadopago',
  failure: 'https://academix.app/pricing?payment=failed',
  pending: 'https://academix.app/dashboard?payment=pending',
};

const httpBackUrls = {
  success: 'http://localhost:4321/dashboard?payment=success&provider=mercadopago',
  failure: 'http://localhost:4321/pricing?payment=failed',
  pending: 'http://localhost:4321/dashboard?payment=pending',
};

function build(
  backUrls: { success: string; failure: string; pending: string },
  notificationUrl?: string
) {
  return buildPreferenceBody(
    'Academix Pro - Suscripción mensual',
    89900,
    'COP',
    'user@test.com',
    'user-123-pro-uuid',
    backUrls,
    notificationUrl
  );
}

describe('buildPreferenceBody', () => {
  it('sets auto_return to "approved" when all back_urls are https', () => {
    const body = build(httpsBackUrls);
    expect(body.auto_return).toBe('approved');
  });

  it('omits auto_return when all back_urls are http', () => {
    const body = build(httpBackUrls);
    expect('auto_return' in body).toBe(false);
    expect(body.auto_return).toBeUndefined();
  });

  it('omits auto_return when only the success back_url is http', () => {
    const body = build({ ...httpsBackUrls, success: httpBackUrls.success });
    expect('auto_return' in body).toBe(false);
  });

  it('omits auto_return when only the failure back_url is http', () => {
    const body = build({ ...httpsBackUrls, failure: httpBackUrls.failure });
    expect('auto_return' in body).toBe(false);
  });

  it('omits auto_return when only the pending back_url is http', () => {
    const body = build({ ...httpsBackUrls, pending: httpBackUrls.pending });
    expect('auto_return' in body).toBe(false);
  });

  it('keeps back_urls and the rest of the payload intact', () => {
    const body = build(httpsBackUrls, 'https://academix.app/api/webhooks/mercadopago');

    expect(body.back_urls).toEqual(httpsBackUrls);
    expect(body.items[0].title).toBe('Academix Pro - Suscripción mensual');
    expect(body.items[0].unit_price).toBe(89900);
    expect(body.items[0].currency_id).toBe('COP');
    expect(body.payer.email).toBe('user@test.com');
    expect(body.external_reference).toBe('user-123-pro-uuid');
    expect(body.notification_url).toBe('https://academix.app/api/webhooks/mercadopago');
  });

  it('serializes without auto_return when a back_url is http', () => {
    const body = build(httpBackUrls);
    const serialized = JSON.stringify(body);
    expect(serialized).not.toContain('auto_return');
    expect(serialized).toContain('back_urls');
  });
});
