export const prerender = false;

import type { APIRoute } from 'astro';
import { verifyIPN } from '../../../lib/payments/mercadopago';
import {
  activateSubscription,
  startGracePeriod,
  cancelSubscription,
  findByProviderSubId,
} from '../../../lib/payments/lifecycle';
import { db } from '../../../lib/db';

/**
 * Mercado Pago IPN webhook handler.
 * MP sends GET or POST with ?topic=subscription_preapproval&id=XXX
 * We fetch the preapproval status from MP API to verify and act accordingly.
 * Always return 200 (MP retries on non-200).
 */
export const GET: APIRoute = async ({ url }) => {
  return handleIPN(url);
};

export const POST: APIRoute = async ({ url }) => {
  return handleIPN(url);
};

async function handleIPN(url: URL): Promise<Response> {
  const topic = url.searchParams.get('topic') || '';
  const id = url.searchParams.get('id') || '';

  if (!topic || !id) {
    return new Response(JSON.stringify({ received: true }), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  let preapproval;
  try {
    preapproval = await verifyIPN(topic, id);
  } catch {
    // Return 200 anyway — MP will retry on non-200
    return new Response(JSON.stringify({ received: true }), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  if (!preapproval) {
    // Topic not subscription_preapproval — ignore
    return new Response(JSON.stringify({ received: true }), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const subscription = await findByProviderSubId(preapproval.id);
  if (!subscription) {
    // No matching subscription in our DB — ignore
    return new Response(JSON.stringify({ received: true }), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const status = preapproval.status as string;

  if (status === 'authorized') {
    await activateSubscription(subscription.id);
    await recordPayment(
      subscription.id,
      subscription.user_id,
      preapproval.auto_recurring.transaction_amount,
      preapproval.auto_recurring.currency_id,
      'succeeded',
      preapproval.id
    );
  } else if (status === 'paused') {
    await startGracePeriod(subscription.id);
  } else if (status === 'cancelled') {
    await cancelSubscription(subscription.id);
  }
  // 'pending' — do nothing, wait for next notification

  return new Response(JSON.stringify({ received: true }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  });
}

async function recordPayment(
  subscriptionId: string,
  userId: string,
  amount: number,
  currency: string,
  status: string,
  providerPaymentId: string
): Promise<void> {
  // Idempotency: skip if already recorded
  const existing = await db.execute({
    sql: 'SELECT id FROM subscription_payments WHERE provider_payment_id = ?',
    args: [providerPaymentId],
  });
  if (existing.rows.length > 0) return;

  const id = crypto.randomUUID();
  const now = new Date().toISOString();

  await db.execute({
    sql: `INSERT INTO subscription_payments (id, user_id, subscription_id, amount, currency, status, provider, provider_payment_id, created_at)
          VALUES (?, ?, ?, ?, ?, ?, 'mercadopago', ?, ?)`,
    args: [id, userId, subscriptionId, amount, currency, status, providerPaymentId, now],
  });
}
