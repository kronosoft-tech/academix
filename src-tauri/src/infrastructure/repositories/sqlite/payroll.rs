//! Payroll SQLite Repository
//!
//! Implements PayrollRepository and PayrollEntryRepository using SQLite.

use crate::application::ports::payroll::{PayrollEntryRepository, PayrollRepository};
use crate::domain::entities::payroll::{
    PayrollEntry, PayrollEntryStatus, PayrollRun, PayrollRunStatus,
};
use crate::infrastructure::database::SqlitePool;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// SQLite implementation of PayrollRepository
pub struct SqlitePayrollRepository {
    pool: Arc<SqlitePool>,
}

impl SqlitePayrollRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    fn row_to_payroll_run(row: &rusqlite::Row<'_>) -> rusqlite::Result<PayrollRun> {
        let status_str: String = row.get(3)?;
        let period_start_str: String = row.get(4)?;
        let period_end_str: String = row.get(5)?;
        let created_str: String = row.get(6)?;

        let status = PayrollRunStatus::from_str(&status_str).unwrap_or(PayrollRunStatus::Draft);

        Ok(PayrollRun {
            id: row.get(0)?,
            period_start: DateTime::parse_from_rfc3339(&period_start_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            period_end: DateTime::parse_from_rfc3339(&period_end_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            status,
            total_gross: row.get(1)?,
            total_deductions: row.get(2)?,
            total_net: row.get(7)?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            created_by: row.get(8)?,
        })
    }

    fn status_to_string(status: PayrollRunStatus) -> &'static str {
        status.as_str()
    }
}

impl PayrollRepository for SqlitePayrollRepository {
    fn create(&self, run: PayrollRun) -> Result<PayrollRun, String> {
        let sql = "INSERT INTO payroll_runs (
                      id, total_gross, total_deductions, total_net,
                      status, period_start, period_end, created_at, created_by
                  ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        self.pool
            .execute(
                sql,
                &[
                    &run.id,
                    &run.total_gross.to_string(),
                    &run.total_deductions.to_string(),
                    &run.total_net.to_string(),
                    &Self::status_to_string(run.status),
                    &run.period_start.to_rfc3339(),
                    &run.period_end.to_rfc3339(),
                    &run.created_at.to_rfc3339(),
                    &run.created_by,
                ],
            )
            .map_err(|e| e.to_string())?;

        Ok(run)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<PayrollRun>, String> {
        let sql = "SELECT id, total_gross, total_deductions, total_net,
                         status, period_start, period_end, created_at, created_by
                  FROM payroll_runs WHERE id = ?";

        self.pool
            .query_row(sql, &[&id], Self::row_to_payroll_run)
            .map_err(|e| e.to_string())
    }

    fn get_by_period(
        &self,
        period_start: &str,
        period_end: &str,
    ) -> Result<Option<PayrollRun>, String> {
        let sql = "SELECT id, total_gross, total_deductions, total_net,
                         status, period_start, period_end, created_at, created_by
                  FROM payroll_runs 
                  WHERE date(period_start) = date(?) AND date(period_end) = date(?)";

        self.pool
            .query_row(sql, &[&period_start, &period_end], Self::row_to_payroll_run)
            .map_err(|e| e.to_string())
    }

    fn list(&self, _status: Option<PayrollRunStatus>) -> Result<Vec<PayrollRun>, String> {
        let sql = "SELECT id, total_gross, total_deductions, total_net,
                         status, period_start, period_end, created_at, created_by
                  FROM payroll_runs ORDER BY period_start DESC";

        self.pool
            .query(sql, &[], Self::row_to_payroll_run)
            .map_err(|e| e.to_string())
    }

    fn update(&self, run: PayrollRun) -> Result<PayrollRun, String> {
        let sql = "UPDATE payroll_runs 
                   SET total_gross = ?, total_deductions = ?, total_net = ?, status = ?
                   WHERE id = ?";

        let affected = self
            .pool
            .execute(
                sql,
                &[
                    &run.total_gross.to_string(),
                    &run.total_deductions.to_string(),
                    &run.total_net.to_string(),
                    &Self::status_to_string(run.status),
                    &run.id,
                ],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("PayrollRun not found: {}", run.id));
        }

        Ok(run)
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let sql = "UPDATE payroll_runs SET status = 'cancelled' WHERE id = ?";

        let affected = self.pool.execute(sql, &[&id]).map_err(|e| e.to_string())?;

        Ok(affected > 0)
    }

