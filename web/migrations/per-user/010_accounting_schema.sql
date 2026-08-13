-- Migration: 010_accounting_schema
-- Description: Create accounting tables for payroll, accounting entries, and invoices
-- Created: 2026-04-13
-- 
-- Tables:
--   - employees: Employee management
--   - payroll_runs: Payroll batch runs
--   - payroll_entries: Individual payroll entries
--   - accounting_entries: General ledger entries
--   - account_categories: Chart of accounts
--   - invoices: Invoice generation
--   - invoice_lines: Invoice details

-- ============================================
-- EMPLOYEES - Employee management
-- ============================================
CREATE TABLE IF NOT EXISTS employees (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    document_type TEXT NOT NULL CHECK(document_type IN ('DNI', 'CE', 'RUC', 'PASSPORT')),
    document_number TEXT NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT NOT NULL,
    phone TEXT,
    address TEXT,
    position TEXT NOT NULL,
    department TEXT NOT NULL,
    contract_type TEXT NOT NULL CHECK(contract_type IN ('fixed', 'indefinite', 'hourly', 'services')),
    base_salary REAL NOT NULL DEFAULT 0,
    bank_name TEXT,
    bank_account TEXT,
    account_type TEXT CHECK(account_type IN ('savings', 'checking')),
    cci TEXT,
    afp TEXT CHECK(afp IN ('prima', 'habitat', 'integra', 'profuturo', 'onp')),
    hire_date TEXT NOT NULL,
    termination_date TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'inactive', 'terminated')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_employees_document_number ON employees(document_number);
CREATE INDEX IF NOT EXISTS idx_employees_email ON employees(email);
CREATE INDEX IF NOT EXISTS idx_employees_status ON employees(status);
CREATE INDEX IF NOT EXISTS idx_employees_department ON employees(department);

-- ============================================
-- PAYROLL_RUNS - Payroll batch runs
-- ============================================
CREATE TABLE IF NOT EXISTS payroll_runs (
    id TEXT PRIMARY KEY,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft', 'calculated', 'confirmed', 'cancelled')),
    total_gross REAL NOT NULL DEFAULT 0,
    total_deductions REAL NOT NULL DEFAULT 0,
    total_net REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    created_by TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_payroll_runs_period_start ON payroll_runs(period_start);
CREATE INDEX IF NOT EXISTS idx_payroll_runs_status ON payroll_runs(status);

-- ============================================
-- PAYROLL_ENTRIES - Individual payroll entries
-- ============================================
CREATE TABLE IF NOT EXISTS payroll_entries (
    id TEXT PRIMARY KEY,
    payroll_run_id TEXT NOT NULL,
    employee_id TEXT NOT NULL,
    base_salary REAL NOT NULL DEFAULT 0,
    hours_worked REAL NOT NULL DEFAULT 0,
    overtime_hours REAL NOT NULL DEFAULT 0,
    overtime_amount REAL NOT NULL DEFAULT 0,
    bonuses REAL NOT NULL DEFAULT 0,
    commissions REAL NOT NULL DEFAULT 0,
    mobility REAL NOT NULL DEFAULT 0,
    food REAL NOT NULL DEFAULT 0,
    other_income REAL NOT NULL DEFAULT 0,
    afp_deduction REAL NOT NULL DEFAULT 0,
    onp_deduction REAL NOT NULL DEFAULT 0,
    essalud REAL NOT NULL DEFAULT 0,
    itf REAL NOT NULL DEFAULT 0,
    other_deductions REAL NOT NULL DEFAULT 0,
    gross_income REAL NOT NULL DEFAULT 0,
    net_income REAL NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'calculated' CHECK(status IN ('calculated', 'paid', 'cancelled')),
    created_at TEXT NOT NULL,
    FOREIGN KEY (payroll_run_id) REFERENCES payroll_runs(id) ON DELETE CASCADE,
    FOREIGN KEY (employee_id) REFERENCES employees(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_payroll_entries_payroll_run_id ON payroll_entries(payroll_run_id);
CREATE INDEX IF NOT EXISTS idx_payroll_entries_employee_id ON payroll_entries(employee_id);
CREATE INDEX IF NOT EXISTS idx_payroll_entries_status ON payroll_entries(status);

-- ============================================
-- ACCOUNT_CATEGORIES - Chart of accounts
-- ============================================
CREATE TABLE IF NOT EXISTS account_categories (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    category_type TEXT NOT NULL CHECK(category_type IN ('asset', 'liability', 'equity', 'expense', 'cost', 'income')),
    parent_id TEXT,
    balance REAL NOT NULL DEFAULT 0,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES account_categories(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_account_categories_code ON account_categories(code);
CREATE INDEX IF NOT EXISTS idx_account_categories_type ON account_categories(category_type);
CREATE INDEX IF NOT EXISTS idx_account_categories_parent_id ON account_categories(parent_id);

-- ============================================
-- ACCOUNTING_ENTRIES - General ledger entries
-- ============================================
CREATE TABLE IF NOT EXISTS accounting_entries (
    id TEXT PRIMARY KEY,
    date TEXT NOT NULL,
    reference TEXT NOT NULL,
    description TEXT NOT NULL,
    debit_account TEXT NOT NULL,
    credit_account TEXT NOT NULL,
    amount REAL NOT NULL,
    entry_type TEXT NOT NULL DEFAULT 'manual' CHECK(entry_type IN ('manual', 'automatic', 'adjustment')),
    related_id TEXT,
    related_type TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT NOT NULL,
    FOREIGN KEY (debit_account) REFERENCES account_categories(id),
    FOREIGN KEY (credit_account) REFERENCES account_categories(id)
);

CREATE INDEX IF NOT EXISTS idx_accounting_entries_date ON accounting_entries(date);
CREATE INDEX IF NOT EXISTS idx_accounting_entries_entry_type ON accounting_entries(entry_type);
CREATE INDEX IF NOT EXISTS idx_accounting_entries_related_id ON accounting_entries(related_id);

-- ============================================
-- INVOICES - Invoice generation
-- ============================================
CREATE TABLE IF NOT EXISTS invoices (
    id TEXT PRIMARY KEY,
    series TEXT NOT NULL,
    number TEXT NOT NULL,
    client_name TEXT NOT NULL,
    client_ruc TEXT NOT NULL,
    client_address TEXT,
    emission_date TEXT NOT NULL,
    due_date TEXT NOT NULL,
    subtotal REAL NOT NULL DEFAULT 0,
    igv REAL NOT NULL DEFAULT 0,
    total REAL NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'paid', 'overdue', 'cancelled')),
    payment_method TEXT CHECK(payment_method IN ('cash', 'transfer', 'card')),
    paid_date TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT NOT NULL,
    UNIQUE(series, number)
);

CREATE INDEX IF NOT EXISTS idx_invoices_series_number ON invoices(series, number);
CREATE INDEX IF NOT EXISTS idx_invoices_client_ruc ON invoices(client_ruc);
CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);
CREATE INDEX IF NOT EXISTS idx_invoices_emission_date ON invoices(emission_date);

-- ============================================
-- INVOICE_LINES - Invoice details
-- ============================================
CREATE TABLE IF NOT EXISTS invoice_lines (
    id TEXT PRIMARY KEY,
    invoice_id TEXT NOT NULL,
    description TEXT NOT NULL,
    quantity REAL NOT NULL DEFAULT 1,
    unit_price REAL NOT NULL DEFAULT 0,
    total REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_invoice_lines_invoice_id ON invoice_lines(invoice_id);