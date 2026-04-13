//! Payroll Repository Ports (Trait Definitions)
//!
//! In-memory repository implementations for payroll.

use crate::application::ports::payroll::{PayrollEntryRepository, PayrollRepository};
use crate::domain::entities::payroll::{
    PayrollEntry, PayrollEntryStatus, PayrollRun, PayrollRunStatus,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory implementation of PayrollRepository
pub struct InMemoryPayrollRepository {
    runs: Arc<RwLock<HashMap<String, PayrollRun>>>,
    next_id: Arc<RwLock<u32>>,
}

impl InMemoryPayrollRepository {
    pub fn new() -> Self {
        Self {
            runs: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    fn generate_id(&self) -> String {
        let mut counter = self.next_id.write().unwrap();
        let id = format!("payrun-{:03}", *counter);
        *counter += 1;
        id
    }
}

impl PayrollRepository for InMemoryPayrollRepository {
    fn create(&self, mut run: PayrollRun) -> Result<PayrollRun, String> {
        if run.id.is_empty() {
            run.id = self.generate_id();
        }
        let mut runs = self.runs.write().unwrap();
        runs.insert(run.id.clone(), run.clone());
        Ok(run)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<PayrollRun>, String> {
        let runs = self.runs.read().unwrap();
        Ok(runs.get(id).cloned())
    }

    fn get_by_period(
        &self,
        period_start: &str,
        period_end: &str,
    ) -> Result<Option<PayrollRun>, String> {
        let runs = self.runs.read().unwrap();
        Ok(runs
            .values()
            .find(|r| {
                r.period_start.to_rfc3339() == period_start
                    && r.period_end.to_rfc3339() == period_end
            })
            .cloned())
    }

    fn list(&self, status: Option<PayrollRunStatus>) -> Result<Vec<PayrollRun>, String> {
        let runs = self.runs.read().unwrap();
        let mut result: Vec<PayrollRun> = runs
            .values()
            .filter(|r| status.map_or(true, |s| r.status == s))
            .cloned()
            .collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }

    fn update(&self, run: PayrollRun) -> Result<PayrollRun, String> {
        let mut runs = self.runs.write().unwrap();
        if runs.contains_key(&run.id) {
            runs.insert(run.id.clone(), run.clone());
            Ok(run)
        } else {
            Err(format!("Payroll run not found: {}", run.id))
        }
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let mut runs = self.runs.write().unwrap();
        if let Some(run) = runs.get_mut(id) {
            run.cancel();
            Ok(true)
        } else {
            Err(format!("Payroll run not found: {}", id))
        }
    }

    fn get_latest(&self) -> Result<Option<PayrollRun>, String> {
        let runs = self.runs.read().unwrap();
        let mut all_runs: Vec<&PayrollRun> = runs.values().collect();
        all_runs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(all_runs.first().cloned().map(|r| r.clone()))
    }
}

/// In-memory implementation of PayrollEntryRepository
pub struct InMemoryPayrollEntryRepository {
    entries: Arc<RwLock<HashMap<String, PayrollEntry>>>,
    next_id: Arc<RwLock<u32>>,
}

impl InMemoryPayrollEntryRepository {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    fn generate_id(&self) -> String {
        let mut counter = self.next_id.write().unwrap();
        let id = format!("payentry-{:03}", *counter);
        *counter += 1;
        id
    }
}

impl PayrollEntryRepository for InMemoryPayrollEntryRepository {
    fn create(&self, mut entry: PayrollEntry) -> Result<PayrollEntry, String> {
        if entry.id.is_empty() {
            entry.id = self.generate_id();
        }
        let mut entries = self.entries.write().unwrap();
        entries.insert(entry.id.clone(), entry.clone());
        Ok(entry)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<PayrollEntry>, String> {
        let entries = self.entries.read().unwrap();
        Ok(entries.get(id).cloned())
    }

    fn get_by_payroll_run(&self, payroll_run_id: &str) -> Result<Vec<PayrollEntry>, String> {
        let entries = self.entries.read().unwrap();
        let mut result: Vec<PayrollEntry> = entries
            .values()
            .filter(|e| e.payroll_run_id == payroll_run_id)
            .cloned()
            .collect();
        result.sort_by(|a, b| a.employee_id.cmp(&b.employee_id));
        Ok(result)
    }

    fn get_by_employee_and_run(
        &self,
        employee_id: &str,
        payroll_run_id: &str,
    ) -> Result<Option<PayrollEntry>, String> {
        let entries = self.entries.read().unwrap();
        Ok(entries
            .values()
            .find(|e| e.employee_id == employee_id && e.payroll_run_id == payroll_run_id)
            .cloned())
    }

    fn list_by_employee(&self, employee_id: &str) -> Result<Vec<PayrollEntry>, String> {
        let entries = self.entries.read().unwrap();
        let mut result: Vec<PayrollEntry> = entries
            .values()
            .filter(|e| e.employee_id == employee_id)
            .cloned()
            .collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }

    fn update(&self, entry: PayrollEntry) -> Result<PayrollEntry, String> {
        let mut entries = self.entries.write().unwrap();
        if entries.contains_key(&entry.id) {
            entries.insert(entry.id.clone(), entry.clone());
            Ok(entry)
        } else {
            Err(format!("Payroll entry not found: {}", entry.id))
        }
    }

    fn update_many(&self, entries_update: &[PayrollEntry]) -> Result<(), String> {
        let mut entries = self.entries.write().unwrap();
        for entry in entries_update {
            entries.insert(entry.id.clone(), entry.clone());
        }
        Ok(())
    }

    fn delete_by_payroll_run(&self, payroll_run_id: &str) -> Result<bool, String> {
        let mut entries = self.entries.write().unwrap();
        let count_before = entries.len();
        entries.retain(|_, e| e.payroll_run_id != payroll_run_id);
        Ok(entries.len() < count_before)
    }

    fn get_total_net_by_run(&self, payroll_run_id: &str) -> Result<f64, String> {
        let entries = self.entries.read().unwrap();
        let total: f64 = entries
            .values()
            .filter(|e| e.payroll_run_id == payroll_run_id)
            .map(|e| e.net_income)
            .sum();
        Ok(total)
    }

    fn get_total_gross_by_run(&self, payroll_run_id: &str) -> Result<f64, String> {
        let entries = self.entries.read().unwrap();
        let total: f64 = entries
            .values()
            .filter(|e| e.payroll_run_id == payroll_run_id)
            .map(|e| e.gross_income)
            .sum();
        Ok(total)
    }

    fn get_total_deductions_by_run(&self, payroll_run_id: &str) -> Result<f64, String> {
        let entries = self.entries.read().unwrap();
        let total: f64 = entries
            .values()
            .filter(|e| e.payroll_run_id == payroll_run_id)
            .map(|e| e.afp_deduction + e.onp_deduction + e.essalud + e.itf + e.other_deductions)
            .sum();
        Ok(total)
    }
}
