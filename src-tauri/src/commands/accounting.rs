//! Accounting Commands - Tauri Command Handlers
//!
//! Expose accounting operations to the frontend.

use crate::application::dto::accounting::{
    AccountCategoryDto, AccountCategoryTreeNode, AccountingEntryDto, AccountingSummary,
    CreateEntryRequest, FinancialBalanceDto, IncomeStatementDto, TrialBalanceDto,
};
use crate::application::use_cases::AccountingService;
use crate::infrastructure::repositories::{
    SqliteAccountCategoryRepository, SqliteAccountingEntryRepository,
};
use tauri::command;
use tauri::State;

/// Type alias for Accounting Service with SQLite repositories
pub type AccountingServiceState =
    AccountingService<SqliteAccountingEntryRepository, SqliteAccountCategoryRepository>;

/// Create a new accounting entry
#[command]
pub fn create_entry(
    state: State<AccountingServiceState>,
    request: CreateEntryRequest,
    created_by: String,
) -> Result<AccountingEntryDto, String> {
    println!(
        "[DEBUG] create_entry called: debit={}, credit={}, amount={}",
        request.debit_account, request.credit_account, request.amount
    );
    state.create_entry(request, created_by)
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

/// Get trial balance
#[command]
pub fn get_trial_balance(
    state: State<AccountingServiceState>,
    as_of_date: String,
) -> Result<TrialBalanceDto, String> {
    state.get_trial_balance(&as_of_date)
}

/// Get income statement
#[command]
pub fn get_income_statement(
    state: State<AccountingServiceState>,
    period_start: String,
    period_end: String,
) -> Result<IncomeStatementDto, String> {
    state.get_income_statement(&period_start, &period_end)
}

/// List account categories (chart of accounts)
#[command]
pub fn list_accounts(
    state: State<AccountingServiceState>,
    category_type: Option<String>,
    active_only: bool,
) -> Result<Vec<AccountCategoryDto>, String> {
    use crate::domain::entities::accounting::CategoryType;

    let type_filter = category_type.and_then(|t| CategoryType::from_str(&t));
    state.list_accounts(type_filter, active_only)
}

/// Get account categories as tree
#[command]
pub fn get_account_tree(
    state: State<AccountingServiceState>,
) -> Result<Vec<AccountCategoryTreeNode>, String> {
    state.get_account_tree()
}

/// Get accounting summary for dashboard
#[command]
pub fn get_accounting_summary(
    state: State<AccountingServiceState>,
) -> Result<AccountingSummary, String> {
    state.get_summary()
}

/// Get financial balance (Balance Financiero)
#[command]
pub fn get_financial_balance(
    state: State<AccountingServiceState>,
    as_of_date: String,
) -> Result<FinancialBalanceDto, String> {
    state.get_financial_balance(&as_of_date)
}
