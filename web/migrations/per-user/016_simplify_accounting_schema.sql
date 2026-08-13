-- Migration: 016_simplify_accounting_schema
-- Description: Simplify accounting to income/expense model
-- Created: 2026-07-14
--
-- Changes:
--   1. Recreate accounting_entries with simplified schema
--   2. Migrate existing data (PUC accounts -> type/category)
--   3. Drop old tables: account_categories, employees, payroll_runs,
--      payroll_entries, invoices, invoice_lines, liabilities, equities

-- ============================================
-- STEP 1: Create new simplified accounting_entries table
-- ============================================
CREATE TABLE IF NOT EXISTS accounting_entries_new (
    id TEXT PRIMARY KEY,
    date TEXT NOT NULL,
    type TEXT NOT NULL CHECK(type IN ('income', 'expense')),
    category TEXT NOT NULL,
    description TEXT NOT NULL,
    amount REAL NOT NULL CHECK(amount > 0),
    reference TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_accounting_entries_new_date ON accounting_entries_new(date);
CREATE INDEX IF NOT EXISTS idx_accounting_entries_new_type ON accounting_entries_new(type);
CREATE INDEX IF NOT EXISTS idx_accounting_entries_new_category ON accounting_entries_new(category);

-- ============================================
-- STEP 2: Migrate existing data
-- ============================================
-- Map PUC accounts to simplified categories:
-- - Credit accounts 6xxx (income) -> type='income', category='tuition'
-- - Debit accounts 4xxx (expenses) -> type='expense', category='other'
-- - We use the credit_account to determine if it's income

INSERT INTO accounting_entries_new (id, date, type, category, description, amount, reference, created_at)
SELECT
    id,
    date,
    CASE
        WHEN credit_account LIKE '6%' THEN 'income'
        ELSE 'expense'
    END as type,
    CASE
        WHEN credit_account LIKE '6%' THEN 'tuition'
        WHEN debit_account LIKE '410%' THEN 'rent'
        WHEN debit_account LIKE '412%' THEN 'salaries'
        WHEN debit_account LIKE '413%' THEN 'utilities'
        ELSE 'other'
    END as category,
    description,
    amount,
    reference,
    created_at
FROM accounting_entries;

-- ============================================
-- STEP 3: Drop old tables and views
-- ============================================

-- Drop the old accounting_entries table
DROP TABLE IF EXISTS accounting_entries;

-- Rename new table to accounting_entries
ALTER TABLE accounting_entries_new RENAME TO accounting_entries;

-- Re-create indexes after rename
CREATE INDEX IF NOT EXISTS idx_accounting_entries_date ON accounting_entries(date);
CREATE INDEX IF NOT EXISTS idx_accounting_entries_type ON accounting_entries(type);
CREATE INDEX IF NOT EXISTS idx_accounting_entries_category ON accounting_entries(category);
CREATE INDEX IF NOT EXISTS idx_accounting_entries_reference ON accounting_entries(reference);

-- Disable FK enforcement for safe table drops (libsql/Hrana enforces FKs)
PRAGMA foreign_keys=OFF;

-- Drop child tables first (those with FK references to other tables)
DROP TABLE IF EXISTS payroll_entries;
DROP TABLE IF EXISTS invoice_lines;

-- Drop parent tables (now safe — no remaining FK references)
DROP TABLE IF EXISTS payroll_runs;
DROP TABLE IF EXISTS employees;
DROP TABLE IF EXISTS invoices;
DROP TABLE IF EXISTS account_categories;
DROP TABLE IF EXISTS liabilities;
DROP TABLE IF EXISTS equities;

-- Drop fixed_assets if it exists (from migration 013)
DROP TABLE IF EXISTS fixed_assets;

-- Drop views if they exist
DROP VIEW IF EXISTS trial_balance;
DROP VIEW IF EXISTS financial_balance;

-- Re-enable FK enforcement
PRAGMA foreign_keys=ON;
