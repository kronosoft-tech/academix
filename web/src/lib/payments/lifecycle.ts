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
  provider: string | null;
  provider_subscription_id: string | null;
  provider_customer_id: string | null;
  trial_starts_at: string | null;
  grace_expires_at: string | null;
}

export async function createTrialSubscription(
  userId: string,
  plan: string,
  stripeSubId: string | null,
  provider: string | null = null
): Promise<void> {
  const id = crypto.randomUUID();
  const now = new Date().toISOString();
  const trialEnd = new Date(Date.now() + 15 * 24 * 60 * 60 * 1000).toISOString();

  await db.execute({
    sql: `INSERT INTO subscriptions (id, user_id, plan_id, status, trial_start, trial_end, stripe_subscription_id, trial_starts_at, provider, created_at, updated_at)
          VALUES (?, ?, ?, 'trial', ?, ?, ?, ?, ?, ?, ?)`,
    args: [id, userId, plan, now, trialEnd, stripeSubId, now, provider, now, now],
  });
}

export async function getExpiredTrials(): Promise<Subscription[]> {
  const now = new Date().toISOString();

  const result = await db.execute({
    sql: `SELECT id, user_id, plan, status, trial_start, trial_end,
                 grace_start, grace_end, stripe_subscription_id,
                 current_period_start, current_period_end,
                 provider, provider_subscription_id, provider_customer_id,
                 trial_starts_at, grace_expires_at
          FROM subscriptions
          WHERE status = 'trial' AND trial_end < ?`,
    args: [now],
  });

  return result.rows.map(mapRowToSubscription);
}

export async function findByProviderSubId(providerSubId: string): Promise<Subscription | null> {
  const result = await db.execute({
    sql: `SELECT id, user_id, plan, status, trial_start, trial_end,
                 grace_start, grace_end, stripe_subscription_id,
                 current_period_start, current_period_end,
                 provider, provider_subscription_id, provider_customer_id,
                 trial_starts_at, grace_expires_at
          FROM subscriptions
          WHERE provider_subscription_id = ?`,
    args: [providerSubId],
  });

  if (result.rows.length === 0) return null;
  return mapRowToSubscription(result.rows[0]);
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
                 current_period_start, current_period_end,
                 provider, provider_subscription_id, provider_customer_id,
                 trial_starts_at, grace_expires_at
          FROM subscriptions
          WHERE status = 'grace' AND grace_end < ?`,
    args: [now],
  });

  return result.rows.map(mapRowToSubscription);
}

export async function findSubscriptionByStripeId(
  stripeSubId: string
): Promise<Subscription | null> {
  const result = await db.execute({
    sql: `SELECT id, user_id, plan, status, trial_start, trial_end,
                 grace_start, grace_end, stripe_subscription_id,
                 current_period_start, current_period_end,
                 provider, provider_subscription_id, provider_customer_id,
                 trial_starts_at, grace_expires_at
          FROM subscriptions
          WHERE stripe_subscription_id = ?`,
    args: [stripeSubId],
  });

  if (result.rows.length === 0) return null;
  return mapRowToSubscription(result.rows[0]);
}

function mapRowToSubscription(row: Record<string, unknown>): Subscription {
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
    provider: row.provider as string | null,
    provider_subscription_id: row.provider_subscription_id as string | null,
    provider_customer_id: row.provider_customer_id as string | null,
    trial_starts_at: row.trial_starts_at as string | null,
    grace_expires_at: row.grace_expires_at as string | null,
  };
}
