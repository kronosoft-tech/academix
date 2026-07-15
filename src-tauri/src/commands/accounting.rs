//! Accounting Commands - Simplified income/expense model
//!
//! Expose accounting operations to the frontend.

use crate::application::dto::accounting::{AccountingEntryDto, AccountingSummaryDto, CreateEntryRequest};
use crate::application::use_cases::AccountingService;
use crate::infrastructure::repositories::SqliteAccountingEntryRepository;
use tauri::command;
use tauri::State;

/// Type alias for Accounting Service with SQLite repository
pub type AccountingServiceState = AccountingService<SqliteAccountingEntryRepository>;

/// Create a new accounting entry (income or expense)
#[command]
pub fn create_entry(
    state: State<AccountingServiceState>,
    request: CreateEntryRequest,
) -> Result<AccountingEntryDto, String> {
    state.create_entry(request)
}

/// Get accounting entry by ID
#[command]
pub fn get_entry(
    state: State<AccountingServiceState>,
    id: String,
) -> Result<Option<AccountingEntryDto>, String> {
    state.get_entry(&id)
}

/// List accounting entries with filters
#[command]
pub fn list_entries(
    state: State<AccountingServiceState>,
    date_from: Option<String>,
    date_to: Option<String>,
    entry_type: Option<String>,
) -> Result<Vec<AccountingEntryDto>, String> {
    use crate::domain::entities::accounting::EntryType;

    let type_filter = entry_type.and_then(|t| EntryType::from_str(&t));
    state.list_entries(date_from.as_deref(), date_to.as_deref(), type_filter)
}

/// Delete accounting entry
#[command]
pub fn delete_entry(
    state: State<AccountingServiceState>,
    id: String,
) -> Result<bool, String> {
    state.delete_entry(&id)
}

/// Get accounting summary for dashboard
#[command]
pub fn get_accounting_summary(
    state: State<AccountingServiceState>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<AccountingSummaryDto, String> {
    state.get_summary(date_from.as_deref(), date_to.as_deref())
}
