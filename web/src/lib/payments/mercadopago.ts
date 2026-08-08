const MP_API_URL =
  import.meta.env.MP_API_URL || 'https://api.mercadopago.com';
const MP_ACCESS_TOKEN = import.meta.env.MP_ACCESS_TOKEN || '';

export interface MpPreapprovalRequest {
  payer_email: string;
  back_url: string;
  reason: string;
  auto_recurring: {
    frequency: number;
    frequency_type: 'months';
    transaction_amount: number;
    currency_id: string;
  };
}

export interface MpPreapprovalResponse {
  id: string;
  status: string;
  init_point: string;
  payer_email: string;
  reason: string;
  auto_recurring: {
    frequency: number;
    frequency_type: string;
    transaction_amount: number;
    currency_id: string;
  };
}

export type MpPreapprovalStatus =
  | 'authorized'
  | 'paused'
  | 'cancelled'
  | 'pending';

/**
 * Create a preapproval (subscription) in Mercado Pago.
 * Returns the preapproval data including init_point URL for redirect.
 */
export async function createPreapproval(
  planName: string,
  amount: number,
  payerEmail: string,
  backUrl: string
): Promise<MpPreapprovalResponse> {
  const body: MpPreapprovalRequest = {
    payer_email: payerEmail,
    back_url: backUrl,
    reason: planName,
    auto_recurring: {
      frequency: 1,
      frequency_type: 'months',
      transaction_amount: amount,
      currency_id: 'COP',
    },
  };

  const response = await fetch(`${MP_API_URL}/preapproval`, {
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
      `MercadoPago createPreapproval failed: ${response.status} - ${error}`
    );
  }

  return response.json();
}

/**
 * Verify an IPN notification by fetching the preapproval status from MP API.
 * MP IPN sends topic + id; we verify by actually retrieving the resource.
 */
export async function verifyIPN(
  topic: string,
  id: string
): Promise<MpPreapprovalResponse | null> {
  if (topic !== 'subscription_preapproval') return null;
  return getPreapprovalStatus(id);
}

/**
 * Get preapproval (subscription) status from Mercado Pago.
 */
export async function getPreapprovalStatus(
  preapprovalId: string
): Promise<MpPreapprovalResponse> {
  const response = await fetch(`${MP_API_URL}/preapproval/${preapprovalId}`, {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${MP_ACCESS_TOKEN}`,
    },
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(
      `MercadoPago getPreapprovalStatus failed: ${response.status} - ${error}`
    );
  }

  return response.json();
}
