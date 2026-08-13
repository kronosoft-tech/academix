-- Migration: 019_add_payment_type
-- Description: Add payment_type column to distinguish enrollment from tuition payments

ALTER TABLE payments ADD COLUMN payment_type TEXT NOT NULL DEFAULT 'tuition' CHECK(payment_type IN ('enrollment', 'tuition'));

CREATE INDEX IF NOT EXISTS idx_payments_payment_type ON payments(payment_type);
