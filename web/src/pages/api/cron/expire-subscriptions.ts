export const prerender = false;

import type { APIRoute } from 'astro';
import {
  getExpiredGraceSubscriptions,
  getExpiredTrials,
  expireSubscription,
  cancelSubscription,
} from '../../../lib/payments/lifecycle';
import { sendCronAlert } from '../../../lib/payments/cron-alert';

export const GET: APIRoute = async ({ request }) => {
  // Verify cron secret — stays OUTSIDE the catch: missing secret (500) and
  // bad auth (401) must not trigger the support alert.
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

  try {
    // End grace-period subscriptions (status='grace' filter makes this
    // idempotent — already-cancelled/expired/activated rows never re-enter).
    const expiredGrace = await getExpiredGraceSubscriptions();
    for (const sub of expiredGrace) {
      await cancelSubscription(sub.id);
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
  } catch (err) {
    await sendCronAlert('expire-subscriptions', err);
    return new Response(JSON.stringify({ error: 'Cron failed' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' },
    });
  }
};