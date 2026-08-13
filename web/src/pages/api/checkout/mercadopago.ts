export const prerender = false;

import type { APIRoute } from 'astro';
import { getFullTokenPayload } from '../../../lib/auth';
import { createPreference } from '../../../lib/payments/mercadopago';
import { getOrCreateTrialSubscription } from '../../../lib/payments/lifecycle';
import { PLANS } from '../../../data/plans';
import { db } from '../../../lib/db';

export const POST: APIRoute = async ({ request, cookies }) => {
  const payload = await getFullTokenPayload(cookies);
  if (!payload) {
    return new Response(JSON.stringify({ error: 'Unauthorized' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  let body: { planId?: string };
  try {
    body = await request.json();
  } catch {
    return new Response(JSON.stringify({ error: 'Invalid JSON body' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const planId = body.planId;
  const plan = PLANS.find((p) => p.id === planId);
  if (!plan) {
    return new Response(JSON.stringify({ error: 'Invalid plan' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const accessToken = import.meta.env.MP_ACCESS_TOKEN;
  if (!accessToken) {
    return new Response(JSON.stringify({ error: 'Mercado Pago not configured' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const siteUrl = import.meta.env.SITE_URL || 'http://localhost:4321';
  const externalReference = `${payload.sub}-${planId}-${crypto.randomUUID()}`;

  let preference;
  try {
    preference = await createPreference(
      `Academix ${plan.name} - Suscripción mensual`,
      plan.priceCOP,
      'COP',
      payload.email,
      externalReference,
      {
        success: `${siteUrl}/dashboard?payment=success&provider=mercadopago`,
        failure: `${siteUrl}/pricing?payment=failed`,
        pending: `${siteUrl}/dashboard?payment=pending`,
      },
      `${siteUrl}/api/webhooks/mercadopago`
    );
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    console.error(
      `[MP CHECKOUT] createPreference failed (plan=${planId}, user=${payload.sub}):`,
      err
    );
    return new Response(JSON.stringify({ error: 'Failed to create payment preference', detail: msg }), {
      status: 502,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  // Ensure the user has a subscription row (lazily created for desktop-first
  // users), then store the MP reference on it for webhook matching.
  const sub = await getOrCreateTrialSubscription(payload.sub, planId);
  const now = new Date().toISOString();
  await db.execute({
    sql: `UPDATE subscriptions
          SET provider = 'mercadopago', provider_subscription_id = ?, updated_at = ?
          WHERE id = ?`,
    args: [externalReference, now, sub.id],
  });

  // Use sandbox_init_point for test mode, init_point for production
  const checkoutUrl = accessToken.startsWith('TEST-')
    ? preference.sandbox_init_point
    : preference.init_point;

  return new Response(JSON.stringify({ url: checkoutUrl }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  });
};
