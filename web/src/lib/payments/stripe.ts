import Stripe from 'stripe';

let stripeClient: Stripe | null = null;

function getStripe(): Stripe {
  if (!stripeClient) {
    const key = import.meta.env.STRIPE_SECRET_KEY;
    if (!key) {
      throw new Error('STRIPE_SECRET_KEY environment variable is not set');
    }
    stripeClient = new Stripe(key, { apiVersion: '2024-12-18.acacia' });
  }
  return stripeClient;
}

export { getStripe };

export function verifyWebhookSignature(
  body: string,
  signature: string
): Stripe.Event {
  const secret = import.meta.env.STRIPE_WEBHOOK_SECRET;
  if (!secret) {
    throw new Error('STRIPE_WEBHOOK_SECRET environment variable is not set');
  }
  return getStripe().webhooks.constructEvent(body, signature, secret);
}

export interface NormalizedEvent {
  type: string;
  data: {
    subscriptionId?: string;
    customerId?: string;
    userId?: string;
    invoiceId?: string;
    amountPaid?: number;
    currency?: string;
  };
}

export function normalizeEvent(event: Stripe.Event): NormalizedEvent {
  const normalized: NormalizedEvent = {
    type: event.type,
    data: {},
  };

  switch (event.type) {
    case 'checkout.session.completed': {
      const session = event.data.object as Stripe.Checkout.Session;
      normalized.data = {
        subscriptionId: session.subscription as string,
        customerId: session.customer as string,
        userId: session.client_reference_id || undefined,
      };
      break;
    }
    case 'invoice.payment_succeeded': {
      const invoice = event.data.object as Stripe.Invoice;
      normalized.data = {
        subscriptionId: invoice.subscription as string,
        customerId: invoice.customer as string,
        invoiceId: invoice.id,
        amountPaid: invoice.amount_paid,
        currency: invoice.currency,
      };
      break;
    }
    case 'invoice.payment_failed': {
      const failedInvoice = event.data.object as Stripe.Invoice;
      normalized.data = {
        subscriptionId: failedInvoice.subscription as string,
        customerId: failedInvoice.customer as string,
        invoiceId: failedInvoice.id,
      };
      break;
    }
    case 'customer.subscription.deleted': {
      const sub = event.data.object as Stripe.Subscription;
      normalized.data = {
        subscriptionId: sub.id,
        customerId: sub.customer as string,
      };
      break;
    }
  }

  return normalized;
}
