-- Migration 002: Multi-gateway subscription support
-- Additive columns for generic provider support and trial tracking.
-- SQLite doesn't support IF NOT EXISTS for ALTER TABLE.
-- Each statement must be run individually; "duplicate column name" errors are safe to ignore.

ALTER TABLE subscriptions ADD COLUMN provider TEXT;
ALTER TABLE subscriptions ADD COLUMN provider_subscription_id TEXT;
ALTER TABLE subscriptions ADD COLUMN provider_customer_id TEXT;
ALTER TABLE subscriptions ADD COLUMN payment_source_token TEXT;
ALTER TABLE subscriptions ADD COLUMN grace_expires_at TEXT;
ALTER TABLE subscriptions ADD COLUMN trial_starts_at TEXT;

ALTER TABLE subscription_payments ADD COLUMN provider_payment_id TEXT;
