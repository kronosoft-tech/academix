-- Migration: 011_liabilities_equity_schema
-- Description: Create tables for liabilities (debts) and equity (patrimonio)
-- Created: 2026-04-23
--
-- Tables:
--   - liabilities: Debt/obligation tracking
--   - equities: Capital, reserves, and results

-- ============================================
-- LIABILITIES - Pasivos (debts and obligations)
-- ============================================
CREATE TABLE IF NOT EXISTS liabilities (
    id TEXT PRIMARY KEY,
    provider_name TEXT NOT NULL,
    document_type TEXT NOT NULL CHECK(document_type IN ('factura', 'recibo', 'letra', 'contrato', 'otro')),
    document_number TEXT NOT NULL,
    amount REAL NOT NULL DEFAULT 0,
    paid_amount REAL NOT NULL DEFAULT 0,
    liability_type TEXT NOT NULL CHECK(liability_type IN ('short_term', 'long_term', 'provisions')),
    due_date TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'partial', 'paid', 'overdue')),
    description TEXT,
    account_code TEXT,  -- Link to chart of accounts (2xxx)
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_liabilities_status ON liabilities(status);
CREATE INDEX IF NOT EXISTS idx_liabilities_due_date ON liabilities(due_date);
CREATE INDEX IF NOT EXISTS idx_liabilities_liability_type ON liabilities(liability_type);

-- ============================================
-- EQUITIES - Patrimonio (capital, reserves, results)
-- ============================================
CREATE TABLE IF NOT EXISTS equities (
    id TEXT PRIMARY KEY,
    equity_type TEXT NOT NULL CHECK(equity_type IN ('capital', 'reserves', 'results', 'retained')),
    description TEXT NOT NULL,
    amount REAL NOT NULL DEFAULT 0,
    account_code TEXT,  -- Link to chart of accounts (3xxx)
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_equities_equity_type ON equities(equity_type);