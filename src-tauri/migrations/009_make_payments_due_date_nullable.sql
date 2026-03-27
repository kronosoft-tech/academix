-- Migration: 009_make_payments_due_date_nullable
-- Description: Make payments.due_date nullable since it's calculated and may not be set

BEGIN TRANSACTION;

-- Make due_date column nullable
ALTER TABLE payments ALTER COLUMN due_date DROP NOT NULL;

-- Also ensure paid_date is nullable (already should be)
ALTER TABLE payments ALTER COLUMN paid_date DROP NOT NULL;

COMMIT;