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
 * Classify a MercadoPago API failure (enriched by getPayment with `.status`,
 * `.detail`, `.retryAfter`) into the HTTP response MP's webhook contract expects.
 *
 * MP's contract: 200 acknowledges receipt and STOPS retries. Returning 500 makes
 * MP treat the endpoint as broken and abandon retries — so a 404 (payment not
 * found / already final) or a transient MP/network error MUST NOT be a 500.
 *
 *  - 404  → 200  (payment not found; may already be final — ack, don't retry-loop)
 *  - 401/403 → 200 + error-level log (token/config issue; MP can't fix by retry)
 *  - 429  → 503  + Retry-After (rate limited; MP retries per Retry-After)
 *  - 5xx  → 503  (MP API/server error; MP retries with backoff)
 *  - no `.status` (DB/TURSO down, unexpected) → 500 (critical, ops action)
 */
function classifyMpError(err: unknown, paymentId: string): Response {
  const status = (err as { status?: number })?.status;

  switch (status) {
    case 404:
      // Payment not found — likely already processed or a sandbox/test id.
      // Acknowledge so MP stops retrying.
      console.warn(
        `[MP WEBHOOK] getPayment 404 for payment ${paymentId}:`,
        (err as { detail?: string })?.detail || err
      );
      return new Response(
        JSON.stringify({
          received: true,
          warning: 'Payment not found (may already be final or a test id)',
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } }
      );

    case 401:
    case 403:
      // Token misconfiguration/expired — MP can't fix by retrying; make it loud.
      console.error('[MP_API_AUTH_ERROR]', `getPayment ${status} for ${paymentId}:`, err);
      return new Response(JSON.stringify({ received: true }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });

    case 429:
      // Rate limited — MP retries per Retry-After.
      const retryAfter = (err as { retryAfter?: string })?.retryAfter || '60';
      return new Response(JSON.stringify({ received: true }), {
        status: 503,
        headers: {
          'Content-Type': 'application/json',
          'Retry-After': retryAfter,
        },
      });

    default: {
      // MP 5xx/server error — retryable by MP.
      if (typeof status === 'number' && status >= 500) {
        // MP/server error — retryable.
        console.error(
          `[MP WEBHOOK] getPayment ${status} for payment ${paymentId}:`,
          err
        );
        return new Response(JSON.stringify({ received: true }), {
          status: 503,
          headers: { 'Content-Type': 'application/json' },
        });
      }
      // No `.status` (DB/TURSO down) or an unhandled status — critical, surface it.
      console.error(
        `[MP WEBHOOK] processPayment failed for payment ${paymentId}:`,
        err
      );
      return new Response(
        JSON.stringify({ error: 'Failed to process payment' }),
        { status: 500, headers: { 'Content-Type': 'application/json' } }
      );
    }
  }
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

  let signatureValid: boolean;
  try {
    signatureValid = verifyWebhookSignature(xSignature, xRequestId, dataId, secret);
  } catch (err) {
    console.error('[MP WEBHOOK] signature verification threw:', err);
    return new Response(JSON.stringify({ error: 'Invalid signature' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' },
    });
  }
  if (!signatureValid) {
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
      return classifyMpError(err, paymentId);
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
      return classifyMpError(err, id);
    }
  }

  return new Response(JSON.stringify({ received: true }), {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  });
};

async function processPayment(paymentId: string): Promise<void> {
  // getPayment enriches its thrown Error with `.status`/`.detail`/`.retryAfter`
  // (mirroring createPreference). All errors bubble to the webhook handler, which
  // classifies them by HTTP status — see classifyMpError. Logging happens once,
  // at the handler level, to avoid the previous double-log path.
  const payment = await getPayment(paymentId);

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
