//! Payroll Commands - Tauri Command Handlers
//!
//! Expose payroll operations to the frontend.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::payroll::{
    PayrollRunDto, PayrollRunWithEntriesDto, PayrollSummary, RunPayrollRequest,
};
use crate::application::use_cases::PayrollService;
use crate::infrastructure::repositories::{
    SqliteEmployeeRepository, SqlitePayrollEntryRepository, SqlitePayrollRepository,
};

/// Payroll service state type
pub type PayrollServiceState =
    PayrollService<SqlitePayrollRepository, SqlitePayrollEntryRepository, SqliteEmployeeRepository>;

/// Run payroll request
#[derive(Debug, Deserialize)]
pub struct RunPayrollCommand {
    pub period_start: String,
    pub period_end: String,
    pub employee_ids: Vec<String>,
    pub created_by: String,
}

/// Payroll response
#[derive(Debug, Serialize)]
pub struct PayrollCommandResponse {
    pub success: bool,
    pub data: Option<PayrollRunWithEntriesDto>,
    pub error: Option<String>,
}

/// Payroll list response
#[derive(Debug, Serialize)]
pub struct PayrollListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<PayrollRunDto>>,
    pub error: Option<String>,
}

/// Payroll summary response
#[derive(Debug, Serialize)]
pub struct PayrollSummaryCommandResponse {
    pub success: bool,
    pub data: Option<PayrollSummary>,
    pub error: Option<String>,
}

/// Run payroll for a period
#[tauri::command]
pub fn run_payroll(
    state: State<PayrollServiceState>,
    request: RunPayrollCommand,
) -> PayrollCommandResponse {
    let request = RunPayrollRequest {
        period_start: request.period_start,
        period_end: request.period_end,
        employee_ids: request.employee_ids,
        created_by: request.created_by,
    };

    match state.run_payroll(request) {
        Ok(result) => PayrollCommandResponse {
            success: true,
            data: Some(result),
            error: None,
        },
        Err(e) => PayrollCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}

/// Get payroll run by ID with entries
#[tauri::command]
pub fn get_payroll_run(state: State<PayrollServiceState>, id: String) -> PayrollCommandResponse {
    match state.get_payroll_run(&id) {
        Ok(run) => PayrollCommandResponse {
            success: true,
            data: run,
            error: None,
        },
        Err(e) => PayrollCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}

/// List all payroll runs
#[tauri::command]
pub fn list_payroll_runs(
    state: State<PayrollServiceState>,
    period_start: Option<String>,
    period_end: Option<String>,
    status: Option<String>,
) -> PayrollListCommandResponse {
    use crate::domain::entities::payroll::PayrollRunStatus;

    let status_filter = status.and_then(|s| PayrollRunStatus::from_str(&s));

    match state.list_payroll_runs(status_filter, period_start.as_deref(), period_end.as_deref()) {
        Ok(runs) => PayrollListCommandResponse {
            success: true,
            data: Some(runs),
            error: None,
        },
        Err(e) => PayrollListCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}

/// Get payroll summary for dashboard
#[tauri::command]
pub fn get_payroll_summary(state: State<PayrollServiceState>) -> PayrollSummaryCommandResponse {
    match state.get_summary() {
        Ok(summary) => PayrollSummaryCommandResponse {
            success: true,
            data: Some(summary),
            error: None,
        },
        Err(e) => PayrollSummaryCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}
