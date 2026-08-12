import { defineAction, ActionError } from 'astro:actions';
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
      sql: `SELECT id FROM subscriptions
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

    await cancelSubscription(result.rows[0].id as string);

    return { success: true };
  },
});
