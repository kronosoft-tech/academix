/**
 * Mercado Pago — Checkout Pro (Payment Preference)
 *
 * Colombia NO soporta preapproval/subscriptions API.
 * Se usa Checkout Pro que crea una preferencia de pago y redirige al usuario
 * al checkout de MP donde puede pagar con cualquier método.
 */

const MP_API_URL =
  import.meta.env.MP_API_URL || 'https://api.mercadopago.com';
const MP_ACCESS_TOKEN = import.meta.env.MP_ACCESS_TOKEN || '';

export interface MpPreferenceResponse {
  id: string;
  init_point: string;
  sandbox_init_point: string;
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
  backUrls: { success: string; failure: string; pending: string },
  notificationUrl?: string
): Promise<MpPreferenceResponse> {
  const body = {
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
    auto_return: 'approved',
    notification_url: notificationUrl || undefined,
  };

  const response = await fetch(`${MP_API_URL}/checkout/preferences`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${MP_ACCESS_TOKEN}`,
    },
    body: JSON.stringify(body),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(
      `MercadoPago createPreference failed: ${response.status} - ${error}`
    );
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
