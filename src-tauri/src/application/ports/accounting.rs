//! Accounting Repository Ports
//!
//! Port interfaces for accounting operations.

use crate::domain::entities::accounting::{
    AccountCategory, AccountingEntry, CategoryType, EntryType,
};

/// Account category repository trait (port)
pub trait AccountCategoryRepository: Send + Sync {
    /// Create a new account category
    fn create(&self, category: AccountCategory) -> Result<AccountCategory, String>;

    /// Get account category by ID
    fn get_by_id(&self, id: &str) -> Result<Option<AccountCategory>, String>;

    /// Get account category by code
    fn get_by_code(&self, code: &str) -> Result<Option<AccountCategory>, String>;

    /// List all account categories
    fn list(
        &self,
        category_type: Option<CategoryType>,
        active_only: bool,
    ) -> Result<Vec<AccountCategory>, String>;

    /// List root accounts (no parent)
    fn list_roots(&self) -> Result<Vec<AccountCategory>, String>;

    /// List children of a parent account
    fn list_children(&self, parent_id: &str) -> Result<Vec<AccountCategory>, String>;

    /// Update account category
    fn update(&self, category: AccountCategory) -> Result<AccountCategory, String>;

    /// Update account balance
    fn update_balance(&self, id: &str, amount: f64) -> Result<(), String>;

    /// Delete account category (soft delete - deactivate)
    fn delete(&self, id: &str) -> Result<bool, String>;

    /// Get accounts by type with balances for trial balance
    fn get_balances_by_type(
        &self,
        category_type: CategoryType,
    ) -> Result<Vec<AccountCategory>, String>;
}

/// Accounting entry repository trait (port)
pub trait AccountingEntryRepository: Send + Sync {
    /// Create a new accounting entry
    fn create(&self, entry: AccountingEntry) -> Result<AccountingEntry, String>;

    /// Get accounting entry by ID
    fn get_by_id(&self, id: &str) -> Result<Option<AccountingEntry>, String>;

    /// List accounting entries with filters
    fn list(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
        entry_type: Option<EntryType>,
    ) -> Result<Vec<AccountingEntry>, String>;

    /// Get accounting entries by related entity
    fn get_by_related(
        &self,
        related_id: &str,
        related_type: &str,
    ) -> Result<Vec<AccountingEntry>, String>;

    /// Get entries by account (debit or credit)
    fn get_by_account(&self, account_id: &str) -> Result<Vec<AccountingEntry>, String>;

    /// Get entries by date range
    fn get_by_date_range(
        &self,
        date_from: &str,
        date_to: &str,
    ) -> Result<Vec<AccountingEntry>, String>;

    /// Update accounting entry
    fn update(&self, entry: AccountingEntry) -> Result<AccountingEntry, String>;

    /// Delete accounting entry
    fn delete(&self, id: &str) -> Result<bool, String>;

    /// Get total debits in date range
    fn get_total_debits(&self, date_from: &str, date_to: &str) -> Result<f64, String>;

    /// Get total credits in date range
    fn get_total_credits(&self, date_from: &str, date_to: &str) -> Result<f64, String>;

    /// Get next entry reference number
    fn get_next_reference(&self, prefix: &str) -> Result<u32, String>;
}
