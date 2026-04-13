//! Accounting Entity - Domain Model
//!
//! Pure domain entities for general ledger and chart of accounts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Account category types (based on Peruvian accounting)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CategoryType {
    Asset,     // 1xxx - Activos
    Liability, // 2xxx - Pasivos
    Equity,    // 3xxx - Patrimonio
    Expense,   // 4xxx - Gastos
    Cost,      // 5xxx - Costos
    Income,    // 6xxx - Ingresos
}

impl CategoryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CategoryType::Asset => "asset",
            CategoryType::Liability => "liability",
            CategoryType::Equity => "equity",
            CategoryType::Expense => "expense",
            CategoryType::Cost => "cost",
            CategoryType::Income => "income",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "asset" => Some(CategoryType::Asset),
            "liability" => Some(CategoryType::Liability),
            "equity" => Some(CategoryType::Equity),
            "expense" => Some(CategoryType::Expense),
            "cost" => Some(CategoryType::Cost),
            "income" => Some(CategoryType::Income),
            _ => None,
        }
    }

    /// Get the numeric prefix for this category type
    pub fn prefix(&self) -> &'static str {
        match self {
            CategoryType::Asset => "1",
            CategoryType::Liability => "2",
            CategoryType::Equity => "3",
            CategoryType::Expense => "4",
            CategoryType::Cost => "5",
            CategoryType::Income => "6",
        }
    }

    /// Check if this account type increases with debits
    pub fn is_debit_increase(&self) -> bool {
        matches!(
            self,
            CategoryType::Asset | CategoryType::Expense | CategoryType::Cost
        )
    }
}

/// Accounting entry type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    Manual,     // Manual entry
    Automatic,  // Generated automatically (e.g., from payroll)
    Adjustment, // Adjustment entry
}

impl EntryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryType::Manual => "manual",
            EntryType::Automatic => "automatic",
            EntryType::Adjustment => "adjustment",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "manual" => Some(EntryType::Manual),
            "automatic" => Some(EntryType::Automatic),
            "adjustment" => Some(EntryType::Adjustment),
            _ => None,
        }
    }
}

/// AccountCategory - represents an account in the chart of accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountCategory {
    pub id: String,
    pub code: String, // Account code (e.g., "10101", "621")
    pub name: String, // Account name (e.g., "Caja General")
    pub category_type: CategoryType,
    pub parent_id: Option<String>, // For hierarchical accounts
    pub balance: f64,              // Current balance
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AccountCategory {
    /// Create a new account category
    pub fn new(id: String, code: String, name: String, category_type: CategoryType) -> Self {
        let now = Utc::now();
        Self {
            id,
            code,
            name,
            category_type,
            parent_id: None,
            balance: 0.0,
            active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a child account
    pub fn with_parent(
        id: String,
        code: String,
        name: String,
        category_type: CategoryType,
        parent_id: String,
    ) -> Self {
        let mut account = Self::new(id, code, name, category_type);
        account.parent_id = Some(parent_id);
        account
    }

    /// Update balance
    pub fn update_balance(&mut self, amount: f64) {
        self.balance += amount;
        self.updated_at = Utc::now();
    }

    /// Set balance (for initialization or corrections)
    pub fn set_balance(&mut self, balance: f64) {
        self.balance = balance;
        self.updated_at = Utc::now();
    }

    /// Deactivate account
    pub fn deactivate(&mut self) {
        self.active = false;
        self.updated_at = Utc::now();
    }

    /// Activate account
    pub fn activate(&mut self) {
        self.active = true;
        self.updated_at = Utc::now();
    }

    /// Check if account is a parent (has children)
    pub fn is_parent(&self) -> bool {
        self.parent_id.is_none() && self.code.len() <= 3
    }

    /// Get display code (with dashes for sub-accounts)
    pub fn display_code(&self) -> String {
        if self.code.len() > 3 {
            format!(
                "{}-{}",
                &self.code[..self.code.len() - 2],
                &self.code[self.code.len() - 2..]
            )
        } else {
            self.code.clone()
        }
    }
}

/// AccountingEntry - represents a journal entry in the general ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingEntry {
    pub id: String,
    pub date: DateTime<Utc>,
    pub reference: String,      // Entry reference (e.g., "AS-001")
    pub description: String,    // Entry description
    pub debit_account: String,  // Debit account ID
    pub credit_account: String, // Credit account ID
    pub amount: f64,            // Amount (must balance)
    pub entry_type: EntryType,
    pub related_id: Option<String>, // Related entity ID (e.g., payroll_run_id)
    pub related_type: Option<String>, // Related entity type
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

impl AccountingEntry {
    /// Create a new accounting entry
    pub fn new(
        id: String,
        date: DateTime<Utc>,
        reference: String,
        description: String,
        debit_account: String,
        credit_account: String,
        amount: f64,
        created_by: String,
    ) -> Self {
        Self {
            id,
            date,
            reference,
            description,
            debit_account,
            credit_account,
            amount,
            entry_type: EntryType::Manual,
            related_id: None,
            related_type: None,
            created_at: Utc::now(),
            created_by,
        }
    }

    /// Create an automatic entry (from payroll, invoice, etc.)
    pub fn automatic(
        id: String,
        date: DateTime<Utc>,
        reference: String,
        description: String,
        debit_account: String,
        credit_account: String,
        amount: f64,
        created_by: String,
        related_id: String,
        related_type: String,
    ) -> Self {
        let mut entry = Self::new(
            id,
            date,
            reference,
            description,
            debit_account,
            credit_account,
            amount,
            created_by,
        );
        entry.entry_type = EntryType::Automatic;
        entry.related_id = Some(related_id);
        entry.related_type = Some(related_type);
        entry
    }

    /// Create an adjustment entry
    pub fn adjustment(
        id: String,
        date: DateTime<Utc>,
        reference: String,
        description: String,
        debit_account: String,
        credit_account: String,
        amount: f64,
        created_by: String,
    ) -> Self {
        let mut entry = Self::new(
            id,
            date,
            reference,
            description,
            debit_account,
            credit_account,
            amount,
            created_by,
        );
        entry.entry_type = EntryType::Adjustment;
        entry
    }

    /// Validate entry (debit != credit, amount > 0)
    pub fn is_valid(&self) -> bool {
        self.debit_account != self.credit_account && self.amount > 0.0
    }

    /// Get display string for the entry
    pub fn display(&self) -> String {
        format!(
            "{} - {} - S/ {:.2}",
            self.reference, self.description, self.amount
        )
    }
}

/// Generate reference for accounting entry
pub fn generate_reference(prefix: &str, number: u32) -> String {
    format!("{}-{:04}", prefix, number)
}

/// Trial balance structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalance {
    pub account_id: String,
    pub account_code: String,
    pub account_name: String,
    pub debit_balance: f64,
    pub credit_balance: f64,
    pub balance_type: String, // "debit" or "credit"
}

impl TrialBalance {
    pub fn new(account: &AccountCategory) -> Self {
        let (debit, credit, balance_type) = if account.category_type.is_debit_increase() {
            if account.balance >= 0.0 {
                (account.balance, 0.0, "debit")
            } else {
                (0.0, account.balance.abs(), "credit")
            }
        } else {
            if account.balance >= 0.0 {
                (0.0, account.balance, "credit")
            } else {
                (account.balance.abs(), 0.0, "debit")
            }
        };

        Self {
            account_id: account.id.clone(),
            account_code: account.code.clone(),
            account_name: account.name.clone(),
            debit_balance: debit,
            credit_balance: credit,
            balance_type: balance_type.to_string(),
        }
    }
}

/// Income statement structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeStatement {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_income: f64,
    pub total_expenses: f64,
    pub total_costs: f64,
    pub net_result: f64, // Income - Expenses - Costs
}

