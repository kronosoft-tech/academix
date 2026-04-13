//! Employee Commands - Tauri Command Handlers
//!
//! Expose employee operations to the frontend.

use crate::application::dto::employee::{
    CreateEmployeeRequest, EmployeeDto, EmployeeListItem, EmployeeSummary, UpdateEmployeeRequest,
};
use crate::application::use_cases::EmployeeService;
use crate::infrastructure::repositories::InMemoryEmployeeRepository;
use tauri::command;

/// Create a new employee
#[command]
pub fn create_employee(request: CreateEmployeeRequest) -> Result<EmployeeDto, String> {
    let repo = InMemoryEmployeeRepository::new();
    let service = EmployeeService::new(repo);
    service.create_employee(request)
}

/// Get employee by ID
#[command]
pub fn get_employee(id: String) -> Result<Option<EmployeeDto>, String> {
    let repo = InMemoryEmployeeRepository::new();
    let service = EmployeeService::new(repo);
    service.get_employee(&id)
}

/// List employees with optional filters
#[command]
pub fn list_employees(
    status: Option<String>,
    department: Option<String>,
) -> Result<Vec<EmployeeListItem>, String> {
    use crate::domain::entities::employee::EmployeeStatus;

    let status_filter = status.and_then(|s| EmployeeStatus::from_str(&s));
    let dept_filter = department.as_deref();

    let repo = InMemoryEmployeeRepository::new();
    let service = EmployeeService::new(repo);
    service.list_employees(status_filter, dept_filter)
}

/// Update employee
#[command]
pub fn update_employee(id: String, request: UpdateEmployeeRequest) -> Result<EmployeeDto, String> {
    let repo = InMemoryEmployeeRepository::new();
    let service = EmployeeService::new(repo);
    service.update_employee(&id, request)
}

/// Delete (deactivate) employee
#[command]
pub fn delete_employee(id: String) -> Result<bool, String> {
    let repo = InMemoryEmployeeRepository::new();
    let service = EmployeeService::new(repo);
    service.delete_employee(&id)
}

/// Get employee summary for dashboard
#[command]
pub fn get_employee_summary() -> Result<EmployeeSummary, String> {
    let repo = InMemoryEmployeeRepository::new();
    let service = EmployeeService::new(repo);
    service.get_summary()
}
