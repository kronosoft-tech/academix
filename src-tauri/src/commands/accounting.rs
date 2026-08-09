//! Accounting Commands - Simplified income/expense model
//!
//! Expose accounting operations to the frontend.

use crate::application::dto::accounting::{AccountingEntryDto, AccountingSummaryDto, CreateEntryRequest};
use crate::application::use_cases::AccountingService;
use crate::infrastructure::repositories::MemoryBackedAccountingEntryRepository;
use tauri::command;
use tauri::State;

/// Type alias for Accounting Service with SQLite repository
pub type AccountingServiceState = AccountingService<MemoryBackedAccountingEntryRepository>;

/// Create a new accounting entry (income or expense)
#[command]
pub async fn create_entry(
    state: State<'_, AccountingServiceState>,
    request: CreateEntryRequest,
) -> Result<AccountingEntryDto, String> {
    state.create_entry(request).await
}

/// Get accounting entry by ID
#[command]
pub async fn get_entry(
    state: State<'_, AccountingServiceState>,
    id: String,
) -> Result<Option<AccountingEntryDto>, String> {
    state.get_entry(&id).await
}

/// List accounting entries with filters
#[command]
pub async fn list_entries(
    state: State<'_, AccountingServiceState>,
    date_from: Option<String>,
    date_to: Option<String>,
    entry_type: Option<String>,
) -> Result<Vec<AccountingEntryDto>, String> {
    use crate::domain::entities::accounting::EntryType;

    let type_filter = entry_type.and_then(|t| EntryType::from_str(&t));
    state.list_entries(date_from.as_deref(), date_to.as_deref(), type_filter).await
}

/// Delete accounting entry
#[command]
pub async fn delete_entry(
    state: State<'_, AccountingServiceState>,
    id: String,
) -> Result<bool, String> {
    state.delete_entry(&id).await
}

/// Get accounting summary for dashboard
#[command]
pub async fn get_accounting_summary(
    state: State<'_, AccountingServiceState>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<AccountingSummaryDto, String> {
    state.get_summary(date_from.as_deref(), date_to.as_deref()).await
}
