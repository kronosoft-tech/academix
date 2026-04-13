//! Accounting Commands - Tauri Command Handlers
//!
//! Expose accounting operations to the frontend.

use crate::application::dto::accounting::{
    AccountCategoryDto, AccountCategoryTreeNode, AccountingEntryDto, AccountingSummary,
    CreateEntryRequest, IncomeStatementDto, TrialBalanceDto,
};
use crate::application::use_cases::AccountingService;
use crate::infrastructure::repositories::{
    InMemoryAccountCategoryRepository, InMemoryAccountingEntryRepository,
};
use tauri::command;

/// Create a new accounting entry
#[command]
pub fn create_entry(
    request: CreateEntryRequest,
    created_by: String,
) -> Result<AccountingEntryDto, String> {
    let entry_repo = InMemoryAccountingEntryRepository::new();
    let category_repo = InMemoryAccountCategoryRepository::new();

    let service = AccountingService::new(entry_repo, category_repo);
    service.create_entry(request, created_by)
}

/// Get accounting entry by ID
#[command]
pub fn get_entry(id: String) -> Result<Option<AccountingEntryDto>, String> {
    let entry_repo = InMemoryAccountingEntryRepository::new();
    let category_repo = InMemoryAccountCategoryRepository::new();

    let service = AccountingService::new(entry_repo, category_repo);
    service.get_entry(&id)
}

/// List accounting entries with filters
#[command]
pub fn list_entries(
    date_from: Option<String>,
    date_to: Option<String>,
    entry_type: Option<String>,
) -> Result<Vec<AccountingEntryDto>, String> {
    use crate::domain::entities::accounting::EntryType;

    let type_filter = entry_type.and_then(|t| EntryType::from_str(&t));

    let entry_repo = InMemoryAccountingEntryRepository::new();
    let category_repo = InMemoryAccountCategoryRepository::new();

    let service = AccountingService::new(entry_repo, category_repo);
    service.list_entries(date_from.as_deref(), date_to.as_deref(), type_filter)
}

/// Get trial balance
#[command]
pub fn get_trial_balance(as_of_date: String) -> Result<TrialBalanceDto, String> {
    let entry_repo = InMemoryAccountingEntryRepository::new();
    let category_repo = InMemoryAccountCategoryRepository::new();

    let service = AccountingService::new(entry_repo, category_repo);
    service.get_trial_balance(&as_of_date)
}

/// Get income statement
#[command]
pub fn get_income_statement(
    period_start: String,
    period_end: String,
) -> Result<IncomeStatementDto, String> {
    let entry_repo = InMemoryAccountingEntryRepository::new();
    let category_repo = InMemoryAccountCategoryRepository::new();

    let service = AccountingService::new(entry_repo, category_repo);
    service.get_income_statement(&period_start, &period_end)
}

/// List account categories (chart of accounts)
#[command]
pub fn list_accounts(
    category_type: Option<String>,
    active_only: bool,
) -> Result<Vec<AccountCategoryDto>, String> {
    use crate::domain::entities::accounting::CategoryType;

    let type_filter = category_type.and_then(|t| CategoryType::from_str(&t));

    let entry_repo = InMemoryAccountingEntryRepository::new();
    let category_repo = InMemoryAccountCategoryRepository::new();

    let service = AccountingService::new(entry_repo, category_repo);
    service.list_accounts(type_filter, active_only)
}

/// Get account categories as tree
#[command]
pub fn get_account_tree() -> Result<Vec<AccountCategoryTreeNode>, String> {
    let entry_repo = InMemoryAccountingEntryRepository::new();
    let category_repo = InMemoryAccountCategoryRepository::new();

    let service = AccountingService::new(entry_repo, category_repo);
    service.get_account_tree()
}

/// Get accounting summary for dashboard
#[command]
pub fn get_accounting_summary() -> Result<AccountingSummary, String> {
    let entry_repo = InMemoryAccountingEntryRepository::new();
    let category_repo = InMemoryAccountCategoryRepository::new();

    let service = AccountingService::new(entry_repo, category_repo);
    service.get_summary()
}
