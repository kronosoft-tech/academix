// Accounting Types - Phase 6
// Mirrors Rust DTOs from src-tauri/src/application/dto/accounting.rs

// Entry Types
export const ENTRY_TYPE = {
  MANUAL: "manual",
  AUTOMATIC: "automatic",
  ADJUSTMENT: "adjustment",
} as const;

export type EntryType = (typeof ENTRY_TYPE)[keyof typeof ENTRY_TYPE];

// Category Types (Peruvian accounting)
export const CATEGORY_TYPE = {
  ASSET: "asset",
  LIABILITY: "liability",
  EQUITY: "equity",
  EXPENSE: "expense",
  COST: "cost",
  INCOME: "income",
} as const;

export type CategoryType = (typeof CATEGORY_TYPE)[keyof typeof CATEGORY_TYPE];

// Liability (Pasivo) Types
export const LIABILITY_TYPE = {
  SHORT_TERM: "short_term",     // Corto plazo (< 1 año)
  LONG_TERM: "long_term",       // Largo plazo (> 1 año)
  PROVISIONS: "provisions",     // Provisiones
} as const;

export type LiabilityType = (typeof LIABILITY_TYPE)[keyof typeof LIABILITY_TYPE];

// Equity (Patrimonio) Types  
export const EQUITY_TYPE = {
  CAPITAL: "capital",           // Capital social
  RESERVES: "reserves",         // Reservas
  RESULTS: "results",           // Resultados
  RETAINED: "retained",        // Resultados acumulados
} as const;

export type EquityType = (typeof EQUITY_TYPE)[keyof typeof EQUITY_TYPE];

// Create Entry Request
export interface CreateEntryRequest {
  date: string;
  description: string;
  debit_account: string;
  credit_account: string;
  amount: number;
  entry_type?: EntryType;
  reference?: string;
  related_id?: string;
  related_type?: string;
}

// Accounting Entry
export interface AccountingEntry {
  id: string;
  date: string;
  reference: string;
  description: string;
  debit_account: string;
  debit_account_name: string;
  credit_account: string;
  credit_account_name: string;
  amount: number;
  entry_type: EntryType;
  related_id?: string;
  related_type?: string;
  created_at: string;
  created_by: string;
}

// Account Category
export interface AccountCategory {
  id: string;
  code: string;
  display_code: string;
  name: string;
  category_type: CategoryType;
  parent_id?: string;
  balance: number;
  active: boolean;
  created_at: string;
  updated_at: string;
}

// Account Category Tree Node
export interface AccountCategoryTreeNode {
  category: AccountCategory;
  children: AccountCategoryTreeNode[];
}

// Trial Balance
export interface TrialBalanceAccount {
  account_id: string;
  account_code: string;
  account_name: string;
  debit_balance: number;
  credit_balance: number;
  balance_type: string;
}

export interface TrialBalance {
  as_of_date: string;
  accounts: TrialBalanceAccount[];
  total_debits: number;
  total_credits: number;
  is_balanced: boolean;
}

// Category Total (for income statement)
export interface CategoryTotal {
  category_id: string;
  category_name: string;
  total: number;
}

// Income Statement
export interface IncomeStatement {
  period_start: string;
  period_end: string;
  total_income: number;
  total_expenses: number;
  total_costs: number;
  net_result: number;
  is_profitable: boolean;
  income_by_category: CategoryTotal[];
  expenses_by_category: CategoryTotal[];
}

// Monthly data for charts
export interface MonthlyDataPoint {
  month: string;
  income: number;
  expenses: number;
}

// Expense breakdown by category
export interface ExpenseByCategory {
  category_name: string;
  amount: number;
}

// Income breakdown by category
export interface IncomeByCategory {
  category_name: string;
  amount: number;
}

// Accounting Summary
export interface AccountingSummary {
  total_income: number;
  total_expenses: number;
  net_balance: number;
  account_count: number;
  entry_count: number;
  recent_entries: AccountingEntry[];
  monthly_data: MonthlyDataPoint[];
  expenses_by_category: ExpenseByCategory[];
  income_by_category: IncomeByCategory[];
}

// Financial Balance (Balance Financiero)
export interface FinancialBalance {
  as_of_date: string;
  assets: AccountBalance[];
  liabilities: AccountBalance[];
  equity: AccountBalance[];
  total_assets: number;
  total_liabilities: number;
  total_equity: number;
  is_balanced: boolean;
}

// Account balance for financial balance report
export interface AccountBalance {
  account_code: string;
  account_name: string;
  balance: number;
}

// Filter options for entries
export interface EntryFilters {
  date_from?: string;
  date_to?: string;
  entry_type?: EntryType;
}

// Account filters
export interface AccountFilters {
  category_type?: CategoryType;
  active_only: boolean;
}

// Liability / Pasivo (new)
export interface CreateLiabilityRequest {
  provider_name: string;
  document_type: string;  // factura, recibo, contrato
  document_number: string;
  amount: number;
  liability_type: LiabilityType;
  due_date: string;
  description?: string;
  /// What is this liability for?
  /// "expense" for services/supplies (goes to expense account 4xxx)
  /// "asset" for equipment/purchases (goes to fixed asset account 16xx)
  for_type?: "expense" | "asset";
  /// Account to debit - expense (4xxx) or asset (16xxx)
  debit_account_code?: string;
}

export interface Liability {
  id: string;
  provider_name: string;
  document_type: string;
  document_number: string;
  amount: number;
  paid_amount: number;
  liability_type: LiabilityType;
  due_date: string;
  status: "pending" | "partial" | "paid" | "overdue";
  description?: string;
  account_code?: string;  // Added: links to chart of accounts (2xxx)
  created_at: string;
  updated_at: string;
}

// Equity / Patrimonio (new) - includes account_code for Balance Financiero
export interface CreateEquityRequest {
  equity_type: EquityType;
  description: string;
  amount: number;
  /// Account code where the money/asset went (e.g., "1105" for caja, "1110" for bancos)
  /// Required when registering capital to keep balance balanced
  asset_account_code?: string;
}

export interface Equity {
  id: string;
  equity_type: EquityType;
  description: string;
  amount: number;
  account_code?: string;  // Added: links to chart of accounts (3xxx)
  created_at: string;
  updated_at: string;
}

// Fixed Asset Types (Activos Fijos)
export interface CreateFixedAssetRequest {
  name: string;
  asset_type: string;
  description?: string;
  acquisition_date: string;
  acquisition_cost: number;
  useful_life_years: number;
  account_code?: string;   // 15xx
  payment_account_code?: string; // 1105 (caja), 1110 (bancos)
}

export interface FixedAsset {
  id: string;
  name: string;
  asset_type: string;
  description?: string;
  acquisition_date: string;
  acquisition_cost: number;
  current_value: number;
  useful_life_years: number;
  account_code: string;
  status: string;
  created_at: string;
  updated_at: string;
}
