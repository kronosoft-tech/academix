//! Accounting DTOs - Simplified income/expense model

use serde::{Deserialize, Serialize};

/// Create accounting entry request
#[derive(Debug, Deserialize)]
pub struct CreateEntryRequest {
    pub date: String,
    pub entry_type: String, // "income" or "expense"
    pub category: String,   // "tuition", "rent", "salaries", "utilities", "other"
    pub description: String,
    pub amount: f64,
    pub reference: Option<String>,
}

/// Accounting entry response
#[derive(Debug, Serialize)]
pub struct AccountingEntryDto {
    pub id: String,
    pub date: String,
    pub entry_type: String,
    pub category: String,
    pub description: String,
    pub amount: f64,
    pub reference: Option<String>,
    pub created_at: String,
}

impl From<crate::domain::entities::accounting::AccountingEntry> for AccountingEntryDto {
    fn from(entry: crate::domain::entities::accounting::AccountingEntry) -> Self {
        Self {
            id: entry.id,
            date: entry.date,
            entry_type: entry.entry_type.as_str().to_string(),
            category: entry.category.as_str().to_string(),
            description: entry.description,
            amount: entry.amount,
            reference: entry.reference,
            created_at: entry.created_at,
        }
    }
}

/// Accounting summary for dashboard
#[derive(Debug, Serialize)]
pub struct AccountingSummaryDto {
    pub total_income: f64,
    pub total_expenses: f64,
    pub net_balance: f64,
    pub entry_count: i64,
    pub recent_entries: Vec<AccountingEntryDto>,
    pub monthly_data: Vec<MonthlyDataPointDto>,
    pub expenses_by_category: Vec<CategoryTotalDto>,
    pub income_by_category: Vec<CategoryTotalDto>,
}

/// Monthly data point for charts
#[derive(Debug, Serialize)]
pub struct MonthlyDataPointDto {
    pub month: String,
    pub income: f64,
    pub expenses: f64,
}

/// Category total for breakdown
#[derive(Debug, Serialize)]
pub struct CategoryTotalDto {
    pub category_name: String,
    pub amount: f64,
}
