/**
 * Mercado Pago — Checkout Pro (Payment Preference)
 *
 * Colombia NO soporta preapproval/subscriptions API.
 * Se usa Checkout Pro que crea una preferencia de pago y redirige al usuario
 * al checkout de MP donde puede pagar con cualquier método.
 */

import { createHmac, timingSafeEqual } from 'node:crypto';
import { db } from '../db';
import { activateSubscription, getOrCreateTrialSubscription } from './lifecycle';

const MP_API_URL =
  import.meta.env.MP_API_URL || 'https://api.mercadopago.com';
const MP_ACCESS_TOKEN = import.meta.env.MP_ACCESS_TOKEN || '';
const MP_WEBHOOK_SECRET = import.meta.env.MP_WEBHOOK_SECRET || '';

export interface MpPreferenceResponse {
  id: string;
  init_point: string;
  sandbox_init_point: string;
}

export interface MpBackUrls {
  success: string;
  failure: string;
  pending: string;
}

export interface MpPreferenceItem {
  title: string;
  quantity: number;
  unit_price: number;
  currency_id: string;
}

export interface MpPreferenceBody {
  items: MpPreferenceItem[];
  payer: { email: string };
  external_reference: string;
  back_urls: MpBackUrls;
  auto_return?: string;
  notification_url?: string;
}

function isHttpsUrl(url: string): boolean {
  return url.startsWith('https://');
}

/**
 * Decide si se envía auto_return para una preferencia de Checkout Pro.
 *
 * Mercado Pago rechaza `auto_return: "approved"` (invalid_auto_return) cuando
 * algún back_url no es https (p.ej. desarrollo local con http://localhost).
 * Solo se envía cuando TODOS los back_urls son https; en otro caso se omite y
 * MP muestra el botón "volver al sitio" para regresar manualmente.
 */
function computeAutoReturn(backUrls: MpBackUrls): string | undefined {
  return isHttpsUrl(backUrls.success) &&
    isHttpsUrl(backUrls.failure) &&
    isHttpsUrl(backUrls.pending)
    ? 'approved'
    : undefined;
}

/**
 * Construye el body de una preferencia de Checkout Pro (pago único).
 * Helper puro exportado para poder probarlo sin llamar a la API de MP.
 */
export function buildPreferenceBody(
  title: string,
  amount: number,
  currency: string,
  payerEmail: string,
  externalReference: string,
  backUrls: MpBackUrls,
  notificationUrl?: string
): MpPreferenceBody {
  const body: MpPreferenceBody = {
    items: [
      {
        title,
        quantity: 1,
        unit_price: amount,
        currency_id: currency,
      },
    ],
    payer: {
      email: payerEmail,
    },
    external_reference: externalReference,
    back_urls: backUrls,
    notification_url: notificationUrl || undefined,
  };

  const autoReturn = computeAutoReturn(backUrls);
  if (autoReturn) body.auto_return = autoReturn;

  return body;
}

/**
 * Create a Checkout Pro preference (single payment).
 * Returns init_point URL for redirect to MP checkout.
 */
export async function createPreference(
  title: string,
  amount: number,
  currency: string,
  payerEmail: string,
  externalReference: string,
  backUrls: MpBackUrls,
  notificationUrl?: string
): Promise<MpPreferenceResponse> {
  const body = buildPreferenceBody(
    title,
    amount,
    currency,
    payerEmail,
    externalReference,
    backUrls,
    notificationUrl
  );

  const response = await fetch(`${MP_API_URL}/checkout/preferences`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${MP_ACCESS_TOKEN}`,
    },
    body: JSON.stringify(body),
  });

  if (!response.ok) {
    const errorText = await response.text();
    let errorBody: unknown = null;
    try {
      errorBody = JSON.parse(errorText);
    } catch {
      errorBody = null;
    }

    const detail =
      errorBody &&
      typeof errorBody === 'object' &&
      'detail' in errorBody &&
      typeof errorBody.detail === 'string'
        ? errorBody.detail
        : null;
    const message = errorBody
      ? detail || JSON.stringify(errorBody)
      : errorText || response.statusText;

    const err = new Error(
      `MercadoPago createPreference failed: ${response.status} - ${message}`
    ) as Error & { status?: number; detail?: string };
    err.status = response.status;
    if (detail) err.detail = detail;
    throw err;
  }

  return response.json();
}

/**
 * Get payment details from MP API.
 * Used to verify payment status after redirect.
 */
export async function getPayment(paymentId: string): Promise<{
  id: number;
  status: string;
  status_detail: string;
  external_reference: string;
  transaction_amount: number;
  currency_id: string;
  payer: { email: string };
}> {
  const response = await fetch(`${MP_API_URL}/v1/payments/${paymentId}`, {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${MP_ACCESS_TOKEN}`,
    },
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`MercadoPago getPayment failed: ${response.status} - ${error}`);
  }

  return response.json();
}

