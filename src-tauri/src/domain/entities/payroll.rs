//! Payroll Entity - Domain Model
//!
//! Pure domain entities for payroll management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Payroll run status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PayrollRunStatus {
    Draft,      // Borrador - no calculado
    Calculated, // Calculado - listo para revisión
    Confirmed,  // Confirmado - payroll pagado
    Cancelled,  // Cancelado
}

impl PayrollRunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PayrollRunStatus::Draft => "draft",
            PayrollRunStatus::Calculated => "calculated",
            PayrollRunStatus::Confirmed => "confirmed",
            PayrollRunStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "draft" => Some(PayrollRunStatus::Draft),
            "calculated" => Some(PayrollRunStatus::Calculated),
            "confirmed" => Some(PayrollRunStatus::Confirmed),
            "cancelled" => Some(PayrollRunStatus::Cancelled),
            _ => None,
        }
    }
}

/// Payroll entry status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PayrollEntryStatus {
    Calculated, // Calculado
    Paid,       // Pagado
    Cancelled,  // Cancelado
}

impl PayrollEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PayrollEntryStatus::Calculated => "calculated",
            PayrollEntryStatus::Paid => "paid",
            PayrollEntryStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "calculated" => Some(PayrollEntryStatus::Calculated),
            "paid" => Some(PayrollEntryStatus::Paid),
            "cancelled" => Some(PayrollEntryStatus::Cancelled),
            _ => None,
        }
    }
}

/// PayrollRun - represents a payroll period (monthly)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollRun {
    pub id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub status: PayrollRunStatus,
    pub total_gross: f64,
    pub total_deductions: f64,
    pub total_net: f64,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

impl PayrollRun {
    /// Create a new payroll run
    pub fn new(
        id: String,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        created_by: String,
    ) -> Self {
        Self {
            id,
            period_start,
            period_end,
            status: PayrollRunStatus::Draft,
            total_gross: 0.0,
            total_deductions: 0.0,
            total_net: 0.0,
            created_at: Utc::now(),
            created_by,
        }
    }

    /// Update totals after calculation
    pub fn set_totals(&mut self, total_gross: f64, total_deductions: f64, total_net: f64) {
        self.total_gross = total_gross;
        self.total_deductions = total_deductions;
        self.total_net = total_net;
    }

    /// Mark as calculated
    pub fn mark_calculated(&mut self) {
        self.status = PayrollRunStatus::Calculated;
    }

    /// Confirm payroll
    pub fn confirm(&mut self) {
        self.status = PayrollRunStatus::Confirmed;
    }

    /// Cancel payroll
    pub fn cancel(&mut self) {
        self.status = PayrollRunStatus::Cancelled;
    }

    /// Get period display string (e.g., "Abril 2026")
    pub fn period_display(&self) -> String {
        let start = self.period_start.format("%B %Y").to_string();
        start
    }
}

/// PayrollEntry - individual employee payroll record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollEntry {
    pub id: String,
    pub payroll_run_id: String,
    pub employee_id: String,
    // Income components
    pub base_salary: f64,
    pub hours_worked: f64,
    pub overtime_hours: f64,
    pub overtime_amount: f64,
    pub bonuses: f64,
    pub commissions: f64,
    pub mobility: f64,
    pub food: f64,
    pub other_income: f64,
    // Deduction components
    pub afp_deduction: f64,
    pub onp_deduction: f64,
    pub essalud: f64,
    pub itf: f64,
    pub other_deductions: f64,
    // Calculated totals
    pub gross_income: f64,
    pub net_income: f64,
    // Status
    pub status: PayrollEntryStatus,
    pub created_at: DateTime<Utc>,
}

impl PayrollEntry {
    /// Create a new payroll entry
    pub fn new(id: String, payroll_run_id: String, employee_id: String, base_salary: f64) -> Self {
        Self {
            id,
            payroll_run_id,
            employee_id,
            base_salary,
            hours_worked: 0.0,
            overtime_hours: 0.0,
            overtime_amount: 0.0,
            bonuses: 0.0,
            commissions: 0.0,
            mobility: 0.0,
            food: 0.0,
            other_income: 0.0,
            afp_deduction: 0.0,
            onp_deduction: 0.0,
            essalud: 0.0,
            itf: 0.0,
            other_deductions: 0.0,
            gross_income: base_salary,
            net_income: base_salary,
            status: PayrollEntryStatus::Calculated,
            created_at: Utc::now(),
        }
    }

    /// Calculate gross income (total income before deductions)
    pub fn calculate_gross(&mut self) {
        self.gross_income = self.base_salary
            + self.overtime_amount
            + self.bonuses
            + self.commissions
            + self.mobility
            + self.food
            + self.other_income;
    }

