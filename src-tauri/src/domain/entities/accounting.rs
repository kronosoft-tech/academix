//! Accounting Entity - Domain Model
//!
//! Simplified accounting model with income/expense tracking.

use serde::{Deserialize, Serialize};

/// Entry type: income or expense
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    Income,
    Expense,
}

impl EntryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryType::Income => "income",
            EntryType::Expense => "expense",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "income" => Some(EntryType::Income),
            "expense" => Some(EntryType::Expense),
            _ => None,
        }
    }
}

/// Accounting categories for income and expense entries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountingCategory {
    // Income categories
    Tuition,
    OtherIncome,
    // Expense categories
    Rent,
    Salaries,
    Utilities,
    OtherExpense,
}

impl AccountingCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountingCategory::Tuition => "tuition",
            AccountingCategory::OtherIncome => "other",
            AccountingCategory::Rent => "rent",
            AccountingCategory::Salaries => "salaries",
            AccountingCategory::Utilities => "utilities",
            AccountingCategory::OtherExpense => "other",
        }
    }

    pub fn from_str(s: &str, entry_type: &EntryType) -> Option<Self> {
        match entry_type {
            EntryType::Income => match s.to_lowercase().as_str() {
                "tuition" => Some(AccountingCategory::Tuition),
                "other" => Some(AccountingCategory::OtherIncome),
                _ => None,
            },
            EntryType::Expense => match s.to_lowercase().as_str() {
                "rent" => Some(AccountingCategory::Rent),
                "salaries" => Some(AccountingCategory::Salaries),
                "utilities" => Some(AccountingCategory::Utilities),
                "other" => Some(AccountingCategory::OtherExpense),
                _ => None,
            },
        }
    }

    /// Get display name in Spanish
    pub fn display_name(&self) -> &'static str {
        match self {
            AccountingCategory::Tuition => "Matrícula",
            AccountingCategory::OtherIncome => "Otros",
            AccountingCategory::Rent => "Arriendo",
            AccountingCategory::Salaries => "Sueldos",
            AccountingCategory::Utilities => "Servicios",
            AccountingCategory::OtherExpense => "Otros",
        }
    }

    /// Check if this category is an income category
    pub fn is_income(&self) -> bool {
        matches!(
            self,
            AccountingCategory::Tuition | AccountingCategory::OtherIncome
        )
    }

    /// Check if this category is an expense category
    pub fn is_expense(&self) -> bool {
        matches!(
            self,
            AccountingCategory::Rent
                | AccountingCategory::Salaries
                | AccountingCategory::Utilities
                | AccountingCategory::OtherExpense
        )
    }
}

/// AccountingEntry - represents a simplified accounting entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingEntry {
    pub id: String,
    pub date: String, // ISO 8601 date
    pub entry_type: EntryType,
    pub category: AccountingCategory,
    pub description: String,
    pub amount: f64,
    pub reference: Option<String>,
    pub created_at: String, // ISO 8601 datetime
}

impl AccountingEntry {
    /// Create a new accounting entry
    pub fn new(
        id: String,
        date: String,
        entry_type: EntryType,
        category: AccountingCategory,
        description: String,
        amount: f64,
    ) -> Self {
        Self {
            id,
            date,
            entry_type,
            category,
            description,
            amount,
            reference: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create an income entry
    pub fn income(
        id: String,
        date: String,
        category: AccountingCategory,
        description: String,
        amount: f64,
    ) -> Self {
        Self::new(id, date, EntryType::Income, category, description, amount)
    }

    /// Create an expense entry
    pub fn expense(
        id: String,
        date: String,
        category: AccountingCategory,
        description: String,
        amount: f64,
    ) -> Self {
        Self::new(id, date, EntryType::Expense, category, description, amount)
    }

    /// Validate entry (amount > 0)
    pub fn is_valid(&self) -> bool {
        self.amount > 0.0
    }

    /// Get display string for the entry
    pub fn display(&self) -> String {
        format!(
            "{} - {} - {} S/ {:.2}",
            self.date,
            self.entry_type.as_str(),
            self.description,
            self.amount
        )
    }
}

/// Accounting summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingSummary {
    pub total_income: f64,
    pub total_expenses: f64,
    pub net_balance: f64,
    pub entry_count: i64,
    pub recent_entries: Vec<AccountingEntry>,
    pub monthly_data: Vec<MonthlyDataPoint>,
    pub expenses_by_category: Vec<CategoryTotal>,
    pub income_by_category: Vec<CategoryTotal>,
}

/// Monthly data point for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyDataPoint {
    pub month: String,
    pub income: f64,
    pub expenses: f64,
}

/// Category total for breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTotal {
    pub category_name: String,
    pub amount: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_type_conversion() {
        assert_eq!(EntryType::from_str("income"), Some(EntryType::Income));
        assert_eq!(EntryType::from_str("expense"), Some(EntryType::Expense));
        assert_eq!(EntryType::from_str("invalid"), None);
    }

    #[test]
    fn test_category_conversion() {
        assert_eq!(
            AccountingCategory::from_str("tuition", &EntryType::Income),
            Some(AccountingCategory::Tuition)
        );
        assert_eq!(
            AccountingCategory::from_str("rent", &EntryType::Expense),
            Some(AccountingCategory::Rent)
        );
        assert_eq!(
            AccountingCategory::from_str("tuition", &EntryType::Expense),
            None
        );
    }

    #[test]
    fn test_accounting_entry_creation() {
        let entry = AccountingEntry::new(
            "test-id".to_string(),
            "2026-07-14".to_string(),
            EntryType::Income,
            AccountingCategory::Tuition,
            "Monthly fee".to_string(),
            500.0,
        );

        assert!(entry.is_valid());
        assert_eq!(entry.entry_type, EntryType::Income);
        assert_eq!(entry.category, AccountingCategory::Tuition);
    }

    #[test]
    fn test_accounting_entry_invalid_amount() {
        let entry = AccountingEntry::new(
            "test-id".to_string(),
            "2026-07-14".to_string(),
            EntryType::Income,
            AccountingCategory::Tuition,
            "Monthly fee".to_string(),
            0.0,
        );

        assert!(!entry.is_valid());
    }
}