    fn get_latest(&self) -> Result<Option<PayrollRun>, String> {
        let sql = "SELECT id, total_gross, total_deductions, total_net,
                         status, period_start, period_end, created_at, created_by
                  FROM payroll_runs ORDER BY period_start DESC LIMIT 1";

        self.pool
            .query_row(sql, &[], Self::row_to_payroll_run)
            .map_err(|e| e.to_string())
    }
}

/// SQLite implementation of PayrollEntryRepository
pub struct SqlitePayrollEntryRepository {
    pool: Arc<SqlitePool>,
}

impl SqlitePayrollEntryRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    fn row_to_payroll_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<PayrollEntry> {
        let status_str: String = row.get(29)?;
        let created_str: String = row.get(30)?;

        let status =
            PayrollEntryStatus::from_str(&status_str).unwrap_or(PayrollEntryStatus::Calculated);

        Ok(PayrollEntry {
            id: row.get(0)?,
            payroll_run_id: row.get(1)?,
            employee_id: row.get(2)?,
            base_salary: row.get(3)?,
            hours_worked: row.get(4)?,
            overtime_hours: row.get(5)?,
            overtime_amount: row.get(6)?,
            bonuses: row.get(7)?,
            commissions: row.get(8)?,
            mobility: row.get(9)?,
            food: row.get(10)?,
            other_income: row.get(11)?,
            afp_deduction: row.get(12)?,
            onp_deduction: row.get(13)?,
            essalud: row.get(14)?,
            itf: row.get(15)?,
            other_deductions: row.get(16)?,
            gross_income: row.get(17)?,
            net_income: row.get(18)?,
            status,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    fn status_to_string(status: PayrollEntryStatus) -> &'static str {
        status.as_str()
    }
}

impl PayrollEntryRepository for SqlitePayrollEntryRepository {
    fn create(&self, entry: PayrollEntry) -> Result<PayrollEntry, String> {
        let sql = "INSERT INTO payroll_entries (
                      id, payroll_run_id, employee_id,
                      base_salary, hours_worked, overtime_hours, overtime_amount,
                      bonuses, commissions, mobility, food, other_income,
                      afp_deduction, onp_deduction, essalud, itf, other_deductions,
                      gross_income, net_income, status, created_at
                  ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        self.pool
            .execute(
                sql,
                &[
                    &entry.id,
                    &entry.payroll_run_id,
                    &entry.employee_id,
                    &entry.base_salary.to_string(),
                    &entry.hours_worked.to_string(),
                    &entry.overtime_hours.to_string(),
                    &entry.overtime_amount.to_string(),
                    &entry.bonuses.to_string(),
                    &entry.commissions.to_string(),
                    &entry.mobility.to_string(),
                    &entry.food.to_string(),
                    &entry.other_income.to_string(),
                    &entry.afp_deduction.to_string(),
                    &entry.onp_deduction.to_string(),
                    &entry.essalud.to_string(),
                    &entry.itf.to_string(),
                    &entry.other_deductions.to_string(),
                    &entry.gross_income.to_string(),
                    &entry.net_income.to_string(),
                    &Self::status_to_string(entry.status),
                    &entry.created_at.to_rfc3339(),
                ],
            )
            .map_err(|e| e.to_string())?;

        Ok(entry)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<PayrollEntry>, String> {
        let sql = "SELECT id, payroll_run_id, employee_id,
                         base_salary, hours_worked, overtime_hours, overtime_amount,
                         bonuses, commissions, mobility, food, other_income,
                         afp_deduction, onp_deduction, essalud, itf, other_deductions,
                         gross_income, net_income, status, created_at
                  FROM payroll_entries WHERE id = ?";

        self.pool
            .query_row(sql, &[&id], Self::row_to_payroll_entry)
            .map_err(|e| e.to_string())
    }

    fn get_by_payroll_run(&self, payroll_run_id: &str) -> Result<Vec<PayrollEntry>, String> {
        let sql = "SELECT id, payroll_run_id, employee_id,
                          base_salary, hours_worked, overtime_hours, overtime_amount,
                          bonuses, commissions, mobility, food, other_income,
                          afp_deduction, onp_deduction, essalud, itf, other_deductions,
                          gross_income, net_income, status, created_at
                    FROM payroll_entries WHERE payroll_run_id = ?";

        self.pool
            .query(sql, &[&payroll_run_id], Self::row_to_payroll_entry)
            .map_err(|e| e.to_string())
    }

    fn get_by_employee_and_run(
        &self,
        employee_id: &str,
        payroll_run_id: &str,
    ) -> Result<Option<PayrollEntry>, String> {
        let sql = "SELECT id, payroll_run_id, employee_id,
                          base_salary, hours_worked, overtime_hours, overtime_amount,
                          bonuses, commissions, mobility, food, other_income,
                          afp_deduction, onp_deduction, essalud, itf, other_deductions,
                          gross_income, net_income, status, created_at
                    FROM payroll_entries 
                    WHERE employee_id = ? AND payroll_run_id = ?";

        self.pool
            .query_row(
                sql,
                &[&employee_id, &payroll_run_id],
                Self::row_to_payroll_entry,
            )
            .map_err(|e| e.to_string())
    }

    fn list_by_employee(&self, employee_id: &str) -> Result<Vec<PayrollEntry>, String> {
        let sql = "SELECT id, payroll_run_id, employee_id,
                          base_salary, hours_worked, overtime_hours, overtime_amount,
                          bonuses, commissions, mobility, food, other_income,
                          afp_deduction, onp_deduction, essalud, itf, other_deductions,
                          gross_income, net_income, status, created_at
                    FROM payroll_entries WHERE employee_id = ? ORDER BY created_at DESC";

        self.pool
            .query(sql, &[&employee_id], Self::row_to_payroll_entry)
            .map_err(|e| e.to_string())
    }

    fn update(&self, entry: PayrollEntry) -> Result<PayrollEntry, String> {
        let sql = "UPDATE payroll_entries 
                  SET base_salary = ?, hours_worked = ?, overtime_hours = ?, overtime_amount = ?,
                      bonuses = ?, commissions = ?, mobility = ?, food = ?, other_income = ?,
                      afp_deduction = ?, onp_deduction = ?, essalud = ?, itf = ?, other_deductions = ?,
                      gross_income = ?, net_income = ?, status = ?
                  WHERE id = ?";

        let affected = self
            .pool
            .execute(
                sql,
                &[
                    &entry.base_salary.to_string(),
                    &entry.hours_worked.to_string(),
                    &entry.overtime_hours.to_string(),
                    &entry.overtime_amount.to_string(),
                    &entry.bonuses.to_string(),
                    &entry.commissions.to_string(),
                    &entry.mobility.to_string(),
                    &entry.food.to_string(),
                    &entry.other_income.to_string(),
                    &entry.afp_deduction.to_string(),
                    &entry.onp_deduction.to_string(),
                    &entry.essalud.to_string(),
                    &entry.itf.to_string(),
                    &entry.other_deductions.to_string(),
                    &entry.gross_income.to_string(),
                    &entry.net_income.to_string(),
                    &Self::status_to_string(entry.status),
                    &entry.id,
                ],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("PayrollEntry not found: {}", entry.id));
        }

        Ok(entry)
    }

