export const prerender = false;

import type { APIRoute } from 'astro';
import { getFullTokenPayload } from '../../../lib/auth';
import {
  activateApprovedPayment,
  getPayment,
} from '../../../lib/payments/mercadopago';

/**
 * Verify a Mercado Pago Checkout Pro payment after redirect and activate the
 * subscription.
 *
 * Mercado Pago appends `?payment_id=...&status=approved&external_reference=...`
 * to the success back_url. The dashboard calls this endpoint after the user
 * returns from the MP checkout; it is the dashboard-side persistence fallback
 * when the MP webhook fails (e.g. missing `MP_WEBHOOK_SECRET` in production).
 *
 * GET /api/payments/verify-mercadopago?payment_id=P&external_reference=U-plan-x
 *  200 { success: true, plan, status: 'active' }
 *  400 { success: false, message, status? } | 401 | 403 | 502 { success: false, message }
 */
export const GET: APIRoute = async ({ url, cookies }) => {
  const payload = await getFullTokenPayload(cookies);
  if (!payload) {
    return new Response(
      JSON.stringify({ success: false, message: 'Unauthorized' }),
      { status: 401, headers: { 'Content-Type': 'application/json' } }
    );
  }

  const paymentId =
    url.searchParams.get('payment_id') || url.searchParams.get('collection_id');
  const externalReference = url.searchParams.get('external_reference');
  if (!paymentId || !externalReference) {
    return new Response(
      JSON.stringify({
        success: false,
        message: 'Missing payment_id or external_reference',
      }),
      { status: 400, headers: { 'Content-Type': 'application/json' } }
    );
  }

  // Ownership check: the external reference must embed the caller's user id
  // (format {userId}-{planId}-{uuid} with userId = 5 UUID parts).
  if (!externalReference.startsWith(payload.sub)) {
    return new Response(
      JSON.stringify({
        success: false,
        message: 'Payment does not belong to this user',
      }),
      { status: 403, headers: { 'Content-Type': 'application/json' } }
    );
  }

  let payment: Awaited<ReturnType<typeof getPayment>>;
  try {
    payment = await getPayment(paymentId);
  } catch (err) {
    console.error(`[VERIFY MP] getPayment failed for ${paymentId}:`, err);
    return new Response(
      JSON.stringify({ success: false, message: 'Failed to verify payment' }),
      { status: 502, headers: { 'Content-Type': 'application/json' } }
    );
  }

  if (payment.status !== 'approved') {
    return new Response(
      JSON.stringify({
        success: false,
        message: 'Payment not approved',
        status: payment.status,
      }),
      { status: 400, headers: { 'Content-Type': 'application/json' } }
    );
  }

  await activateApprovedPayment({
    paymentId,
    externalReference,
    transactionAmount: payment.transaction_amount,
    currencyId: payment.currency_id,
    expectedUserId: payload.sub,
  });

  // Plan is the 6th part of {userId}-{planId}-{uuid} (userId = 5 UUID parts).
  const plan = externalReference.split('-')[5] || 'basico';

  return new Response(
    JSON.stringify({ success: true, plan, status: 'active' }),
    { status: 200, headers: { 'Content-Type': 'application/json' } }
  );
};
