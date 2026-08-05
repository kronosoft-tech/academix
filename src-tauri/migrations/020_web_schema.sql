-- Web schema: additive tables for web platform features
-- NO changes to existing tables

CREATE TABLE IF NOT EXISTS web_admins (
  id TEXT PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  name TEXT NOT NULL,
  role TEXT NOT NULL CHECK(role IN ('superadmin', 'employee', 'manager')),
  is_active INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_web_admins_email ON web_admins(email);

CREATE TABLE IF NOT EXISTS subscriptions (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  plan TEXT NOT NULL CHECK(plan IN ('basic', 'pro', 'premium')),
  status TEXT NOT NULL CHECK(status IN ('trial', 'active', 'grace', 'expired', 'cancelled')),
  trial_start TEXT,
  trial_end TEXT,
  grace_start TEXT,
  grace_end TEXT,
  stripe_subscription_id TEXT,
  current_period_start TEXT,
  current_period_end TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS subscription_payments (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  subscription_id TEXT,
  amount REAL NOT NULL,
  currency TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('pending', 'succeeded', 'failed', 'refunded')),
  provider TEXT NOT NULL CHECK(provider IN ('stripe', 'mercadopago', 'payu')),
  provider_payment_id TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS invoices (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  payment_id TEXT,
  number TEXT NOT NULL,
  pdf_url TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS pqrs_tickets (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  type TEXT NOT NULL CHECK(type IN ('petition', 'complaint', 'claim', 'suggestion')),
  subject TEXT NOT NULL,
  description TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'open' CHECK(status IN ('open', 'in_progress', 'resolved')),
  assigned_to TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  resolved_at TEXT
);

CREATE TABLE IF NOT EXISTS pqrs_responses (
  id TEXT PRIMARY KEY,
  ticket_id TEXT NOT NULL,
  author_id TEXT NOT NULL,
  message TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY(ticket_id) REFERENCES pqrs_tickets(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS password_resets (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  token TEXT NOT NULL UNIQUE,
  expires_at TEXT NOT NULL,
  used_at TEXT
);

CREATE TABLE IF NOT EXISTS downloads (
  id TEXT PRIMARY KEY,
  os TEXT,
  arch TEXT,
  version TEXT,
  ip TEXT,
  country TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ai_conversations (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  provider TEXT,
  model TEXT,
  messages_json TEXT,
  created_at TEXT NOT NULL
);
