//! Payroll Service
//!
//! Use case for payroll operations with Peruvian legal requirements.

use crate::application::dto::payroll::{
    PayrollEntryDto, PayrollRunDto, PayrollRunWithEntriesDto, PayrollSummary, RunPayrollRequest,
};
use crate::application::ports::employee::EmployeeRepository;
use crate::application::ports::payroll::{PayrollEntryRepository, PayrollRepository};
use crate::domain::entities::employee::Employee;
use crate::domain::entities::payroll::{PayrollEntry, PayrollRun, PayrollRunStatus};
use chrono::{DateTime, Utc};

/// Payroll service - orchestrates payroll operations
pub struct PayrollService<R: PayrollRepository, E: PayrollEntryRepository, Emp: EmployeeRepository>
{
    payroll_repo: R,
    entry_repo: E,
    employee_repo: Emp,
}

impl<R: PayrollRepository, E: PayrollEntryRepository, Emp: EmployeeRepository>
    PayrollService<R, E, Emp>
{
    pub fn new(payroll_repo: R, entry_repo: E, employee_repo: Emp) -> Self {
        Self {
            payroll_repo,
            entry_repo,
            employee_repo,
        }
    }

    /// Run payroll for a period
    pub fn run_payroll(
        &self,
        request: RunPayrollRequest,
    ) -> Result<PayrollRunWithEntriesDto, String> {
        // Parse dates
        let period_start = DateTime::parse_from_rfc3339(&request.period_start)
            .map_err(|e| format!("Invalid period_start: {}", e))?
            .with_timezone(&Utc);
        let period_end = DateTime::parse_from_rfc3339(&request.period_end)
            .map_err(|e| format!("Invalid period_end: {}", e))?
            .with_timezone(&Utc);

        // Get employees
        let employees = self.employee_repo.list_by_ids(&request.employee_ids)?;
        if employees.is_empty() {
            return Err("No employees selected for payroll".to_string());
        }

        // Create payroll run
        let mut run = PayrollRun::new(String::new(), period_start, period_end, request.created_by);

        // Calculate entries for each employee
        let mut entries: Vec<PayrollEntry> = Vec::new();
        let mut total_gross = 0.0;
        let mut total_deductions = 0.0;

        for employee in &employees {
            let entry = self.calculate_payroll_entry(employee);
            total_gross += entry.gross_income;
            total_deductions += entry.afp_deduction
                + entry.onp_deduction
                + entry.essalud
                + entry.itf
                + entry.other_deductions;
            entries.push(entry);
        }

        let total_net = total_gross - total_deductions;
        run.set_totals(total_gross, total_deductions, total_net);
        run.mark_calculated();

        // Save payroll run
        let run = self.payroll_repo.create(run)?;

        // Update entries with run ID and save
        let mut entry_dtos = Vec::new();
        for mut entry in entries {
            entry.payroll_run_id = run.id.clone();
            let saved_entry = self.entry_repo.create(entry)?;

            // Get employee name for DTO
            let emp_name = employees
                .iter()
                .find(|e| e.id == saved_entry.employee_id)
                .map(|e| e.full_name())
                .unwrap_or_default();

            let mut dto = PayrollEntryDto::from(saved_entry);
            dto.employee_name = emp_name;
            entry_dtos.push(dto);
        }

        let mut run_dto = PayrollRunDto::from(run);
        run_dto.employee_count = entry_dtos.len() as i32;

        Ok(PayrollRunWithEntriesDto {
            run: run_dto,
            entries: entry_dtos,
        })
    }

    /// Calculate payroll entry for a single employee
    fn calculate_payroll_entry(&self, employee: &Employee) -> PayrollEntry {
        let mut entry = PayrollEntry::new(
            String::new(),
            String::new(), // Will be set after run is created
            employee.id.clone(),
            employee.base_salary,
        );

        // Calculate gross income
        entry.calculate_gross();

        // Calculate deductions based on Peruvian law
        // AFP: 11.25% to 11.60% (varies by fund)
        // For simplicity, using 11.25% as default
        if let Some(afp) = &employee.afp {
            entry.afp_deduction = employee.base_salary * afp.rate();
        } else {
            // If no AFP, calculate ONP (13%)
            entry.onp_deduction = employee.base_salary * 0.13;
        }

        // Essalud: 9% (employer contribution, shown in payslip for reference)
        entry.essalud = employee.base_salary * 0.09;

        // ITF: 0.5% on bank deposits > S/3500
        // Simplified: apply to gross income if > 3500
        if employee.base_salary > 3500.0 {
            entry.itf = (employee.base_salary - 3500.0) * 0.005;
        }

        // Calculate net
        entry.calculate_net();

        entry
    }

    /// Get payroll run by ID with entries
    pub fn get_payroll_run(&self, id: &str) -> Result<Option<PayrollRunWithEntriesDto>, String> {
        let run = self.payroll_repo.get_by_id(id)?;

        if let Some(run) = run {
            let entries = self.entry_repo.get_by_payroll_run(&run.id)?;

            // Get employee names
            let employee_ids: Vec<String> = entries.iter().map(|e| e.employee_id.clone()).collect();
            let employees = self.employee_repo.list_by_ids(&employee_ids)?;

            let entry_dtos: Vec<PayrollEntryDto> = entries
                .into_iter()
                .map(|e| {
                    let emp_name = employees
                        .iter()
                        .find(|emp| emp.id == e.employee_id)
                        .map(|emp| emp.full_name())
                        .unwrap_or_default();
                    let mut dto = PayrollEntryDto::from(e);
                    dto.employee_name = emp_name;
                    dto
                })
                .collect();

            let mut run_dto = PayrollRunDto::from(run);
            run_dto.employee_count = entry_dtos.len() as i32;

            Ok(Some(PayrollRunWithEntriesDto {
                run: run_dto,
                entries: entry_dtos,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all payroll runs
    pub fn list_payroll_runs(
        &self,
        status: Option<PayrollRunStatus>,
        period_start: Option<&str>,
        period_end: Option<&str>,
    ) -> Result<Vec<PayrollRunDto>, String> {
        let runs = self.payroll_repo.list(status)?;
        let runs_dto: Vec<PayrollRunDto> = runs.into_iter().map(PayrollRunDto::from).collect();

        let filtered: Vec<PayrollRunDto> = if period_start.is_some() || period_end.is_some() {
            runs_dto
                .into_iter()
                .filter(|r| {
                    let matches_start = period_start
                        .map(|start| r.period_start.as_str() >= start)
                        .unwrap_or(true);
                    let matches_end = period_end
                        .map(|end| r.period_end.as_str() <= end)
                        .unwrap_or(true);
                    matches_start && matches_end
                })
                .collect()
        } else {
            runs_dto
        };

        Ok(filtered)
    }

    /// Get payroll summary for dashboard
    pub fn get_summary(&self) -> Result<PayrollSummary, String> {
        let runs = self.payroll_repo.list(None)?;

        let mut total_net = 0.0;
        let mut total_gross = 0.0;
        let mut total_deductions = 0.0;
        let mut employee_count = 0;

        for run in &runs {
            total_net += run.total_net;
            total_gross += run.total_gross;
            total_deductions += run.total_deductions;
            employee_count += self.entry_repo.get_by_payroll_run(&run.id)?.len() as i32;
        }

        let latest = self.payroll_repo.get_latest()?;

        Ok(PayrollSummary {
            total_payroll: total_net,
            total_gross,
            total_deductions,
            total_net,
            employee_count,
            latest_run: latest.map(PayrollRunDto::from),
        })
    }
}

/// AFP rates for Peru (employee contribution rates)
pub mod afp_rates {
    /// AFP Prima - 11.25%
    pub const PRIMA_RATE: f64 = 0.1125;
    /// AFP Habitat - 11.35%
    pub const HABITAT_RATE: f64 = 0.1135;
    /// AFP Integra - 11.45%
    pub const INTEGRA_RATE: f64 = 0.1145;
    /// AFP Profuturo - 11.60%
    pub const PROFUTURO_RATE: f64 = 0.1160;
    /// ONP rate (public pension) - 13%
    pub const ONP_RATE: f64 = 0.13;
}

/// Calculate AFP deduction based on fund
pub fn calculate_afp_deduction(base_salary: f64, fund: &str) -> f64 {
    match fund.to_lowercase().as_str() {
        "prima" => base_salary * afp_rates::PRIMA_RATE,
        "habitat" => base_salary * afp_rates::HABITAT_RATE,
        "integra" => base_salary * afp_rates::INTEGRA_RATE,
        "profuturo" => base_salary * afp_rates::PROFUTURO_RATE,
        _ => base_salary * afp_rates::PRIMA_RATE, // Default
    }
}

/// Calculate ONP deduction
pub fn calculate_onp_deduction(base_salary: f64) -> f64 {
    base_salary * afp_rates::ONP_RATE
}

/// Calculate Essalud (9% employer contribution, shown in payslip)
pub fn calculate_essalud(base_salary: f64) -> f64 {
    base_salary * 0.09
}

/// Calculate ITF (0.5% on deposits > S/3500)
pub fn calculate_itf(monthly_total: f64) -> f64 {
    if monthly_total > 3500.0 {
        (monthly_total - 3500.0) * 0.005
    } else {
        0.0
    }
}