    /// Calculate net income (after deductions)
    pub fn calculate_net(&mut self) {
        self.net_income = self.gross_income
            - self.afp_deduction
            - self.onp_deduction
            - self.essalud
            - self.itf
            - self.other_deductions;
    }

    /// Set overtime hours and calculate amount
    /// Formula: (base_salary / 240) * 2 * hours
    pub fn set_overtime(&mut self, hours: f64) {
        let hourly_rate = self.base_salary / 240.0;
        self.overtime_hours = hours;
        self.overtime_amount = hourly_rate * 2.0 * hours;
    }

    /// Set bonuses
    pub fn set_bonuses(&mut self, amount: f64) {
        self.bonuses = amount;
    }

    /// Set commissions
    pub fn set_commissions(&mut self, amount: f64) {
        self.commissions = amount;
    }

    /// Set mobility allowance
    pub fn set_mobility(&mut self, amount: f64) {
        self.mobility = amount;
    }

    /// Set food allowance
    pub fn set_food(&mut self, amount: f64) {
        self.food = amount;
    }

    /// Set other income
    pub fn set_other_income(&mut self, amount: f64) {
        self.other_income = amount;
    }

    /// Set AFP deduction
    pub fn set_afp_deduction(&mut self, amount: f64) {
        self.afp_deduction = amount;
    }

    /// Set ONP deduction
    pub fn set_onp_deduction(&mut self, amount: f64) {
        self.onp_deduction = amount;
    }

    /// Set Essalud (employer contribution, shown in payslip)
    pub fn set_essalud(&mut self, amount: f64) {
        self.essalud = amount;
    }

    /// Set ITF
    pub fn set_itf(&mut self, amount: f64) {
        self.itf = amount;
    }

    /// Set other deductions
    pub fn set_other_deductions(&mut self, amount: f64) {
        self.other_deductions = amount;
    }

    /// Mark as paid
    pub fn mark_paid(&mut self) {
        self.status = PayrollEntryStatus::Paid;
    }

    /// Cancel entry
    pub fn cancel(&mut self) {
        self.status = PayrollEntryStatus::Cancelled;
    }
}

/// Payroll calculation constants for Colombia
pub mod constants {
    /// Hourly rate divisor (48 hours/week * 4.33 weeks/month ~ 208 hours)
    pub const HOURS_PER_MONTH: f64 = 208.0;

    /// Overtime multiplier (diurnal)
    pub const OVERTIME_MULTIPLIER: f64 = 1.25;

    /// Overtime night multiplier
    pub const OVERTIME_NIGHT_MULTIPLIER: f64 = 1.75;

    /// Overtime Sunday/holiday multiplier
    pub const OVERTIME_HOLIDAY_MULTIPLIER: f64 = 2.0;

    /// SMMLV 2026 (Salario Mínimo Mensual Legal Vigente)
    pub const SMMLV: f64 = 1500000.0;

    /// Transport allowance (2026)
    pub const TRANSPORT_ALLOWANCE: f64 = 200000.0;

    /// Transport allowance threshold
    pub const TRANSPORT_THRESHOLD: f64 = 2600000.0;

    /// Health contribution employee (EPS) - 4%
    pub const HEALTH_RATE: f64 = 0.04;

    /// Pension contribution employee (AFP) - 4%
    pub const PENSION_RATE: f64 = 0.04;

    /// Health contribution employer - 8.5%
    pub const HEALTH_EMPLOYER_RATE: f64 = 0.085;

    /// Pension contribution employer - 12%
    pub const PENSION_EMPLOYER_RATE: f64 = 0.12;

    /// ARL contribution employer (varies by risk class)
    pub const ARL_RATE: f64 = 0.0522; // Class II - 5.22%

    /// ICBF contribution - 3%
    pub const ICBF_RATE: f64 = 0.03;

    /// SENA contribution - 2%
    pub const SENA_RATE: f64 = 0.02;

    /// Unemployment insurance ( parafiscales) - 1%
    pub const UNEMPLOYMENT_RATE: f64 = 0.01;

    /// Bonus deduction (employees earning < 10 SMMLV)
    pub const BONUS_EXEMPT_RATE: f64 = 0.00;

    /// VAT rate for invoices
    pub const VAT_RATE: f64 = 0.19; // 19%

    /// GMF (Gravamen a los Movimientos Financieros) - 4x1000
    pub const GMF_RATE: f64 = 0.004;

    /// ICA rate (varies by municipality, using average 0.5%)
    pub const ICA_RATE: f64 = 0.005;
}

/// Calculate health contribution (EPS) - employee portion
pub fn calculate_health(deductible_income: f64) -> f64 {
    if deductible_income <= constants::SMMLV * 2.0 {
        deductible_income * constants::HEALTH_RATE
    } else {
        0.0 // For higher incomes, no employee contribution (employer covers)
    }
}

