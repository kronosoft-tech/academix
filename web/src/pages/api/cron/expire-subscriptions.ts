export const prerender = false;

import type { APIRoute } from 'astro';
import {
  getExpiredGraceSubscriptions,
  getExpiredTrials,
  expireSubscription,
} from '../../../lib/payments/lifecycle';

export const GET: APIRoute = async ({ request }) => {
  // Verify cron secret
  const authHeader = request.headers.get('authorization');
  const cronSecret = import.meta.env.CRON_SECRET;

  if (!cronSecret) {
    return new Response(JSON.stringify({ error: 'CRON_SECRET not configured' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  if (authHeader !== `Bearer ${cronSecret}`) {
    return new Response(JSON.stringify({ error: 'Unauthorized' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  // Expire grace-period subscriptions
  const expiredGrace = await getExpiredGraceSubscriptions();
  for (const sub of expiredGrace) {
    await expireSubscription(sub.id);
  }

  // Expire trial subscriptions past their trial_end
  const expiredTrials = await getExpiredTrials();
  for (const sub of expiredTrials) {
    await expireSubscription(sub.id);
  }

  return new Response(
    JSON.stringify({
      expiredGrace: expiredGrace.length,
      expiredTrials: expiredTrials.length,
    }),
    {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    }
  );
};
