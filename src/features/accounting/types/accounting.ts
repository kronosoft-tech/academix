// Accounting Types - Simplified
// Mirrors simplified Rust DTOs

export const INCOME_CATEGORY = {
  TUITION: "tuition",
  OTHER: "other",
} as const;

export type IncomeCategory = (typeof INCOME_CATEGORY)[keyof typeof INCOME_CATEGORY];

export const EXPENSE_CATEGORY = {
  RENT: "rent",
  SALARIES: "salaries",
  UTILITIES: "utilities",
  OTHER: "other",
} as const;

export type ExpenseCategory = (typeof EXPENSE_CATEGORY)[keyof typeof EXPENSE_CATEGORY];

export type AccountingEntryType = "income" | "expense";

export interface AccountingEntry {
  id: string;
  date: string;
  entry_type: AccountingEntryType;
  category: string;
  description: string;
  amount: number;
  reference?: string;
  created_at: string;
}

export interface AccountingSummary {
  total_income: number;
  total_expenses: number;
  net_balance: number;
  entry_count: number;
  recent_entries: AccountingEntry[];
  monthly_data: MonthlyDataPoint[];
  expenses_by_category: ExpenseByCategory[];
  income_by_category: IncomeByCategory[];
}

export interface MonthlyDataPoint {
  month: string;
  income: number;
  expenses: number;
}

export interface ExpenseByCategory {
  category_name: string;
  amount: number;
}

export interface IncomeByCategory {
  category_name: string;
  amount: number;
}

export interface CreateEntryPayload {
  date: string;
  category: string;
  description: string;
  amount: number;
}

export interface EntryFilters {
  date_from?: string;
  date_to?: string;
  type?: AccountingEntryType;
}