/// Calculate pension contribution (AFP) - employee portion
pub fn calculate_pension(deductible_income: f64) -> f64 {
    deductible_income * constants::PENSION_RATE
}

/// Calculate health contribution (employer portion)
pub fn calculate_health_employer(gross_income: f64) -> f64 {
    gross_income * constants::HEALTH_EMPLOYER_RATE
}

/// Calculate pension contribution (employer portion)
pub fn calculate_pension_employer(gross_income: f64) -> f64 {
    gross_income * constants::PENSION_EMPLOYER_RATE
}

/// Calculate ARL contribution (employer portion)
pub fn calculate_arl(gross_income: f64) -> f64 {
    gross_income * constants::ARL_RATE
}

/// Calculate parafiscales (ICBF, SENA, Unemployment)
pub fn calculate_parafiscales(gross_income: f64) -> f64 {
    gross_income * (constants::ICBF_RATE + constants::SENA_RATE + constants::UNEMPLOYMENT_RATE)
}

/// Calculate total employer payroll taxes
pub fn calculate_employer_taxes(gross_income: f64) -> f64 {
    calculate_health_employer(gross_income)
        + calculate_pension_employer(gross_income)
        + calculate_arl(gross_income)
        + calculate_parafiscales(gross_income)
}

/// Calculate transport allowance (only for incomes below threshold)
pub fn calculate_transport_allowance(base_salary: f64) -> f64 {
    if base_salary + calculate_transport_allowance(0.0) <= constants::TRANSPORT_THRESHOLD {
        constants::TRANSPORT_ALLOWANCE
    } else {
        0.0
    }
}

/// Calculate overtime amount
pub fn calculate_overtime(base_salary: f64, hours: f64, is_night: bool, is_holiday: bool) -> f64 {
    let hourly_rate = base_salary / constants::HOURS_PER_MONTH;
    let multiplier = if is_holiday {
        constants::OVERTIME_HOLIDAY_MULTIPLIER
    } else if is_night {
        constants::OVERTIME_NIGHT_MULTIPLIER
    } else {
        constants::OVERTIME_MULTIPLIER
    };
    hourly_rate * multiplier * hours
}

/// Calculate GMF (4x1000) on bank transactions
pub fn calculate_gmf(amount: f64) -> f64 {
    amount * constants::GMF_RATE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payroll_entry_creation() {
        let entry = PayrollEntry::new(
            "entry-id".to_string(),
            "run-id".to_string(),
            "employee-id".to_string(),
            3000.0,
        );

        assert_eq!(entry.base_salary, 3000.0);
        assert_eq!(entry.gross_income, 3000.0);
        assert_eq!(entry.net_income, 3000.0);
    }

    #[test]
    fn test_calculate_gross() {
        let mut entry = PayrollEntry::new(
            "entry-id".to_string(),
            "run-id".to_string(),
            "employee-id".to_string(),
            3000.0,
        );

        entry.set_overtime(10.0);
        entry.set_bonuses(500.0);
        entry.set_mobility(200.0);
        entry.calculate_gross();

        // 3000 + (3000/240*2*10) + 500 + 200 = 3000 + 250 + 500 + 200 = 3950
        assert!((entry.gross_income - 3950.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_net() {
        let mut entry = PayrollEntry::new(
            "entry-id".to_string(),
            "run-id".to_string(),
            "employee-id".to_string(),
            3000.0,
        );

        entry.set_bonuses(500.0);
        entry.calculate_gross();
        entry.set_afp_deduction(337.50); // 11.25% of 3000
        entry.set_essalud(270.0);
        entry.calculate_net();

        // Gross: 3500, Deductions: 337.50 + 270 = 607.50
        // Net: 3500 - 607.50 = 2892.50
        assert!((entry.net_income - 2892.50).abs() < 0.01);
    }

    #[test]
    fn test_afp_calculation() {
        let deduction = calculate_afp(3000.0, "prima");
        assert!((deduction - 337.50).abs() < 0.01); // 11.25% of 3000
    }

    #[test]
    fn test_essalud_calculation() {
        let essalud = calculate_essalud(3000.0);
        assert!((essalud - 270.0).abs() < 0.01); // 9% of 3000
    }

    #[test]
    fn test_itf_above_threshold() {
        let itf = calculate_itf(4000.0);
        assert!((itf - 20.0).abs() < 0.01); // 0.5% of 4000 = 20
    }

    #[test]
    fn test_itf_below_threshold() {
        let itf = calculate_itf(3000.0);
        assert!(itf == 0.0);
    }

    #[test]
    fn test_overtime_calculation() {
        let overtime = calculate_overtime(3000.0, 10.0);
        // (3000/240) * 2 * 10 = 12.5 * 2 * 10 = 250
        assert!((overtime - 250.0).abs() < 0.01);
    }
}
