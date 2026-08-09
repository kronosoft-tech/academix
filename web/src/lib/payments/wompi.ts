const WOMPI_API_URL =
  import.meta.env.WOMPI_API_URL || 'https://production.wompi.co/v1';
const WOMPI_PUBLIC_KEY = import.meta.env.WOMPI_PUBLIC_KEY || '';
const WOMPI_PRIVATE_KEY = import.meta.env.WOMPI_PRIVATE_KEY || '';
const WOMPI_EVENTS_SECRET = import.meta.env.WOMPI_EVENTS_SECRET || '';

export interface WompiTransactionResponse {
  data: {
    id: string;
    status: string;
    reference: string;
    amount_in_cents: number;
    currency: string;
  };
}

export interface WompiMerchantResponse {
  data: {
    id: number;
    name: string;
    presigned_acceptance: {
      acceptance_token: string;
      permalink: string;
      type: string;
    };
  };
}

export interface WompiWebhookEvent {
  event: string;
  data: {
    transaction: {
      id: string;
      status: string;
      reference: string;
      amount_in_cents: number;
      currency: string;
      payment_source_id?: number;
    };
  };
  timestamp: number;
  signature?: {
    checksum: string;
    properties: string[];
  };
  sent_at: string;
}

/**
 * Create a transaction via Wompi API using a stored payment source.
 */
export async function createTransaction(
  amountInCents: number,
  currency: string,
  reference: string,
  paymentSourceId: number,
  customerEmail: string
): Promise<WompiTransactionResponse> {
  const response = await fetch(`${WOMPI_API_URL}/transactions`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${WOMPI_PRIVATE_KEY}`,
    },
    body: JSON.stringify({
      amount_in_cents: amountInCents,
      currency,
      reference,
      payment_source_id: paymentSourceId,
      customer_email: customerEmail,
    }),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Wompi createTransaction failed: ${response.status} - ${error}`);
  }

  return response.json();
}

/**
 * Get acceptance token from Wompi merchants endpoint.
 * Required for the client-side widget to show terms acceptance.
 */
export async function getAcceptanceToken(): Promise<string> {
  const response = await fetch(`${WOMPI_API_URL}/merchants/${WOMPI_PUBLIC_KEY}`, {
    method: 'GET',
    headers: { 'Content-Type': 'application/json' },
  });

  if (!response.ok) {
    throw new Error(`Wompi getAcceptanceToken failed: ${response.status}`);
  }

  const data: WompiMerchantResponse = await response.json();
  return data.data.presigned_acceptance.acceptance_token;
}

/**
 * Verify Wompi webhook signature.
 * Formula: SHA256(transaction.id + transaction.status + transaction.reference + timestamp + events_secret)
 */
export async function verifyWebhookSignature(
  event: WompiWebhookEvent,
  receivedChecksum: string
): Promise<boolean> {
  const { transaction } = event.data;
  const concatenated =
    transaction.id +
    transaction.status +
    transaction.reference +
    event.timestamp +
    WOMPI_EVENTS_SECRET;

  const encoder = new TextEncoder();
  const data = encoder.encode(concatenated);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const computedChecksum = hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');

  return computedChecksum === receivedChecksum;
}

export function getPublicKey(): string {
  return WOMPI_PUBLIC_KEY;
}