impl IncomeStatement {
    pub fn calculate(income: f64, expenses: f64, costs: f64) -> Self {
        Self {
            period_start: Utc::now(),
            period_end: Utc::now(),
            total_income: income,
            total_expenses: expenses,
            total_costs: costs,
            net_result: income - expenses - costs,
        }
    }

    pub fn is_profitable(&self) -> bool {
        self.net_result > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_category_creation() {
        let account = AccountCategory::new(
            "acc-id".to_string(),
            "10101".to_string(),
            "Caja General".to_string(),
            CategoryType::Asset,
        );

        assert_eq!(account.code, "10101");
        assert_eq!(account.name, "Caja General");
        assert!(account.active);
    }

    #[test]
    fn test_display_code() {
        let account = AccountCategory::new(
            "acc-id".to_string(),
            "10101".to_string(),
            "Caja".to_string(),
            CategoryType::Asset,
        );

        assert_eq!(account.display_code(), "101-01");
    }

    #[test]
    fn test_accounting_entry_validation() {
        let entry = AccountingEntry::new(
            "entry-id".to_string(),
            Utc::now(),
            "AS-001".to_string(),
            "Test entry".to_string(),
            "account-1".to_string(),
            "account-2".to_string(),
            1000.0,
            "user-id".to_string(),
        );

        assert!(entry.is_valid());
    }

    #[test]
    fn test_entry_invalid_same_accounts() {
        let entry = AccountingEntry::new(
            "entry-id".to_string(),
            Utc::now(),
            "AS-001".to_string(),
            "Test entry".to_string(),
            "account-1".to_string(),
            "account-1".to_string(),
            1000.0,
            "user-id".to_string(),
        );

        assert!(!entry.is_valid());
    }

    #[test]
    fn test_generate_reference() {
        let ref1 = generate_reference("AS", 1);
        let ref2 = generate_reference("AS", 42);

        assert_eq!(ref1, "AS-0001");
        assert_eq!(ref2, "AS-0042");
    }

    #[test]
    fn test_income_statement_profitable() {
        let is = IncomeStatement::calculate(10000.0, 6000.0, 2000.0);

        assert!(is.is_profitable());
        assert_eq!(is.net_result, 2000.0);
    }

    #[test]
    fn test_income_statement_not_profitable() {
        let is = IncomeStatement::calculate(5000.0, 6000.0, 1000.0);

        assert!(!is.is_profitable());
        assert_eq!(is.net_result, -2000.0);
    }

    #[test]
    fn test_category_type_is_debit_increase() {
        assert!(CategoryType::Asset.is_debit_increase());
        assert!(CategoryType::Expense.is_debit_increase());
        assert!(CategoryType::Cost.is_debit_increase());
        assert!(!CategoryType::Liability.is_debit_increase());
        assert!(!CategoryType::Equity.is_debit_increase());
        assert!(!CategoryType::Income.is_debit_increase());
    }
}
