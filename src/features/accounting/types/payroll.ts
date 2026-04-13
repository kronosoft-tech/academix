// Payroll Types - Phase 6
// Mirrors Rust DTOs from src-tauri/src/application/dto/payroll.rs

// Payroll Run Status
export const PAYROLL_RUN_STATUS = {
  DRAFT: "draft",
  PROCESSING: "processing",
  COMPLETED: "completed",
  CANCELLED: "cancelled",
} as const;

export type PayrollRunStatus = (typeof PAYROLL_RUN_STATUS)[keyof typeof PAYROLL_RUN_STATUS];

// Payroll Entry Status
export const PAYROLL_ENTRY_STATUS = {
  PENDING: "pending",
  PROCESSED: "processed",
  PAID: "paid",
  FAILED: "failed",
} as const;

export type PayrollEntryStatus = (typeof PAYROLL_ENTRY_STATUS)[keyof typeof PAYROLL_ENTRY_STATUS];

// Run Payroll Request
export interface RunPayrollRequest {
  period_start: string;
  period_end: string;
  employee_ids: string[];
  created_by: string;
}

// Payroll Run
export interface PayrollRun {
  id: string;
  period_start: string;
  period_end: string;
  period_display: string;
  status: PayrollRunStatus;
  total_gross: number;
  total_deductions: number;
  total_net: number;
  employee_count: number;
  created_at: string;
  created_by: string;
}

// Payroll Entry
export interface PayrollEntry {
  id: string;
  payroll_run_id: string;
  employee_id: string;
  employee_name: string;
  // Income
  base_salary: number;
  hours_worked: number;
  overtime_hours: number;
  overtime_amount: number;
  bonuses: number;
  commissions: number;
  mobility: number;
  food: number;
  other_income: number;
  gross_income: number;
  // Deductions
  afp_deduction: number;
  onp_deduction: number;
  essalud: number;
  itf: number;
  other_deductions: number;
  total_deductions: number;
  // Net
  net_income: number;
  status: PayrollEntryStatus;
  created_at: string;
}

// Payroll Run with Entries
export interface PayrollRunWithEntries {
  run: PayrollRun;
  entries: PayrollEntry[];
}

// Payroll Summary
export interface PayrollSummary {
  total_payroll: number;
  total_gross: number;
  total_deductions: number;
  total_net: number;
  employee_count: number;
  latest_run?: PayrollRun;
}

// Entry filters
export interface PayrollFilters {
  period_start?: string;
  period_end?: string;
  status?: PayrollRunStatus;
  employee_id?: string;
}