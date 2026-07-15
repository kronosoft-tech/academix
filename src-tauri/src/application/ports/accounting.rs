//! Accounting Repository Ports
//!
//! Port interfaces for simplified accounting operations.

use crate::domain::entities::accounting::{AccountingEntry, EntryType};

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

    /// Delete accounting entry
    fn delete(&self, id: &str) -> Result<bool, String>;

    /// Get total income in date range
    fn get_total_income(&self, date_from: &str, date_to: &str) -> Result<f64, String>;

    /// Get total expenses in date range
    fn get_total_expenses(&self, date_from: &str, date_to: &str) -> Result<f64, String>;

    /// Get next entry reference number
    fn get_next_reference(&self, prefix: &str) -> Result<u32, String>;
}
