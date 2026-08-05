export const prerender = false;

import type { APIRoute } from 'astro';
import { verifyWebhookSignature, normalizeEvent } from '../../../lib/payments/stripe';
import {
  createTrialSubscription,
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
        await createTrialSubscription(userId, 'basic', stripeSubId);
        await recordPayment(userId, null, 0, 'usd', 'succeeded', 'stripe', stripeSubId);
      }
      break;
    }

    case 'invoice.payment_succeeded': {
      const sub = await findSubscriptionByStripeId(normalized.data.subscriptionId || '');
      if (sub) {
        await activateSubscription(sub.id);
        await recordPayment(
          sub.user_id,
          sub.id,
          (normalized.data.amountPaid || 0) / 100,
          normalized.data.currency || 'usd',
          'succeeded',
          'stripe',
          normalized.data.invoiceId || null
        );
      }
      break;
    }

    case 'invoice.payment_failed': {
      const sub = await findSubscriptionByStripeId(normalized.data.subscriptionId || '');
      if (sub) {
        await startGracePeriod(sub.id);
        await recordPayment(
          sub.user_id,
          sub.id,
          0,
          'usd',
          'failed',
          'stripe',
          normalized.data.invoiceId || null
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
