export const prerender = false;

import type { APIRoute } from 'astro';
import { createTransaction } from '../../../lib/payments/wompi';
import { startGracePeriod } from '../../../lib/payments/lifecycle';
import { getPlanById } from '../../../data/plans';
import { sendCronAlert } from '../../../lib/payments/cron-alert';
import { db } from '../../../lib/db';

interface DueSubscription {
  id: string;
  user_id: string;
  plan_id: string;
  payment_source_token: string;
  email: string;
}

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
    const now = new Date().toISOString();

    // Query active Wompi subscriptions that are due for renewal.
    const result = await db.execute({
      sql: `SELECT s.id, s.user_id, s.plan_id, s.payment_source_token, u.email
            FROM subscriptions s
            JOIN users u ON u.id = s.user_id
            WHERE s.provider = 'wompi'
              AND s.status = 'active'
              AND s.current_period_end <= ?
              AND s.payment_source_token IS NOT NULL`,
      args: [now],
    });

    const dueSubscriptions: DueSubscription[] = result.rows.map((row) => ({
      id: row.id as string,
      user_id: row.user_id as string,
      plan_id: row.plan_id as string,
      payment_source_token: row.payment_source_token as string,
      email: row.email as string,
    }));

    let charged = 0;
    let failed = 0;

    for (const sub of dueSubscriptions) {
      const plan = getPlanById(sub.plan_id);

      if (!plan) {
        // Unknown plan — fail loud (grace + log), never charge a silent
        // default/stale price.
        console.error(
          `[cron:charge-wompi] Unknown plan_id '${sub.plan_id}' for subscription ${sub.id}; starting grace period`
        );
        await startGracePeriod(sub.id);
        failed++;
        continue;
      }

      const reference = `renewal-${sub.id}-${Date.now()}`;
      const amountInCents = plan.priceCOP * 100;

      try {
        await createTransaction(
          amountInCents,
          'COP',
          reference,
          parseInt(sub.payment_source_token, 10),
          sub.email
        );
        // Transaction created — confirmation comes via webhook.
        charged++;
      } catch {
        // Immediate failure — start grace period and continue with the rest.
        await startGracePeriod(sub.id);
        failed++;
      }
    }

    return new Response(
      JSON.stringify({ charged, failed, total: dueSubscriptions.length }),
      {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }
    );
  } catch (err) {
    await sendCronAlert('charge-wompi', err);
    return new Response(JSON.stringify({ error: 'Cron failed' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' },
    });
  }
};