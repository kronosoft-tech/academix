export const prerender = false;

import type { APIRoute } from 'astro';
import { getFullTokenPayload } from '../../../lib/auth';
import { getAcceptanceToken, getPublicKey } from '../../../lib/payments/wompi';
import { PLANS } from '../../../data/plans';

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

  const publicKey = getPublicKey();
  if (!publicKey) {
    return new Response(JSON.stringify({ error: 'Wompi not configured' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  let acceptanceToken: string;
  try {
    acceptanceToken = await getAcceptanceToken();
  } catch {
    return new Response(JSON.stringify({ error: 'Failed to get acceptance token' }), {
      status: 502,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const reference = `${payload.sub}-${planId}-${crypto.randomUUID()}`;
  const amountInCents = plan.priceCOP * 100;
  const siteUrl = import.meta.env.SITE_URL || '';

  return new Response(
    JSON.stringify({
      publicKey,
      currency: 'COP',
      amountInCents,
      reference,
      redirectUrl: `${siteUrl}/dashboard?payment=success`,
      acceptanceToken,
      customerEmail: payload.email,
    }),
    {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    }
  );
};
