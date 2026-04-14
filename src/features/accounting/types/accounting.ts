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