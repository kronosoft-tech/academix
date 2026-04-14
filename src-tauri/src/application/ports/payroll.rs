//! Payroll Repository Ports
//!
//! Port interfaces for payroll operations.

use crate::domain::entities::payroll::{PayrollEntry, PayrollRun, PayrollRunStatus};

/// Payroll run repository trait (port)
pub trait PayrollRepository: Send + Sync {
    /// Create a new payroll run
    fn create(&self, run: PayrollRun) -> Result<PayrollRun, String>;

    /// Get payroll run by ID
    fn get_by_id(&self, id: &str) -> Result<Option<PayrollRun>, String>;

    /// Get payroll run by period
    fn get_by_period(
        &self,
        period_start: &str,
        period_end: &str,
    ) -> Result<Option<PayrollRun>, String>;

    /// List all payroll runs
    fn list(&self, status: Option<PayrollRunStatus>) -> Result<Vec<PayrollRun>, String>;

    /// Update payroll run
    fn update(&self, run: PayrollRun) -> Result<PayrollRun, String>;

    /// Delete payroll run (soft delete)
    fn delete(&self, id: &str) -> Result<bool, String>;

    /// Get latest payroll run
    fn get_latest(&self) -> Result<Option<PayrollRun>, String>;
}

/// Payroll entry repository trait (port)
pub trait PayrollEntryRepository: Send + Sync {
    /// Create a new payroll entry
    fn create(&self, entry: PayrollEntry) -> Result<PayrollEntry, String>;

    /// Get payroll entry by ID
    fn get_by_id(&self, id: &str) -> Result<Option<PayrollEntry>, String>;

    /// Get payroll entries by payroll run ID
    fn get_by_payroll_run(&self, payroll_run_id: &str) -> Result<Vec<PayrollEntry>, String>;

    /// Get payroll entry by employee and payroll run
    fn get_by_employee_and_run(
        &self,
        employee_id: &str,
        payroll_run_id: &str,
    ) -> Result<Option<PayrollEntry>, String>;

    /// List payroll entries by employee
    fn list_by_employee(&self, employee_id: &str) -> Result<Vec<PayrollEntry>, String>;

    /// Update payroll entry
    fn update(&self, entry: PayrollEntry) -> Result<PayrollEntry, String>;

    /// Update multiple payroll entries
    fn update_many(&self, entries: &[PayrollEntry]) -> Result<(), String>;

    /// Delete payroll entries by payroll run ID
    fn delete_by_payroll_run(&self, payroll_run_id: &str) -> Result<bool, String>;

    /// Get total net pay for a payroll run
    fn get_total_net_by_run(&self, payroll_run_id: &str) -> Result<f64, String>;

    /// Get total gross by run
    fn get_total_gross_by_run(&self, payroll_run_id: &str) -> Result<f64, String>;

    /// Get total deductions by run
    fn get_total_deductions_by_run(&self, payroll_run_id: &str) -> Result<f64, String>;
}
