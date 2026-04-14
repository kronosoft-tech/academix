//! Accounting DTOs

use crate::domain::entities::accounting::{CategoryType, EntryType};
use serde::{Deserialize, Serialize};

/// Create accounting entry request
#[derive(Debug, Deserialize)]
pub struct CreateEntryRequest {
    pub date: String,
    pub description: String,
    pub debit_account: String,
    pub credit_account: String,
    pub amount: f64,
    pub entry_type: Option<EntryType>,
    pub reference: Option<String>,
    pub related_id: Option<String>,
    pub related_type: Option<String>,
}

/// Accounting entry response
#[derive(Debug, Serialize)]
pub struct AccountingEntryDto {
    pub id: String,
    pub date: String,
    pub reference: String,
    pub description: String,
    pub debit_account: String,
    pub debit_account_name: String,
    pub credit_account: String,
    pub credit_account_name: String,
    pub amount: f64,
    pub entry_type: EntryType,
    pub related_id: Option<String>,
    pub related_type: Option<String>,
    pub created_at: String,
    pub created_by: String,
}

impl From<crate::domain::entities::accounting::AccountingEntry> for AccountingEntryDto {
    fn from(entry: crate::domain::entities::accounting::AccountingEntry) -> Self {
        Self {
            id: entry.id,
            date: entry.date.to_rfc3339(),
            reference: entry.reference,
            description: entry.description,
            debit_account: entry.debit_account,
            debit_account_name: String::new(), // Will be filled by service
            credit_account: entry.credit_account,
            credit_account_name: String::new(), // Will be filled by service
            amount: entry.amount,
            entry_type: entry.entry_type,
            related_id: entry.related_id,
            related_type: entry.related_type,
            created_at: entry.created_at.to_rfc3339(),
            created_by: entry.created_by,
        }
    }
}

/// Account category response
#[derive(Debug, Serialize)]
pub struct AccountCategoryDto {
    pub id: String,
    pub code: String,
    pub display_code: String,
    pub name: String,
    pub category_type: CategoryType,
    pub parent_id: Option<String>,
    pub balance: f64,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::domain::entities::accounting::AccountCategory> for AccountCategoryDto {
    fn from(cat: crate::domain::entities::accounting::AccountCategory) -> Self {
        Self {
            id: cat.id.clone(),
            code: cat.code.clone(),
            display_code: cat.display_code(),
            name: cat.name,
            category_type: cat.category_type,
            parent_id: cat.parent_id,
            balance: cat.balance,
            active: cat.active,
            created_at: cat.created_at.to_rfc3339(),
            updated_at: cat.updated_at.to_rfc3339(),
        }
    }
}

/// Account category tree node
#[derive(Debug, Serialize)]
pub struct AccountCategoryTreeNode {
    pub category: AccountCategoryDto,
    pub children: Vec<AccountCategoryTreeNode>,
}

/// Trial balance response
#[derive(Debug, Serialize)]
pub struct TrialBalanceDto {
    pub as_of_date: String,
    pub accounts: Vec<TrialBalanceAccountDto>,
    pub total_debits: f64,
    pub total_credits: f64,
    pub is_balanced: bool,
}

#[derive(Debug, Serialize)]
pub struct TrialBalanceAccountDto {
    pub account_id: String,
    pub account_code: String,
    pub account_name: String,
    pub debit_balance: f64,
    pub credit_balance: f64,
    pub balance_type: String,
}

/// Income statement response
#[derive(Debug, Serialize)]
pub struct IncomeStatementDto {
    pub period_start: String,
    pub period_end: String,
    pub total_income: f64,
    pub total_expenses: f64,
    pub total_costs: f64,
    pub net_result: f64,
    pub is_profitable: bool,
    pub income_by_category: Vec<CategoryTotalDto>,
    pub expenses_by_category: Vec<CategoryTotalDto>,
}

#[derive(Debug, Serialize)]
pub struct CategoryTotalDto {
    pub category_id: String,
    pub category_name: String,
    pub total: f64,
}

/// Monthly data point for charts
#[derive(Debug, Serialize)]
pub struct MonthlyDataPoint {
    pub month: String,
    pub income: f64,
    pub expenses: f64,
}

/// Expense breakdown by category
#[derive(Debug, Serialize)]
pub struct ExpenseByCategory {
    pub category_name: String,
    pub amount: f64,
}

/// Accounting summary for dashboard
#[derive(Debug, Serialize)]
pub struct AccountingSummary {
    /// Total de dinero que ha entrado por ingresos (cuentas 6xxx)
    pub total_income: f64,
    /// Total de dinero gastado en el periodo (cuentas 4xxx y 5xxx)
    pub total_expenses: f64,
    /// Balance = Ingresos - Gastos (positivo = ganancia, negativo = pérdida)
    pub net_balance: f64,
    /// Cantidad de cuentas contables activas
    pub account_count: i64,
    /// Cantidad de asientos contables registrados
    pub entry_count: i64,
    /// Asientos más recientes
    pub recent_entries: Vec<AccountingEntryDto>,
    /// Datos mensuales para gráfico de tendencias
    pub monthly_data: Vec<MonthlyDataPoint>,
    /// Gastos por categoría
    pub expenses_by_category: Vec<ExpenseByCategory>,
    /// Ingresos por categoría
    pub income_by_category: Vec<ExpenseByCategory>,
}

/// Financial Balance (Balance Financiero) - shows Assets, Liabilities, Equity
#[derive(Debug, Serialize)]
pub struct FinancialBalanceDto {
    pub as_of_date: String,
    /// Activos (cuentas 1xxx)
    pub assets: Vec<AccountBalanceDto>,
    /// Pasivos (cuentas 2xxx)
    pub liabilities: Vec<AccountBalanceDto>,
    /// Patrimonio (cuentas 3xxx)
    pub equity: Vec<AccountBalanceDto>,
    /// Total Activos
    pub total_assets: f64,
    /// Total Pasivos
    pub total_liabilities: f64,
    /// Total Patrimonio
    pub total_equity: f64,
    /// Balance verification (Activos = Pasivos + Patrimonio)
    pub is_balanced: bool,
}

/// Account balance for financial balance report
#[derive(Debug, Serialize)]
pub struct AccountBalanceDto {
    pub account_code: String,
    pub account_name: String,
    pub balance: f64,
}
