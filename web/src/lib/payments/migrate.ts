import { db } from '../db';

/**
 * Runs migration 002: adds multi-gateway columns to subscriptions and subscription_payments.
 * Each ALTER TABLE is executed individually with error swallowing for "duplicate column" —
 * this is the standard SQLite pattern since IF NOT EXISTS is not supported for ALTER TABLE.
 */
export async function runMigration002(): Promise<{ applied: string[]; skipped: string[] }> {
  const statements = [
    'ALTER TABLE subscriptions ADD COLUMN provider TEXT',
    'ALTER TABLE subscriptions ADD COLUMN provider_subscription_id TEXT',
    'ALTER TABLE subscriptions ADD COLUMN provider_customer_id TEXT',
    'ALTER TABLE subscriptions ADD COLUMN payment_source_token TEXT',
    'ALTER TABLE subscriptions ADD COLUMN grace_expires_at TEXT',
    'ALTER TABLE subscriptions ADD COLUMN trial_starts_at TEXT',
    'ALTER TABLE subscription_payments ADD COLUMN provider_payment_id TEXT',
  ];

  const applied: string[] = [];
  const skipped: string[] = [];

  for (const sql of statements) {
    try {
      await db.execute(sql);
      applied.push(sql);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      if (message.includes('duplicate column')) {
        skipped.push(sql);
      } else {
        throw err;
      }
    }
  }

  return { applied, skipped };
}
