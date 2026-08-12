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

  const integritySecret = import.meta.env.WOMPI_INTEGRITY_SECRET;
  if (!integritySecret) {
    return new Response(JSON.stringify({ error: 'Wompi integrity secret not configured' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  let acceptanceToken: string;
  try {
    acceptanceToken = await getAcceptanceToken();
  } catch (err) {
    console.error('[WOMPI CHECKOUT] getAcceptanceToken failed:', err);
    return new Response(JSON.stringify({ error: 'Failed to get acceptance token', detail: String(err) }), {
      status: 502,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const reference = `${payload.sub}-${planId}-${crypto.randomUUID()}`;
  const amountInCents = plan.priceCOP * 100;
  const currency = 'COP';
  const siteUrl = import.meta.env.SITE_URL || 'http://localhost:4321';
  const redirectUrl = `${siteUrl}/dashboard?payment=success`;

  // Calculate integrity signature: SHA256(reference + amountInCents + currency + integritySecret)
  const encoder = new TextEncoder();
  const data = encoder.encode(`${reference}${amountInCents}${currency}${integritySecret}`);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const integrity = hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');

  return new Response(
    JSON.stringify({
      publicKey,
      currency,
      amountInCents,
      reference,
      redirectUrl,
      acceptanceToken,
      customerEmail: payload.email,
      integrity,
    }),
    {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    }
  );
};
