export const prerender = false;

import type { APIRoute } from 'astro';
import { verifyWebhookSignature, normalizeEvent } from '../../../lib/payments/stripe';
import {
  activateSubscription,
  startGracePeriod,
  cancelSubscription,
  findSubscriptionByStripeId,
} from '../../../lib/payments/lifecycle';
import { db } from '../../../lib/db';

export const POST: APIRoute = async ({ request }) => {
  const body = await request.text();
  const signature = request.headers.get('stripe-signature');

  if (!signature) {
    return new Response(JSON.stringify({ error: 'Missing stripe-signature header' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  let event;
  try {
    event = verifyWebhookSignature(body, signature);
  } catch {
    return new Response(JSON.stringify({ error: 'Invalid signature' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const normalized = normalizeEvent(event);

  switch (event.type) {
    case 'checkout.session.completed': {
      const userId = normalized.data.userId;
      const stripeSubId = normalized.data.subscriptionId;
      if (userId && stripeSubId) {
        // Link the Stripe subscription to the user's existing trial subscription
        await db.execute({
          sql: `UPDATE subscriptions
                SET stripe_subscription_id = ?, provider = 'stripe',
                    provider_subscription_id = ?, provider_customer_id = ?,
                    status = 'active', updated_at = ?
                WHERE user_id = ? AND status = 'trial'`,
          args: [
            stripeSubId,
            stripeSubId,
            normalized.data.customerId || null,
            new Date().toISOString(),
            userId,
          ],
        });
      }
      break;
    }

    case 'invoice.payment_succeeded': {
      const sub = await findSubscriptionByStripeId(normalized.data.subscriptionId || '');
      if (sub) {
        await activateSubscription(sub.id);
        const providerPaymentId = normalized.data.invoiceId || null;
        // Idempotency: skip if this payment was already recorded
        if (providerPaymentId && (await paymentExists(providerPaymentId))) break;
        await recordPayment(
          sub.user_id,
          sub.id,
          (normalized.data.amountPaid || 0) / 100,
          normalized.data.currency || 'usd',
          'succeeded',
          'stripe',
          providerPaymentId
        );
      }
      break;
    }

    case 'invoice.payment_failed': {
      const sub = await findSubscriptionByStripeId(normalized.data.subscriptionId || '');
      if (sub) {
        await startGracePeriod(sub.id);
        const providerPaymentId = normalized.data.invoiceId || null;
        if (providerPaymentId && (await paymentExists(providerPaymentId))) break;
        await recordPayment(
          sub.user_id,
          sub.id,
          0,
          'usd',
          'failed',
          'stripe',
          providerPaymentId
        );
      }
      break;
    }

    case 'customer.subscription.deleted': {
      const sub = await findSubscriptionByStripeId(normalized.data.subscriptionId || '');
      if (sub) {
        await cancelSubscription(sub.id);
      }
      break;
    }
  }

  return new Response(JSON.stringify({ received: true }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  });
};

async function paymentExists(providerPaymentId: string): Promise<boolean> {
  const result = await db.execute({
    sql: 'SELECT id FROM subscription_payments WHERE provider_payment_id = ?',
    args: [providerPaymentId],
  });
  return result.rows.length > 0;
}

async function recordPayment(
  userId: string,
  subscriptionId: string | null,
  amount: number,
  currency: string,
  status: string,
  provider: string,
  providerPaymentId: string | null
): Promise<void> {
  const id = crypto.randomUUID();
  const now = new Date().toISOString();

  await db.execute({
    sql: `INSERT INTO subscription_payments (id, user_id, subscription_id, amount, currency, status, provider, provider_payment_id, created_at)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`,
    args: [id, userId, subscriptionId, amount, currency, status, provider, providerPaymentId, now],
  });
}
