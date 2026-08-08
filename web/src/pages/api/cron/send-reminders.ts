export const prerender = false;

import type { APIRoute } from 'astro';
import { db } from '../../../lib/db';
import { sendTrialReminder, sendGraceWarning } from '../../../lib/payments/email';

interface ReminderSubscription {
  id: string;
  user_id: string;
  email: string;
  academy_name: string;
  days_left: number;
}

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

  let trialReminders = 0;
  let graceWarnings = 0;

  // Query trial subscriptions in their last 7 days
  const now = new Date();
  const sevenDaysFromNow = new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000).toISOString();

  const trialResult = await db.execute({
    sql: `SELECT s.id, s.user_id, u.email, u.name as academy_name, s.trial_end
          FROM subscriptions s
          JOIN users u ON u.id = s.user_id
          WHERE s.status = 'trial'
            AND s.trial_end > ?
            AND s.trial_end <= ?`,
    args: [now.toISOString(), sevenDaysFromNow],
  });

  const trialSubs: ReminderSubscription[] = trialResult.rows.map((row) => {
    const trialEnd = new Date(row.trial_end as string);
    const diffMs = trialEnd.getTime() - now.getTime();
    const daysLeft = Math.max(1, Math.ceil(diffMs / (1000 * 60 * 60 * 24)));
    return {
      id: row.id as string,
      user_id: row.user_id as string,
      email: row.email as string,
      academy_name: (row.academy_name as string) || 'tu academia',
      days_left: daysLeft,
    };
  });

  for (const sub of trialSubs) {
    try {
      await sendTrialReminder(sub.email, sub.days_left, sub.academy_name);
      trialReminders++;
    } catch (err) {
      console.error(`Failed to send trial reminder to ${sub.email}:`, err);
    }
  }

  // Query grace subscriptions
  const graceResult = await db.execute({
    sql: `SELECT s.id, s.user_id, u.email, u.name as academy_name, s.grace_end
          FROM subscriptions s
          JOIN users u ON u.id = s.user_id
          WHERE s.status = 'grace'
            AND s.grace_end > ?`,
    args: [now.toISOString()],
  });

  const graceSubs: ReminderSubscription[] = graceResult.rows.map((row) => {
    const graceEnd = new Date(row.grace_end as string);
    const diffMs = graceEnd.getTime() - now.getTime();
    const daysLeft = Math.max(1, Math.ceil(diffMs / (1000 * 60 * 60 * 24)));
    return {
      id: row.id as string,
      user_id: row.user_id as string,
      email: row.email as string,
      academy_name: (row.academy_name as string) || 'tu academia',
      days_left: daysLeft,
    };
  });

  for (const sub of graceSubs) {
    try {
      await sendGraceWarning(sub.email, sub.days_left, sub.academy_name);
      graceWarnings++;
    } catch (err) {
      console.error(`Failed to send grace warning to ${sub.email}:`, err);
    }
  }

  return new Response(
    JSON.stringify({ trialReminders, graceWarnings }),
    {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    }
  );
};
