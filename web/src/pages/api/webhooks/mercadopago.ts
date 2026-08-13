export const prerender = false;

import type { APIRoute } from 'astro';
import {
  activateApprovedPayment,
  getPayment,
  getWebhookSecret,
  verifyWebhookSignature,
} from '../../../lib/payments/mercadopago';

interface MercadoPagoWebhookBody {
  action?: string;
  topic?: string;
  type?: string;
  id?: string | number;
  data?: { id?: string | number };
}

/**
 * Mercado Pago Webhook — handles payment notifications from Checkout Pro.
 * MP sends POST with { action: "payment.created", data: { id: "PAYMENT_ID" } }
 * or GET with ?topic=payment&id=PAYMENT_ID (IPN legacy).
 *
 * Every POST is authenticated via the `x-signature` header (HMAC-SHA256 over
 * `id:<data.id>;request-id:<x-request-id>;ts:<ts>;` using MP_WEBHOOK_SECRET).
 * Unverified requests are rejected with 401 before any processing.
 */
export const POST: APIRoute = async ({ request }) => {
  let body: MercadoPagoWebhookBody = {};
  try {
    body = await request.json();
  } catch {
    return new Response(JSON.stringify({ error: 'Invalid JSON' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const secret = getWebhookSecret();
  if (!secret) {
    return new Response(
      JSON.stringify({ error: 'Mercado Pago webhook secret not configured' }),
      {
        status: 500,
        headers: { 'Content-Type': 'application/json' },
      }
    );
  }

  const dataId = body.data?.id?.toString() ?? body.id?.toString() ?? '';
  const xSignature = request.headers.get('x-signature') || '';
  const xRequestId = request.headers.get('x-request-id') || '';

  if (!verifyWebhookSignature(xSignature, xRequestId, dataId, secret)) {
    return new Response(JSON.stringify({ error: 'Invalid signature' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  let paymentId: string | null = null;
  // Webhook v2 format
  if (body.action === 'payment.created' || body.action === 'payment.updated') {
    paymentId = dataId || null;
  }
  // IPN legacy format (topic in JSON body)
  if (body.topic === 'payment') {
    paymentId = dataId || null;
  }

  if (paymentId) {
    try {
      await processPayment(paymentId);
    } catch (err) {
      console.error(
        `[MP WEBHOOK] processPayment failed for payment ${paymentId}:`,
        err
      );
      return new Response(JSON.stringify({ error: 'Failed to process payment' }), {
        status: 500,
        headers: { 'Content-Type': 'application/json' },
      });
    }
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
    try {
      await processPayment(id);
    } catch (err) {
      console.error(`[MP WEBHOOK] processPayment failed for payment ${id}:`, err);
      return new Response(JSON.stringify({ error: 'Failed to process payment' }), {
        status: 500,
        headers: { 'Content-Type': 'application/json' },
      });
    }
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
  } catch (err) {
    console.error(
      `[MP WEBHOOK] getPayment failed for payment ${paymentId}:`,
      err
    );
    throw err;
  }

  if (payment.status !== 'approved') return;

  const externalReference = payment.external_reference;
  if (!externalReference) return;

  await activateApprovedPayment({
    paymentId,
    externalReference,
    transactionAmount: payment.transaction_amount,
    currencyId: payment.currency_id,
  });
}
