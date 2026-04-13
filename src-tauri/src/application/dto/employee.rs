//! Employee DTOs

use crate::domain::entities::employee::{
    AccountType, ContractType, DocumentType, EmployeeStatus, AFP,
};
use serde::{Deserialize, Serialize};

/// Create employee request
#[derive(Debug, Deserialize)]
pub struct CreateEmployeeRequest {
    pub document_type: DocumentType,
    pub document_number: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub position: String,
    pub department: String,
    pub contract_type: ContractType,
    pub base_salary: f64,
    pub bank_name: Option<String>,
    pub bank_account: Option<String>,
    pub account_type: Option<AccountType>,
    pub cci: Option<String>,
    pub afp: Option<AFP>,
}

/// Update employee request
#[derive(Debug, Deserialize)]
pub struct UpdateEmployeeRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub position: Option<String>,
    pub department: Option<String>,
    pub contract_type: Option<ContractType>,
    pub base_salary: Option<f64>,
    pub bank_name: Option<String>,
    pub bank_account: Option<String>,
    pub account_type: Option<AccountType>,
    pub cci: Option<String>,
    pub afp: Option<AFP>,
    pub status: Option<EmployeeStatus>,
}

/// Employee response
#[derive(Debug, Serialize)]
pub struct EmployeeDto {
    pub id: String,
    pub user_id: Option<String>,
    pub document_type: DocumentType,
    pub document_number: String,
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub position: String,
    pub department: String,
    pub contract_type: ContractType,
    pub base_salary: f64,
    pub bank_name: Option<String>,
    pub bank_account: Option<String>,
    pub account_type: Option<AccountType>,
    pub cci: Option<String>,
    pub afp: Option<AFP>,
    pub hire_date: String,
    pub termination_date: Option<String>,
    pub status: EmployeeStatus,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::domain::entities::employee::Employee> for EmployeeDto {
    fn from(emp: crate::domain::entities::employee::Employee) -> Self {
        let full = emp.full_name();
        Self {
            id: emp.id,
            user_id: emp.user_id,
            document_type: emp.document_type,
            document_number: emp.document_number,
            first_name: emp.first_name,
            last_name: emp.last_name,
            full_name: full,
            email: emp.email,
            phone: emp.phone,
            address: emp.address,
            position: emp.position,
            department: emp.department,
            contract_type: emp.contract_type,
            base_salary: emp.base_salary,
            bank_name: emp.bank_name,
            bank_account: emp.bank_account,
            account_type: emp.account_type,
            cci: emp.cci,
            afp: emp.afp,
            hire_date: emp.hire_date.to_rfc3339(),
            termination_date: emp.termination_date.map(|d| d.to_rfc3339()),
            status: emp.status,
            created_at: emp.created_at.to_rfc3339(),
            updated_at: emp.updated_at.to_rfc3339(),
        }
    }
}

/// Employee list item (for tables)
#[derive(Debug, Serialize)]
pub struct EmployeeListItem {
    pub id: String,
    pub document_number: String,
    pub full_name: String,
    pub email: String,
    pub position: String,
    pub department: String,
    pub contract_type: ContractType,
    pub base_salary: f64,
    pub status: EmployeeStatus,
}

impl From<crate::domain::entities::employee::Employee> for EmployeeListItem {
    fn from(emp: crate::domain::entities::employee::Employee) -> Self {
        let full = emp.full_name();
        Self {
            id: emp.id,
            document_number: emp.document_number,
            full_name: full,
            email: emp.email,
            position: emp.position,
            department: emp.department,
            contract_type: emp.contract_type,
            base_salary: emp.base_salary,
            status: emp.status,
        }
    }
}

/// Employee summary for dashboard
#[derive(Debug, Serialize)]
pub struct EmployeeSummary {
    pub total_employees: i64,
    pub active_employees: i64,
    pub inactive_employees: i64,
    pub total_salary_expense: f64,
    pub by_department: Vec<DepartmentSummary>,
}

#[derive(Debug, Serialize)]
pub struct DepartmentSummary {
    pub department: String,
    pub count: i64,
    pub total_salary: f64,
}
