// Employee Types - Phase 6
// Mirrors Rust DTOs from src-tauri/src/application/dto/employee.rs

// Document Types
export const DOCUMENT_TYPE = {
  DNI: "dni",
  RUC: "ruc",
  CE: "ce",
  PASSPORT: "passport",
} as const;

export type DocumentType = (typeof DOCUMENT_TYPE)[keyof typeof DOCUMENT_TYPE];

// Contract Types
export const CONTRACT_TYPE = {
  FULL_TIME: "full_time",
  PART_TIME: "part_time",
  TEMPORARY: "temporary",
  INTERNSHIP: "internship",
} as const;

export type ContractType = (typeof CONTRACT_TYPE)[keyof typeof CONTRACT_TYPE];

// Account Types (bank)
export const ACCOUNT_TYPE = {
  SAVINGS: "savings",
  CHECKING: "checking",
} as const;

export type AccountType = (typeof ACCOUNT_TYPE)[keyof typeof ACCOUNT_TYPE];

// AFP (Peruvian pension system)
export const AFP = {
  HABITAT: "habitat",
  INTEGRA: "integra",
  PRIMA: "prima",
  PROFUTURO: "profuturo",
} as const;

export type AFP = (typeof AFP)[keyof typeof AFP];

// Employee Status
export const EMPLOYEE_STATUS = {
  ACTIVE: "active",
  INACTIVE: "inactive",
  SUSPENDED: "suspended",
  TERMINATED: "terminated",
} as const;

export type EmployeeStatus = (typeof EMPLOYEE_STATUS)[keyof typeof EMPLOYEE_STATUS];

// Create Employee Request
export interface CreateEmployeeRequest {
  document_type: DocumentType;
  document_number: string;
  first_name: string;
  last_name: string;
  email: string;
  phone?: string;
  address?: string;
  position: string;
  department: string;
  contract_type: ContractType;
  base_salary: number;
  bank_name?: string;
  bank_account?: string;
  account_type?: AccountType;
  cci?: string;
  afp?: AFP;
}

// Update Employee Request
export interface UpdateEmployeeRequest {
  first_name?: string;
  last_name?: string;
  email?: string;
  phone?: string;
  address?: string;
  position?: string;
  department?: string;
  contract_type?: ContractType;
  base_salary?: number;
  bank_name?: string;
  bank_account?: string;
  account_type?: AccountType;
  cci?: string;
  afp?: AFP;
  status?: EmployeeStatus;
}

// Employee Response
export interface Employee {
  id: string;
  user_id?: string;
  document_type: DocumentType;
  document_number: string;
  first_name: string;
  last_name: string;
  full_name: string;
  email: string;
  phone?: string;
  address?: string;
  position: string;
  department: string;
  contract_type: ContractType;
  base_salary: number;
  bank_name?: string;
  bank_account?: string;
  account_type?: AccountType;
  cci?: string;
  afp?: AFP;
  hire_date: string;
  termination_date?: string;
  status: EmployeeStatus;
  created_at: string;
  updated_at: string;
}

// Employee List Item (for tables)
export interface EmployeeListItem {
  id: string;
  document_number: string;
  full_name: string;
  email: string;
  position: string;
  department: string;
  contract_type: ContractType;
  base_salary: number;
  status: EmployeeStatus;
}

// Department Summary
export interface DepartmentSummary {
  department: string;
  count: number;
  total_salary: number;
}

// Employee Summary
export interface EmployeeSummary {
  total_employees: number;
  active_employees: number;
  inactive_employees: number;
  total_salary_expense: number;
  by_department: DepartmentSummary[];
}