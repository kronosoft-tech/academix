export type Gateway = 'wompi' | 'mercadopago';

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
 * Colombia → Wompi, everything else → MercadoPago.
 */
export function geoToGateway(countryCode: string | null): Gateway {
  if (!countryCode) return 'wompi';
  const code = countryCode.toUpperCase();
  if (code === 'CO') return 'wompi';
  return 'mercadopago';
}
