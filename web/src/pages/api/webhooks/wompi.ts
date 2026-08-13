export const prerender = false;

import type { APIRoute } from 'astro';
import { verifyWebhookSignature, type WompiWebhookEvent } from '../../../lib/payments/wompi';
import {
  activateSubscription,
  startGracePeriod,
  getOrCreateTrialSubscription,
} from '../../../lib/payments/lifecycle';
import { db } from '../../../lib/db';

export const POST: APIRoute = async ({ request }) => {
  let event: WompiWebhookEvent;
  try {
    event = await request.json();
  } catch {
    return new Response(JSON.stringify({ error: 'Invalid JSON' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  // Verify signature from X-Event-Checksum header
  const checksum = request.headers.get('x-event-checksum') || '';
  if (!checksum) {
    return new Response(JSON.stringify({ error: 'Missing checksum header' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const isValid = await verifyWebhookSignature(event, checksum);
  if (!isValid) {
    return new Response(JSON.stringify({ error: 'Invalid signature' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const { transaction } = event.data;
  const reference = transaction.reference;
  const isRenewal = reference.startsWith('renewal-');

  if (transaction.status === 'APPROVED') {
    if (isRenewal) {
      // Renewal charge approved — extend subscription period
      const subscriptionId = extractSubscriptionIdFromReference(reference);
      if (subscriptionId) {
        await activateSubscription(subscriptionId);
        await recordPayment(
          subscriptionId,
          transaction.amount_in_cents / 100,
          transaction.currency,
          'succeeded',
          transaction.id
        );
      }
    } else {
      // First payment approved — activate subscription
      // Reference format: {userId}-{planId}-{uuid}
      const userId = extractUserIdFromReference(reference);
      if (userId) {
        const planId = extractPlanIdFromReference(reference);
        // Get (or lazily create for desktop-first users) the trial row
        const sub = await getOrCreateTrialSubscription(userId, planId);
        await db.execute({
          sql: `UPDATE subscriptions
                SET provider = 'wompi', provider_subscription_id = ?,
                    payment_source_token = ?, plan_id = ?, updated_at = ?
                WHERE id = ?`,
          args: [
            reference,
            transaction.payment_source_id?.toString() || null,
            planId,
            new Date().toISOString(),
            sub.id,
          ],
        });
        await activateSubscription(sub.id);
        await recordPayment(
          sub.id,
          transaction.amount_in_cents / 100,
          transaction.currency,
          'succeeded',
          transaction.id
        );
      }
    }
  } else if (
    (transaction.status === 'DECLINED' || transaction.status === 'VOIDED') &&
    isRenewal
  ) {
    // Renewal failed — start grace period
    const subscriptionId = extractSubscriptionIdFromReference(reference);
    if (subscriptionId) {
      await startGracePeriod(subscriptionId);
      await recordPayment(
        subscriptionId,
        0,
        transaction.currency,
        'failed',
        transaction.id
      );
    }
  }
  // For non-renewal DECLINED/VOIDED: do nothing (first payment failed, no sub to grace)

  return new Response(JSON.stringify({ received: true }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  });
};

/**
 * Extract subscription ID from renewal reference.
 * Format: renewal-{subscriptionId}-{timestamp}
 */
function extractSubscriptionIdFromReference(reference: string): string | null {
  const parts = reference.split('-');
  // renewal-{uuid parts}-{timestamp}
  // UUID has 5 parts joined by dashes, so: renewal + 5 UUID parts + timestamp = 7 parts
  if (parts.length < 7) return null;
  // Extract UUID: parts[1] through parts[5]
  return parts.slice(1, 6).join('-');
}

/**
 * Extract user ID from first-payment reference.
 * Format: {userId}-{planId}-{uuid}
 * userId is a UUID (5 dash-separated parts)
 */
function extractUserIdFromReference(reference: string): string | null {
  const parts = reference.split('-');
  // userId (5 parts) + planId (1 part) + uuid (5 parts) = 11 parts minimum
  if (parts.length < 7) return null;
  return parts.slice(0, 5).join('-');
}

/**
 * Extract plan ID from first-payment reference.
 * Format: {userId}-{planId}-{uuid}
 */
function extractPlanIdFromReference(reference: string): string {
  const parts = reference.split('-');
  return parts.length >= 6 ? parts[5] : 'basico';
}

async function recordPayment(
  subscriptionId: string,
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

  // Get user_id from subscription
  const sub = await db.execute({
    sql: 'SELECT user_id FROM subscriptions WHERE id = ?',
    args: [subscriptionId],
  });
  const userId = sub.rows.length > 0 ? (sub.rows[0].user_id as string) : null;

  const id = crypto.randomUUID();
  const now = new Date().toISOString();

  await db.execute({
    sql: `INSERT INTO subscription_payments (id, user_id, subscription_id, amount, currency, status, provider, provider_payment_id, created_at)
          VALUES (?, ?, ?, ?, ?, ?, 'wompi', ?, ?)`,
    args: [id, userId, subscriptionId, amount, currency, status, providerPaymentId, now],
  });
}
