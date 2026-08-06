export const prerender = false;

import type { APIRoute } from 'astro';
import { getFullTokenPayload } from '../../../lib/auth';
import { createPreapproval } from '../../../lib/payments/mercadopago';
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

  const siteUrl = import.meta.env.SITE_URL || '';
  const backUrl = `${siteUrl}/dashboard?payment=success`;

  let preapproval;
  try {
    preapproval = await createPreapproval(
      plan.name,
      plan.priceCOP,
      payload.email,
      backUrl
    );
  } catch {
    return new Response(JSON.stringify({ error: 'Failed to create preapproval' }), {
      status: 502,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  // Store preapproval_id on subscription for webhook matching
  const now = new Date().toISOString();
  await db.execute({
    sql: `UPDATE subscriptions
          SET provider = 'mercadopago', provider_subscription_id = ?, updated_at = ?
          WHERE user_id = ? AND status = 'trial'`,
    args: [preapproval.id, now, payload.sub],
  });

  return new Response(JSON.stringify({ url: preapproval.init_point }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  });
};
