export const prerender = false;

import type { APIRoute } from 'astro';
import { getFullTokenPayload } from '../../../lib/auth';
import { activateSubscription, getOrCreateTrialSubscription } from '../../../lib/payments/lifecycle';
import { db } from '../../../lib/db';

const WOMPI_API_URL = import.meta.env.WOMPI_API_URL || 'https://production.wompi.co/v1';
const WOMPI_PUBLIC_KEY = import.meta.env.WOMPI_PUBLIC_KEY || '';

/**
 * Verify a Wompi transaction after redirect and activate the subscription.
 * Called from the frontend when user returns from Wompi checkout.
 * POST /api/payments/verify-wompi { transactionId }
 */
export const POST: APIRoute = async ({ request, cookies }) => {
  const payload = await getFullTokenPayload(cookies);
  if (!payload) {
    return new Response(JSON.stringify({ error: 'Unauthorized' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  let body: { transactionId?: string };
  try {
    body = await request.json();
  } catch {
    return new Response(JSON.stringify({ error: 'Invalid JSON' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const { transactionId } = body;
  if (!transactionId) {
    return new Response(JSON.stringify({ error: 'Missing transactionId' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  // Fetch transaction from Wompi API (requires the public key as Bearer auth)
  const res = await fetch(`${WOMPI_API_URL}/transactions/${transactionId}`, {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${WOMPI_PUBLIC_KEY}`,
    },
  });

  if (!res.ok) {
    console.error(
      `[VERIFY WOMPI] transaction fetch failed for ${transactionId}: ${res.status}`,
    );
    return new Response(JSON.stringify({ error: 'Failed to verify transaction' }), {
      status: 502,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const data = await res.json();
  const transaction = data.data;

  if (transaction.status !== 'APPROVED') {
    return new Response(JSON.stringify({
      error: 'Transaction not approved',
      status: transaction.status,
    }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  // Extract plan from reference: {userId}-{planId}-{uuid}
  const reference = transaction.reference || '';
  const parts = reference.split('-');
  // userId is 5 UUID parts, planId is 1 part (basico/pro/premium)
  const planId = parts.length >= 6 ? parts[5] : 'basico';

  // Ownership check: the transaction reference must embed the caller's userId
  if (parts.slice(0, 5).join('-') !== payload.sub) {
    return new Response(JSON.stringify({ error: 'Transaction does not belong to this user' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  // Get (or lazily create for desktop-first users) the trial row
  const sub = await getOrCreateTrialSubscription(payload.sub, planId);
  const subscriptionId = sub.id;
  const now = new Date().toISOString();

  // Update subscription: set provider, plan, activate
  await db.execute({
    sql: `UPDATE subscriptions
          SET provider = 'wompi',
              provider_subscription_id = ?,
              payment_source_token = ?,
              plan_id = ?,
              updated_at = ?
          WHERE id = ?`,
    args: [
      reference,
      transaction.payment_source_id?.toString() || null,
      planId,
      now,
      subscriptionId,
    ],
  });

  await activateSubscription(subscriptionId);

  // Record payment (idempotent)
  const existingPayment = await db.execute({
    sql: 'SELECT id FROM subscription_payments WHERE provider_payment_id = ?',
    args: [transaction.id],
  });

  if (existingPayment.rows.length === 0) {
    await db.execute({
      sql: `INSERT INTO subscription_payments (id, user_id, subscription_id, amount, currency, status, provider, provider_payment_id, created_at)
            VALUES (?, ?, ?, ?, ?, ?, 'wompi', ?, ?)`,
      args: [
        crypto.randomUUID(),
        payload.sub,
        subscriptionId,
        transaction.amount_in_cents / 100,
        transaction.currency || 'COP',
        'succeeded',
        transaction.id,
        now,
      ],
    });
  }

  return new Response(JSON.stringify({
    success: true,
    plan: planId,
    status: 'active',
  }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  });
};
