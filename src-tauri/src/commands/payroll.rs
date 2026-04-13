//! Payroll Commands - Tauri Command Handlers
//!
//! Expose payroll operations to the frontend.

use crate::application::dto::payroll::{
    PayrollEntryDto, PayrollRunDto, PayrollRunWithEntriesDto, PayrollSummary, RunPayrollRequest,
};
use crate::application::use_cases::PayrollService;
use crate::infrastructure::repositories::{
    InMemoryEmployeeRepository, InMemoryPayrollEntryRepository, InMemoryPayrollRepository,
};
use tauri::command;

/// Run payroll for a period
#[command]
pub fn run_payroll(request: RunPayrollRequest) -> Result<PayrollRunWithEntriesDto, String> {
    let payroll_repo = InMemoryPayrollRepository::new();
    let entry_repo = InMemoryPayrollEntryRepository::new();
    let employee_repo = InMemoryEmployeeRepository::new();

    let service = PayrollService::new(payroll_repo, entry_repo, employee_repo);
    service.run_payroll(request)
}

/// Get payroll run by ID with entries
#[command]
pub fn get_payroll_run(id: String) -> Result<Option<PayrollRunWithEntriesDto>, String> {
    let payroll_repo = InMemoryPayrollRepository::new();
    let entry_repo = InMemoryPayrollEntryRepository::new();
    let employee_repo = InMemoryEmployeeRepository::new();

    let service = PayrollService::new(payroll_repo, entry_repo, employee_repo);
    service.get_payroll_run(&id)
}

/// List all payroll runs
#[command]
pub fn list_payroll_runs(status: Option<String>) -> Result<Vec<PayrollRunDto>, String> {
    use crate::domain::entities::payroll::PayrollRunStatus;

    let status_filter = status.and_then(|s| PayrollRunStatus::from_str(&s));

    let payroll_repo = InMemoryPayrollRepository::new();
    let entry_repo = InMemoryPayrollEntryRepository::new();
    let employee_repo = InMemoryEmployeeRepository::new();

    let service = PayrollService::new(payroll_repo, entry_repo, employee_repo);
    service.list_payroll_runs(status_filter)
}

/// Get payroll summary for dashboard
#[command]
pub fn get_payroll_summary() -> Result<PayrollSummary, String> {
    let payroll_repo = InMemoryPayrollRepository::new();
    let entry_repo = InMemoryPayrollEntryRepository::new();
    let employee_repo = InMemoryEmployeeRepository::new();

    let service = PayrollService::new(payroll_repo, entry_repo, employee_repo);
    service.get_summary()
}
