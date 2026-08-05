import { defineAction, ActionError } from 'astro:actions';
import { z } from 'astro/zod';
import { getStripe } from '../lib/payments/stripe';

const PRICE_MAP: Record<string, string> = {
  basic: 'STRIPE_PRICE_BASIC',
  pro: 'STRIPE_PRICE_PRO',
  premium: 'STRIPE_PRICE_PREMIUM',
};

export const createCheckout = defineAction({
  input: z.object({
    plan: z.enum(['basic', 'pro', 'premium']),
  }),
  handler: async (input, context) => {
    const user = context.locals.user;
    if (!user) {
      throw new ActionError({
        code: 'UNAUTHORIZED',
        message: 'Debes iniciar sesión para suscribirte',
      });
    }

    const priceEnvKey = PRICE_MAP[input.plan];
    const priceId = import.meta.env[priceEnvKey];

    if (!priceId) {
      throw new ActionError({
        code: 'INTERNAL_SERVER_ERROR',
        message: 'Plan no configurado correctamente',
      });
    }

    const stripe = getStripe();
    const session = await stripe.checkout.sessions.create({
      mode: 'subscription',
      payment_method_types: ['card'],
      line_items: [{ price: priceId, quantity: 1 }],
      success_url: `${context.url.origin}/dashboard?session_id={CHECKOUT_SESSION_ID}`,
      cancel_url: `${context.url.origin}/pricing`,
      client_reference_id: user.id,
      subscription_data: {
        trial_period_days: 7,
      },
    });

    return { url: session.url };
  },
});
