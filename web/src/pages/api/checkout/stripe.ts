export const prerender = false;

import type { APIRoute } from 'astro';
import Stripe from 'stripe';
import { getFullTokenPayload } from '../../../lib/auth';
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

  const stripeKey = import.meta.env.STRIPE_SECRET_KEY;
  if (!stripeKey) {
    return new Response(JSON.stringify({ error: 'Stripe not configured' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const stripe = new Stripe(stripeKey);

  const priceId = getPriceId(planId);
  if (!priceId) {
    return new Response(JSON.stringify({ error: 'Price not configured for this plan' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const session = await stripe.checkout.sessions.create({
    mode: 'subscription',
    customer_email: payload.email,
    client_reference_id: payload.sub,
    line_items: [{ price: priceId, quantity: 1 }],
    success_url: `${import.meta.env.SITE_URL}/dashboard?payment=success`,
    cancel_url: `${import.meta.env.SITE_URL}/pricing?payment=cancelled`,
    subscription_data: { trial_period_days: 15 },
    metadata: { userId: payload.sub, planId },
  });

  return new Response(JSON.stringify({ url: session.url }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  });
};

function getPriceId(planId: string): string {
  const prices: Record<string, string> = {
    basico: import.meta.env.STRIPE_PRICE_BASIC || '',
    pro: import.meta.env.STRIPE_PRICE_PRO || '',
    premium: import.meta.env.STRIPE_PRICE_PREMIUM || '',
  };
  return prices[planId] || '';
}
