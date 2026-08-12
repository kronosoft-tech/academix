export const prerender = false;

import type { APIRoute } from 'astro';
import { getPayment } from '../../../lib/payments/mercadopago';
import { activateSubscription } from '../../../lib/payments/lifecycle';
import { db } from '../../../lib/db';

/**
 * Mercado Pago Webhook — handles payment notifications from Checkout Pro.
 * MP sends POST with { action: "payment.created", data: { id: "PAYMENT_ID" } }
 * or GET with ?topic=payment&id=PAYMENT_ID (IPN legacy)
 */
export const POST: APIRoute = async ({ request }) => {
  let paymentId: string | null = null;

  try {
    const body = await request.json();
    // Webhook v2 format
    if (body.action === 'payment.created' || body.action === 'payment.updated') {
      paymentId = body.data?.id?.toString() || null;
    }
    // IPN legacy format
    if (body.topic === 'payment') {
      paymentId = body.id?.toString() || null;
    }
  } catch {
    // Not JSON — ignore
  }

  if (paymentId) {
    await processPayment(paymentId);
  }

  return new Response(JSON.stringify({ received: true }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  });
};

export const GET: APIRoute = async ({ url }) => {
  const topic = url.searchParams.get('topic');
  const id = url.searchParams.get('id');

  if (topic === 'payment' && id) {
    await processPayment(id);
  }

  return new Response(JSON.stringify({ received: true }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  });
};

async function processPayment(paymentId: string): Promise<void> {
  let payment;
  try {
    payment = await getPayment(paymentId);
  } catch {
    return; // Can't verify — ignore (MP will retry)
  }

  if (payment.status !== 'approved') return;

  const externalReference = payment.external_reference;
  if (!externalReference) return;

  // Find subscription by provider_subscription_id (which stores external_reference)
  const subResult = await db.execute({
    sql: `SELECT id, user_id FROM subscriptions WHERE provider_subscription_id = ? LIMIT 1`,
    args: [externalReference],
  });

  if (subResult.rows.length === 0) return;

  const subscriptionId = subResult.rows[0].id as string;
  const userId = subResult.rows[0].user_id as string;

  // Activate subscription
  await activateSubscription(subscriptionId);

  // Extract planId from reference: {userId}-{planId}-{uuid}
  const parts = externalReference.split('-');
  const planId = parts.length >= 6 ? parts[5] : 'basico';

  await db.execute({
    sql: `UPDATE subscriptions SET plan_id = ?, updated_at = ? WHERE id = ?`,
    args: [planId, new Date().toISOString(), subscriptionId],
  });

  // Record payment (idempotent)
  const existing = await db.execute({
    sql: 'SELECT id FROM subscription_payments WHERE provider_payment_id = ?',
    args: [paymentId],
  });
  if (existing.rows.length > 0) return;

  await db.execute({
    sql: `INSERT INTO subscription_payments (id, user_id, subscription_id, amount, currency, status, provider, provider_payment_id, created_at)
          VALUES (?, ?, ?, ?, ?, ?, 'mercadopago', ?, ?)`,
    args: [
      crypto.randomUUID(),
      userId,
      subscriptionId,
      payment.transaction_amount,
      payment.currency_id || 'COP',
      'succeeded',
      paymentId,
      new Date().toISOString(),
    ],
  });
}
