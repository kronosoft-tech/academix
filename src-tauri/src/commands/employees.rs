//! Employee Commands - Tauri Command Handlers
//!
//! Expose employee operations to the frontend.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::employee::{
    CreateEmployeeRequest, EmployeeDto, EmployeeListItem, EmployeeSummary, UpdateEmployeeRequest,
};
use crate::application::use_cases::EmployeeService;
use crate::domain::entities::employee::{AccountType, ContractType, DocumentType, AFP};
use crate::infrastructure::repositories::SqliteEmployeeRepository;

/// Employee service state type
pub type EmployeeServiceState = EmployeeService<SqliteEmployeeRepository>;

/// Create employee request payload
#[derive(Debug, Deserialize)]
pub struct CreateEmployeeCommand {
    pub document_type: String,
    pub document_number: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub position: String,
    pub department: String,
    pub contract_type: String,
    pub base_salary: f64,
    pub bank_name: Option<String>,
    pub bank_account: Option<String>,
    pub account_type: Option<String>,
    pub cci: Option<String>,
    pub afp: Option<String>,
}

/// Update employee request payload
#[derive(Debug, Deserialize)]
pub struct UpdateEmployeeCommand {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub position: Option<String>,
    pub department: Option<String>,
    pub contract_type: Option<String>,
    pub base_salary: Option<f64>,
    pub bank_name: Option<String>,
    pub bank_account: Option<String>,
    pub account_type: Option<String>,
    pub cci: Option<String>,
    pub afp: Option<String>,
}

/// Employee response
#[derive(Debug, Serialize)]
pub struct EmployeeCommandResponse {
    pub success: bool,
    pub data: Option<EmployeeDto>,
    pub error: Option<String>,
}

/// Employee list response
#[derive(Debug, Serialize)]
pub struct EmployeeListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<EmployeeListItem>>,
    pub error: Option<String>,
}

/// Employee summary response
#[derive(Debug, Serialize)]
pub struct EmployeeSummaryCommandResponse {
    pub success: bool,
    pub data: Option<EmployeeSummary>,
    pub error: Option<String>,
}

/// Create a new employee
#[tauri::command]
pub fn create_employee(
    state: State<EmployeeServiceState>,
    request: CreateEmployeeCommand,
) -> EmployeeCommandResponse {
    let request = CreateEmployeeRequest {
        document_type: DocumentType::from_str(&request.document_type).unwrap_or(DocumentType::CC),
        document_number: request.document_number,
        first_name: request.first_name,
        last_name: request.last_name,
        email: request.email,
        phone: request.phone,
        address: request.address,
        position: request.position,
        department: request.department,
        contract_type: ContractType::from_str(&request.contract_type)
            .unwrap_or(ContractType::Indefinite),
        base_salary: request.base_salary,
        bank_name: request.bank_name,
        bank_account: request.bank_account,
        account_type: request.account_type.and_then(|s| AccountType::from_str(&s)),
        cci: request.cci,
        afp: request.afp.and_then(|s| AFP::from_str(&s)),
    };

    match state.create_employee(request) {
        Ok(employee) => EmployeeCommandResponse {
            success: true,
            data: Some(employee),
            error: None,
        },
        Err(e) => EmployeeCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}

/// Get employee by ID
#[tauri::command]
pub fn get_employee(state: State<EmployeeServiceState>, id: String) -> EmployeeCommandResponse {
    match state.get_employee(&id) {
        Ok(employee) => EmployeeCommandResponse {
            success: true,
            data: employee,
            error: None,
        },
        Err(e) => EmployeeCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}

/// List employees with optional filters
#[tauri::command]
pub fn list_employees(
    state: State<EmployeeServiceState>,
    status: Option<String>,
    department: Option<String>,
    search: Option<String>,
) -> EmployeeListCommandResponse {
    use crate::domain::entities::employee::EmployeeStatus;

    let status_filter = status.and_then(|s| EmployeeStatus::from_str(&s));
    let dept_filter = department.as_deref();
    let search_filter = search.as_deref();

    match state.list_employees(status_filter, dept_filter, search_filter) {
        Ok(employees) => EmployeeListCommandResponse {
            success: true,
            data: Some(employees),
            error: None,
        },
        Err(e) => EmployeeListCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}

/// Update employee
#[tauri::command]
pub fn update_employee(
    state: State<EmployeeServiceState>,
    id: String,
    request: UpdateEmployeeCommand,
) -> EmployeeCommandResponse {
    let request = UpdateEmployeeRequest {
        first_name: request.first_name,
        last_name: request.last_name,
        email: request.email,
        phone: request.phone,
        address: request.address,
        position: request.position,
        department: request.department,
        contract_type: request
            .contract_type
            .and_then(|s| ContractType::from_str(&s)),
        base_salary: request.base_salary,
        bank_name: request.bank_name,
        bank_account: request.bank_account,
        account_type: request.account_type.and_then(|s| AccountType::from_str(&s)),
        cci: request.cci,
        afp: request.afp.and_then(|s| AFP::from_str(&s)),
        status: None,
    };

    match state.update_employee(&id, request) {
        Ok(employee) => EmployeeCommandResponse {
            success: true,
            data: Some(employee),
            error: None,
        },
        Err(e) => EmployeeCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}

/// Delete (deactivate) employee
#[tauri::command]
pub fn delete_employee(state: State<EmployeeServiceState>, id: String) -> EmployeeCommandResponse {
    match state.delete_employee(&id) {
        Ok(_) => EmployeeCommandResponse {
            success: true,
            data: None,
            error: None,
        },
        Err(e) => EmployeeCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}

/// Get employee summary for dashboard
#[tauri::command]
pub fn get_employee_summary(state: State<EmployeeServiceState>) -> EmployeeSummaryCommandResponse {
    match state.get_summary() {
        Ok(summary) => EmployeeSummaryCommandResponse {
            success: true,
            data: Some(summary),
            error: None,
        },
        Err(e) => EmployeeSummaryCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}
