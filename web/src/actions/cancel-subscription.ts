import { defineAction, ActionError } from 'astro:actions';
import { getStripe } from '../lib/payments/stripe';
import { db } from '../lib/db';
import { cancelSubscription } from '../lib/payments/lifecycle';

export const cancelSub = defineAction({
  handler: async (_input, context) => {
    const user = context.locals.user;
    if (!user) {
      throw new ActionError({
        code: 'UNAUTHORIZED',
        message: 'Debes iniciar sesión',
      });
    }

    const result = await db.execute({
      sql: `SELECT id, stripe_subscription_id
            FROM subscriptions
            WHERE user_id = ? AND status IN ('trial', 'active', 'grace')
            ORDER BY created_at DESC LIMIT 1`,
      args: [user.id],
    });

    if (result.rows.length === 0) {
      throw new ActionError({
        code: 'NOT_FOUND',
        message: 'No tienes una suscripción activa',
      });
    }

    const subscription = result.rows[0];
    const stripeSubId = subscription.stripe_subscription_id as string;

    if (stripeSubId) {
      const stripe = getStripe();
      await stripe.subscriptions.cancel(stripeSubId);
    }

    await cancelSubscription(subscription.id as string);

    return { success: true };
  },
});
