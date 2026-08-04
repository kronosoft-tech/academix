export const prerender = false;

import type { APIRoute } from 'astro';
import {
  getExpiredGraceSubscriptions,
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

  const expired = await getExpiredGraceSubscriptions();

  for (const sub of expired) {
    await expireSubscription(sub.id);
  }

  return new Response(
    JSON.stringify({ expired: expired.length }),
    {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    }
  );
};