export function getWebhookSecret(): string {
  return MP_WEBHOOK_SECRET;
}

/**
 * Verify a Mercado Pago webhook `x-signature` header.
 *
 * MP signs every notification with the webhook secret (configured in the MP
 * dashboard — different from the Access Token). The `x-signature` header is:
 *   ts=<unix-timestamp>,v1=<hmac-sha256-hex>
 * The HMAC is computed over the canonical manifest:
 *   id:<data.id>;request-id:<x-request-id>;ts:<ts>;
 * (trailing semicolons included; data.id lowercased, x-request-id from the
 * `x-request-id` header). Comparison uses a timing-safe check.
 */
export function verifyWebhookSignature(
  xSignature: string,
  xRequestId: string,
  dataId: string,
  secret: string
): boolean {
  try {
    const parts = Object.fromEntries(
      xSignature.split(',').map((pair) => {
        const idx = pair.indexOf('=');
        if (idx === -1) return [pair, ''];
        return [pair.slice(0, idx).trim(), pair.slice(idx + 1).trim()];
      })
    );
    const ts = parts['ts'];
    const v1 = parts['v1'];
    if (!ts || !v1) return false;

    const manifest = `id:${dataId.toLowerCase()};request-id:${xRequestId};ts:${ts};`;
    const expected = createHmac('sha256', secret).update(manifest).digest('hex');

    const expectedBuf = Buffer.from(expected, 'hex');
    const receivedBuf = Buffer.from(v1, 'hex');
    if (expectedBuf.length !== receivedBuf.length) return false;
    return timingSafeEqual(expectedBuf, receivedBuf);
  } catch (err) {
    console.error('[MP WEBHOOK] verifyWebhookSignature error:', err);
    return false;
  }
}

export interface ApprovedPaymentInput {
  paymentId: string;
  externalReference: string;
  transactionAmount: number;
  currencyId: string;
  expectedUserId?: string;
}

/**
 * Activate the subscription for an approved Mercado Pago payment.
 *
 * Matches the subscription by `provider_subscription_id` (which stores the
 * checkout `external_reference`). For users without any subscription row
 * (e.g. desktop-registered users), lazily creates a default trial row before
 * activating, so the payment is never a silent no-op. Idempotent on the
 * recorded payment.
 */
export async function activateApprovedPayment({
  paymentId,
  externalReference,
  transactionAmount,
  currencyId,
  expectedUserId,
}: ApprovedPaymentInput): Promise<void> {
  // Ownership guard: when the caller provides the expected user id, refuse to
  // activate unless the reference is prefixed with it (reference format
  // {userId}-{planId}-{uuid}, userId = 5 UUID parts). The webhook never passes
  // the guard — signature verification is its trust boundary. The dashboard
  // verify endpoint passes the JWT `sub` as defense-in-depth.
  if (expectedUserId && !externalReference.startsWith(expectedUserId)) return;

  // Replay guard: if this paymentId was already processed, do nothing (idempotent)
  const alreadyProcessed = await db.execute({
    sql: 'SELECT id FROM subscription_payments WHERE provider_payment_id = ?',
    args: [paymentId],
  });
  if (alreadyProcessed.rows.length > 0) return;

  // Reference format: {userId}-{planId}-{uuid} (userId = 5 UUID parts, planId = 1 part)
  const parts = externalReference.split('-');
  if (parts.length < 6) return;
  const userId = parts.slice(0, 5).join('-');
  const planId = parts[5];

  const subResult = await db.execute({
    sql: `SELECT id FROM subscriptions WHERE provider_subscription_id = ? LIMIT 1`,
    args: [externalReference],
  });

  let subscriptionId: string;
  if (subResult.rows.length > 0) {
    subscriptionId = subResult.rows[0].id as string;
  } else {
    const sub = await getOrCreateTrialSubscription(userId, planId);
    subscriptionId = sub.id;
    await db.execute({
      sql: `UPDATE subscriptions
            SET provider = 'mercadopago', provider_subscription_id = ?,
                plan_id = ?, updated_at = ?
            WHERE id = ?`,
      args: [externalReference, planId, new Date().toISOString(), subscriptionId],
    });
  }

  await activateSubscription(subscriptionId);
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
      transactionAmount,
      currencyId || 'COP',
      'succeeded',
      paymentId,
      new Date().toISOString(),
    ],
  });
}