    fn update_many(&self, entries: &[PayrollEntry]) -> Result<(), String> {
        for entry in entries {
            self.update(entry.clone())?;
        }
        Ok(())
    }

    fn delete_by_payroll_run(&self, payroll_run_id: &str) -> Result<bool, String> {
        let sql = "DELETE FROM payroll_entries WHERE payroll_run_id = ?";

        let affected = self
            .pool
            .execute(sql, &[&payroll_run_id])
            .map_err(|e| e.to_string())?;

        Ok(affected > 0)
    }

    fn get_total_net_by_run(&self, payroll_run_id: &str) -> Result<f64, String> {
        let sql =
            "SELECT COALESCE(SUM(net_income), 0) FROM payroll_entries WHERE payroll_run_id = ?";
        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        let total: f64 = conn
            .query_row(sql, [payroll_run_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        Ok(total)
    }

    fn get_total_gross_by_run(&self, payroll_run_id: &str) -> Result<f64, String> {
        let sql =
            "SELECT COALESCE(SUM(gross_income), 0) FROM payroll_entries WHERE payroll_run_id = ?";
        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        let total: f64 = conn
            .query_row(sql, [payroll_run_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        Ok(total)
    }

    fn get_total_deductions_by_run(&self, payroll_run_id: &str) -> Result<f64, String> {
        let sql = "SELECT COALESCE(SUM(afp_deduction + onp_deduction + essalud + itf + other_deductions), 0) 
                  FROM payroll_entries WHERE payroll_run_id = ?";
        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        let total: f64 = conn
            .query_row(sql, [payroll_run_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        Ok(total)
    }
}
