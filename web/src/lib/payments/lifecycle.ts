import { db } from '../db';

export interface Subscription {
  id: string;
  user_id: string;
  plan_id: string;
  status: string;
  trial_end: string | null;
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
    sql: `INSERT INTO subscriptions (id, user_id, plan_id, status, trial_end, stripe_subscription_id, trial_starts_at, provider, created_at, updated_at)
          VALUES (?, ?, ?, 'trial', ?, ?, ?, ?, ?, ?)`,
    args: [id, userId, plan, trialEnd, stripeSubId, now, provider, now, now],
  });
}

/**
 * Return the user's trial subscription, lazily creating a default trial row
 * when the user has no subscription at all (e.g. desktop-registered users who
 * never went through the web register flow). Without this, every payment path
 * silently no-ops for those users.
 */
export async function getOrCreateTrialSubscription(
  userId: string,
  plan: string = 'basico'
): Promise<{ id: string }> {
  const trial = await db.execute({
    sql: `SELECT id FROM subscriptions WHERE user_id = ? AND status = 'trial' LIMIT 1`,
    args: [userId],
  });
  if (trial.rows.length > 0) return { id: trial.rows[0].id as string };

  const anyRow = await db.execute({
    sql: `SELECT id FROM subscriptions WHERE user_id = ? LIMIT 1`,
    args: [userId],
  });
  if (anyRow.rows.length > 0) return { id: anyRow.rows[0].id as string };

  await createTrialSubscription(userId, plan, null);
  const created = await db.execute({
    sql: `SELECT id FROM subscriptions WHERE user_id = ? AND status = 'trial' LIMIT 1`,
    args: [userId],
  });
  return { id: created.rows[0].id as string };
}

export async function getExpiredTrials(): Promise<Subscription[]> {
  const now = new Date().toISOString();

  const result = await db.execute({
    sql: `SELECT id, user_id, plan_id, status, trial_end,
                 stripe_subscription_id,
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
    sql: `SELECT id, user_id, plan_id, status, trial_end,
                 stripe_subscription_id,
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
          SET status = 'active', trial_end = NULL, trial_starts_at = NULL,
              grace_expires_at = NULL,
              current_period_start = ?, current_period_end = ?
          WHERE id = ?`,
    args: [now, periodEnd, subscriptionId],
  });
}

export async function startGracePeriod(subscriptionId: string): Promise<void> {
  const graceEnd = new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString();

  await db.execute({
    sql: `UPDATE subscriptions
          SET status = 'grace', grace_expires_at = ?
          WHERE id = ?`,
    args: [graceEnd, subscriptionId],
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
    sql: `SELECT id, user_id, plan_id, status, trial_end,
                 stripe_subscription_id,
                 current_period_start, current_period_end,
                 provider, provider_subscription_id, provider_customer_id,
                 trial_starts_at, grace_expires_at
          FROM subscriptions
          WHERE status = 'grace' AND grace_expires_at < ?`,
    args: [now],
  });

  return result.rows.map(mapRowToSubscription);
}

export async function findSubscriptionByStripeId(
  stripeSubId: string
): Promise<Subscription | null> {
  const result = await db.execute({
    sql: `SELECT id, user_id, plan_id, status, trial_end,
                 stripe_subscription_id,
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
    plan_id: row.plan_id as string,
    status: row.status as string,
    trial_end: row.trial_end as string | null,
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
