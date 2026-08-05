import { db } from '../db';

export interface Subscription {
  id: string;
  user_id: string;
  plan: string;
  status: string;
  trial_start: string | null;
  trial_end: string | null;
  grace_start: string | null;
  grace_end: string | null;
  stripe_subscription_id: string | null;
  current_period_start: string | null;
  current_period_end: string | null;
}

export async function createTrialSubscription(
  userId: string,
  plan: string,
  stripeSubId: string
): Promise<void> {
  const id = crypto.randomUUID();
  const now = new Date().toISOString();
  const trialEnd = new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString();

  await db.execute({
    sql: `INSERT INTO subscriptions (id, user_id, plan, status, trial_start, trial_end, stripe_subscription_id)
          VALUES (?, ?, ?, 'trial', ?, ?, ?)`,
    args: [id, userId, plan, now, trialEnd, stripeSubId],
  });
}

export async function activateSubscription(subscriptionId: string): Promise<void> {
  const now = new Date().toISOString();
  const periodEnd = new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString();

  await db.execute({
    sql: `UPDATE subscriptions
          SET status = 'active', trial_start = NULL, trial_end = NULL,
              grace_start = NULL, grace_end = NULL,
              current_period_start = ?, current_period_end = ?
          WHERE id = ?`,
    args: [now, periodEnd, subscriptionId],
  });
}

export async function startGracePeriod(subscriptionId: string): Promise<void> {
  const now = new Date().toISOString();
  const graceEnd = new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString();

  await db.execute({
    sql: `UPDATE subscriptions
          SET status = 'grace', grace_start = ?, grace_end = ?
          WHERE id = ?`,
    args: [now, graceEnd, subscriptionId],
  });
}

export async function expireSubscription(subscriptionId: string): Promise<void> {
  await db.execute({
    sql: `UPDATE subscriptions SET status = 'expired' WHERE id = ?`,
    args: [subscriptionId],
  });
}

export async function cancelSubscription(subscriptionId: string): Promise<void> {
  await db.execute({
    sql: `UPDATE subscriptions SET status = 'cancelled' WHERE id = ?`,
    args: [subscriptionId],
  });
}

export async function getExpiredGraceSubscriptions(): Promise<Subscription[]> {
  const now = new Date().toISOString();

  const result = await db.execute({
    sql: `SELECT id, user_id, plan, status, trial_start, trial_end,
                 grace_start, grace_end, stripe_subscription_id,
                 current_period_start, current_period_end
          FROM subscriptions
          WHERE status = 'grace' AND grace_end < ?`,
    args: [now],
  });

  return result.rows.map((row) => ({
    id: row.id as string,
    user_id: row.user_id as string,
    plan: row.plan as string,
    status: row.status as string,
    trial_start: row.trial_start as string | null,
    trial_end: row.trial_end as string | null,
    grace_start: row.grace_start as string | null,
    grace_end: row.grace_end as string | null,
    stripe_subscription_id: row.stripe_subscription_id as string | null,
    current_period_start: row.current_period_start as string | null,
    current_period_end: row.current_period_end as string | null,
  }));
}

export async function findSubscriptionByStripeId(
  stripeSubId: string
): Promise<Subscription | null> {
  const result = await db.execute({
    sql: `SELECT id, user_id, plan, status, trial_start, trial_end,
                 grace_start, grace_end, stripe_subscription_id,
                 current_period_start, current_period_end
          FROM subscriptions
          WHERE stripe_subscription_id = ?`,
    args: [stripeSubId],
  });

  if (result.rows.length === 0) return null;

  const row = result.rows[0];
  return {
    id: row.id as string,
    user_id: row.user_id as string,
    plan: row.plan as string,
    status: row.status as string,
    trial_start: row.trial_start as string | null,
    trial_end: row.trial_end as string | null,
    grace_start: row.grace_start as string | null,
    grace_end: row.grace_end as string | null,
    stripe_subscription_id: row.stripe_subscription_id as string | null,
    current_period_start: row.current_period_start as string | null,
    current_period_end: row.current_period_end as string | null,
  };
}
