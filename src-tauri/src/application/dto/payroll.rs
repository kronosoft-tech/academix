//! Payroll DTOs

use crate::domain::entities::payroll::{PayrollEntryStatus, PayrollRunStatus};
use serde::{Deserialize, Serialize};

/// Run payroll request
#[derive(Debug, Deserialize)]
pub struct RunPayrollRequest {
    pub period_start: String,
    pub period_end: String,
    pub employee_ids: Vec<String>,
    pub created_by: String,
}

/// Payroll run response
#[derive(Debug, Serialize)]
pub struct PayrollRunDto {
    pub id: String,
    pub period_start: String,
    pub period_end: String,
    pub period_display: String,
    pub status: PayrollRunStatus,
    pub total_gross: f64,
    pub total_deductions: f64,
    pub total_net: f64,
    pub employee_count: i32,
    pub created_at: String,
    pub created_by: String,
}

impl From<crate::domain::entities::payroll::PayrollRun> for PayrollRunDto {
    fn from(run: crate::domain::entities::payroll::PayrollRun) -> Self {
        let period = run.period_display();
        Self {
            id: run.id,
            period_start: run.period_start.to_rfc3339(),
            period_end: run.period_end.to_rfc3339(),
            period_display: period,
            status: run.status,
            total_gross: run.total_gross,
            total_deductions: run.total_deductions,
            total_net: run.total_net,
            employee_count: 0, // Will be filled by service
            created_at: run.created_at.to_rfc3339(),
            created_by: run.created_by,
        }
    }
}

/// Payroll entry response
#[derive(Debug, Serialize)]
pub struct PayrollEntryDto {
    pub id: String,
    pub payroll_run_id: String,
    pub employee_id: String,
    pub employee_name: String,
    // Income
    pub base_salary: f64,
    pub hours_worked: f64,
    pub overtime_hours: f64,
    pub overtime_amount: f64,
    pub bonuses: f64,
    pub commissions: f64,
    pub mobility: f64,
    pub food: f64,
    pub other_income: f64,
    pub gross_income: f64,
    // Deductions
    pub afp_deduction: f64,
    pub onp_deduction: f64,
    pub essalud: f64,
    pub itf: f64,
    pub other_deductions: f64,
    pub total_deductions: f64,
    // Net
    pub net_income: f64,
    pub status: PayrollEntryStatus,
    pub created_at: String,
}

impl From<crate::domain::entities::payroll::PayrollEntry> for PayrollEntryDto {
    fn from(entry: crate::domain::entities::payroll::PayrollEntry) -> Self {
        Self {
            id: entry.id,
            payroll_run_id: entry.payroll_run_id,
            employee_id: entry.employee_id,
            employee_name: String::new(), // Will be filled by service
            base_salary: entry.base_salary,
            hours_worked: entry.hours_worked,
            overtime_hours: entry.overtime_hours,
            overtime_amount: entry.overtime_amount,
            bonuses: entry.bonuses,
            commissions: entry.commissions,
            mobility: entry.mobility,
            food: entry.food,
            other_income: entry.other_income,
            gross_income: entry.gross_income,
            afp_deduction: entry.afp_deduction,
            onp_deduction: entry.onp_deduction,
            essalud: entry.essalud,
            itf: entry.itf,
            other_deductions: entry.other_deductions,
            total_deductions: entry.afp_deduction
                + entry.onp_deduction
                + entry.essalud
                + entry.itf
                + entry.other_deductions,
            net_income: entry.net_income,
            status: entry.status,
            created_at: entry.created_at.to_rfc3339(),
        }
    }
}

/// Payroll run with entries
#[derive(Debug, Serialize)]
pub struct PayrollRunWithEntriesDto {
    pub run: PayrollRunDto,
    pub entries: Vec<PayrollEntryDto>,
}

/// Payroll summary for dashboard
#[derive(Debug, Serialize)]
pub struct PayrollSummary {
    pub total_payroll: f64,
    pub total_gross: f64,
    pub total_deductions: f64,
    pub total_net: f64,
    pub employee_count: i32,
    pub latest_run: Option<PayrollRunDto>,
}
