import { useState, useEffect } from 'react';
import { PLANS, type Plan } from '../data/plans';

type Gateway = 'wompi' | 'mercadopago';

interface Props {
  countryCode: string;
  currencyCode: string;
  currencySymbol: string;
  prices: { basic: number; pro: number; premium: number };
  displayName: string;
}

function formatPrice(amount: number, currencyCode: string): string {
  return new Intl.NumberFormat('es', {
    style: 'currency',
    currency: currencyCode,
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(amount);
}

export default function CheckoutPlans({ countryCode, currencyCode, prices }: Props) {
  const defaultGateway: Gateway = countryCode?.toUpperCase() === 'CO' ? 'wompi' : 'mercadopago';
  const [gateway, setGateway] = useState<Gateway>(defaultGateway);
  const [loadingPlan, setLoadingPlan] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const safePrices = prices || { basic: 29, pro: 49, premium: 79 };
  const safeCurrency = currencyCode || 'USD';

  const planPrices: Record<string, number> = {
    basico: safePrices.basic,
    pro: safePrices.pro,
    premium: safePrices.premium,
  };

  // Load Wompi Widget script
  useEffect(() => {
    if (document.getElementById('wompi-widget-script')) return;
    const script = document.createElement('script');
    script.id = 'wompi-widget-script';
    script.src = 'https://checkout.wompi.co/widget.js';
    script.async = true;
    document.head.appendChild(script);
  }, []);

  const handleSubscribe = async (plan: Plan) => {
    setError(null);
    setLoadingPlan(plan.id);

    try {
      const response = await fetch(`/api/checkout/${gateway}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ planId: plan.id }),
      });

      if (response.status === 401) {
        window.location.href = '/auth/login?redirect=/pricing';
        return;
      }

      if (!response.ok) {
        const data = await response.json().catch(() => null);
        throw new Error(data?.error || 'Error al crear la sesión de pago');
      }

      const data = await response.json();

      if (gateway === 'mercadopago' && data.url) {
        // Mercado Pago Checkout Pro — redirect
        window.location.href = data.url;
      } else if (gateway === 'wompi' && data.publicKey) {
        // Wompi Widget — open popup checkout
        const WidgetCheckout = (window as any).WidgetCheckout;
        if (!WidgetCheckout) {
          throw new Error('Wompi widget not loaded. Refresh the page.');
        }

        const checkout = new WidgetCheckout({
          currency: data.currency,
          amountInCents: data.amountInCents,
          reference: data.reference,
          publicKey: data.publicKey,
          signature: { integrity: data.integrity },
          redirectUrl: data.redirectUrl,
          customerData: {
            email: data.customerEmail,
          },
        });

        checkout.open((result: any) => {
          const transaction = result.transaction;
          if (transaction && transaction.status === 'APPROVED') {
            window.location.href = `${data.redirectUrl}?id=${transaction.id}`;
          } else if (transaction) {
            setError(`Pago ${transaction.status === 'DECLINED' ? 'rechazado' : 'pendiente'}. Intenta de nuevo.`);
          }
        });
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error inesperado');
    } finally {
      setLoadingPlan(null);
    }
  };

  return (
    <div style={{ width: '100%' }}>
      {/* Gateway selector */}
      <div style={{ display: 'flex', justifyContent: 'center', marginBottom: '24px', gap: '8px' }}>
        <button
          onClick={() => setGateway('wompi')}
          className={`px-4 py-2 rounded-lg text-sm font-medium border transition-colors ${gateway === 'wompi'
              ? 'border-emerald-500 text-emerald-400 bg-emerald-500/10'
              : 'border-slate-700 text-slate-400 hover:border-slate-500'
            }`}
        >
          Bancolombia / Nequi / PSE
        </button>
        <button
          onClick={() => setGateway('mercadopago')}
          className={`px-4 py-2 rounded-lg text-sm font-medium border transition-colors ${gateway === 'mercadopago'
              ? 'border-emerald-500 text-emerald-400 bg-emerald-500/10'
              : 'border-slate-700 text-slate-400 hover:border-slate-500'
            }`}
        >
          Mercado Pago
        </button>
      </div>

      {/* Error */}
      {error && (
        <div className="mb-4 p-3 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm">
          {error}
          <button onClick={() => setError(null)} className="ml-2 underline">Cerrar</button>
        </div>
      )}

      {/* Plan cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {PLANS.map((plan) => {
          const isRecommended = plan.id === 'pro';
          const isLoading = loadingPlan === plan.id;
          const localPrice = planPrices[plan.id] || plan.priceUSD;

          return (
            <div
              key={plan.id}
              className={`relative rounded-xl border p-6 flex flex-col ${isRecommended
                  ? 'border-emerald-500 bg-slate-900'
                  : 'border-slate-700 bg-slate-900/50'
                }`}
            >
              {isRecommended && (
                <span className="absolute -top-3 left-1/2 -translate-x-1/2 px-3 py-1 text-xs font-medium rounded-full bg-emerald-600 text-white">
                  Recomendado
                </span>
              )}

              <h3 className="text-xl font-bold text-white mb-2">{plan.name}</h3>

              <div className="mb-4">
                <span className="text-3xl font-bold text-white">
                  {formatPrice(localPrice, safeCurrency)}
                </span>
                <span className="text-slate-400 text-sm ml-1">/ mes</span>
              </div>

              <ul className="space-y-2 flex-1 mb-6">
                {plan.features.map((feature) => (
                  <li key={feature} className="flex items-center gap-2 text-sm text-slate-300">
                    <svg className="w-4 h-4 text-emerald-400 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                    {feature}
                  </li>
                ))}
              </ul>

              <button
                onClick={() => handleSubscribe(plan)}
                disabled={isLoading || loadingPlan !== null}
                className={`w-full py-3 rounded-lg font-semibold transition-colors text-center disabled:opacity-50 disabled:cursor-not-allowed ${isRecommended
                    ? 'bg-emerald-600 text-white hover:bg-emerald-500'
                    : 'border border-slate-600 text-slate-200 hover:border-slate-400 hover:text-white'
                  }`}
              >
                {isLoading ? 'Procesando...' : isRecommended ? 'Suscribirse' : 'Comenzar'}
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
}
