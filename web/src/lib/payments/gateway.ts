export type Gateway = 'stripe' | 'wompi' | 'mercadopago';

export interface NormalizedWebhookResult {
  action: 'activate' | 'grace' | 'cancel' | 'payment' | 'ignore';
  providerSubId?: string;
  userId?: string;
  amount?: number;
  currency?: string;
  providerPaymentId?: string;
}

export interface CheckoutResult {
  url?: string;
  widgetToken?: string;
}

const LATAM_COUNTRIES = [
  'AR', 'MX', 'BR', 'CL', 'PE', 'UY', 'EC', 'VE', 'PY', 'BO',
  'PA', 'CR', 'GT', 'HN', 'SV', 'NI', 'DO',
];

/**
 * Determines the payment gateway based on user country code.
 * Colombia → Wompi, other LatAm → MercadoPago, rest → Stripe.
 */
export function geoToGateway(countryCode: string | null): Gateway {
  if (!countryCode) return 'stripe';
  const code = countryCode.toUpperCase();
  if (code === 'CO') return 'wompi';
  if (LATAM_COUNTRIES.includes(code)) return 'mercadopago';
  return 'stripe';
}
