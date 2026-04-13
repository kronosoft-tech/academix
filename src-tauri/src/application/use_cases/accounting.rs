//! Accounting Service
//!
//! Use case for accounting operations - general ledger, trial balance, income statement.

use crate::application::dto::accounting::{
    AccountCategoryDto, AccountCategoryTreeNode, AccountingEntryDto, AccountingSummary,
    CategoryTotalDto, CreateEntryRequest, IncomeStatementDto, TrialBalanceAccountDto,
    TrialBalanceDto,
};
use crate::application::ports::accounting::{AccountCategoryRepository, AccountingEntryRepository};
use crate::domain::entities::accounting::{
    AccountCategory, AccountingEntry, CategoryType, EntryType,
};
use chrono::{DateTime, Utc};

/// Accounting service - orchestrates accounting operations
pub struct AccountingService<R: AccountingEntryRepository, C: AccountCategoryRepository> {
    entry_repo: R,
    category_repo: C,
}

impl<R: AccountingEntryRepository, C: AccountCategoryRepository> AccountingService<R, C> {
    pub fn new(entry_repo: R, category_repo: C) -> Self {
        Self {
            entry_repo,
            category_repo,
        }
    }

    /// Create a new accounting entry
    pub fn create_entry(
        &self,
        request: CreateEntryRequest,
        created_by: String,
    ) -> Result<AccountingEntryDto, String> {
        // Validate: debit != credit
        if request.debit_account == request.credit_account {
            return Err("Debit and credit accounts must be different".to_string());
        }
        // Validate: amount > 0
        if request.amount <= 0.0 {
            return Err("Amount must be greater than 0".to_string());
        }

        // Verify accounts exist
        let debit_acc = self
            .category_repo
            .get_by_id(&request.debit_account)?
            .ok_or_else(|| format!("Debit account not found: {}", request.debit_account))?;
        let credit_acc = self
            .category_repo
            .get_by_id(&request.credit_account)?
            .ok_or_else(|| format!("Credit account not found: {}", request.credit_account))?;

        // Parse date
        let date = DateTime::parse_from_rfc3339(&request.date)
            .map_err(|e| format!("Invalid date: {}", e))?
            .with_timezone(&Utc);

        // Generate reference if not provided
        let reference = request.reference.unwrap_or_else(|| {
            self.entry_repo
                .get_next_reference("AS")
                .map(|n| format!("AS-{:04}", n))
                .unwrap_or_else(|_| "AS-0001".to_string())
        });

        let entry = AccountingEntry::new(
            String::new(),
            date,
            reference,
            request.description,
            request.debit_account.clone(),
            request.credit_account.clone(),
            request.amount,
            created_by,
        );

        let created = self.entry_repo.create(entry)?;

        // Update account balances
        // Debit increases asset/expense/cost, decreases liability/equity/income
        // Credit increases liability/equity/income, decreases asset/expense/cost
        self.update_account_balance(&request.debit_account, request.amount, true)?;
        self.update_account_balance(&request.credit_account, request.amount, false)?;

        // Build DTO with account names
        let mut dto = AccountingEntryDto::from(created);
        dto.debit_account_name = debit_acc.name;
        dto.credit_account_name = credit_acc.name;

        Ok(dto)
    }

    /// Update account balance after entry
    fn update_account_balance(
        &self,
        account_id: &str,
        amount: f64,
        is_debit: bool,
    ) -> Result<(), String> {
        let category = self
            .category_repo
            .get_by_id(account_id)?
            .ok_or_else(|| format!("Account not found: {}", account_id))?;

        let balance_change = if category.category_type.is_debit_increase() {
            if is_debit {
                amount
            } else {
                -amount
            }
        } else {
            if is_debit {
                -amount
            } else {
                amount
            }
        };

        self.category_repo
            .update_balance(account_id, balance_change)
    }

    /// Get accounting entry by ID
    pub fn get_entry(&self, id: &str) -> Result<Option<AccountingEntryDto>, String> {
        let entry = self.entry_repo.get_by_id(id)?;

        if let Some(entry) = entry {
            let debit_acc = self.category_repo.get_by_id(&entry.debit_account)?;
            let credit_acc = self.category_repo.get_by_id(&entry.credit_account)?;

            let mut dto = AccountingEntryDto::from(entry);
            if let Some(acc) = debit_acc {
                dto.debit_account_name = acc.name;
            }
            if let Some(acc) = credit_acc {
                dto.credit_account_name = acc.name;
            }
            Ok(Some(dto))
        } else {
            Ok(None)
        }
    }

    /// List accounting entries with filters
    pub fn list_entries(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
        entry_type: Option<EntryType>,
    ) -> Result<Vec<AccountingEntryDto>, String> {
        let entries = self.entry_repo.list(date_from, date_to, entry_type)?;

        let mut dtos = Vec::new();
        for entry in entries {
            let debit_acc = self.category_repo.get_by_id(&entry.debit_account)?;
            let credit_acc = self.category_repo.get_by_id(&entry.credit_account)?;

            let mut dto = AccountingEntryDto::from(entry);
            if let Some(acc) = debit_acc {
                dto.debit_account_name = acc.name;
            }
            if let Some(acc) = credit_acc {
                dto.credit_account_name = acc.name;
            }
            dtos.push(dto);
        }

        Ok(dtos)
    }

    /// Get trial balance (balance de comprobación)
    pub fn get_trial_balance(&self, _as_of_date: &str) -> Result<TrialBalanceDto, String> {
        // Get all active accounts
        let accounts = self.category_repo.list(None, true)?;

        let mut account_dtos = Vec::new();
        let mut total_debits = 0.0;
        let mut total_credits = 0.0;

        for account in accounts {
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

            if debit > 0.0 || credit > 0.0 {
                account_dtos.push(TrialBalanceAccountDto {
                    account_id: account.id.clone(),
                    account_code: account.code.clone(),
                    account_name: account.name.clone(),
                    debit_balance: debit,
                    credit_balance: credit,
                    balance_type: balance_type.to_string(),
                });
                total_debits += debit;
                total_credits += credit;
            }
        }

        // Sort by account code
        account_dtos.sort_by(|a, b| a.account_code.cmp(&b.account_code));

        Ok(TrialBalanceDto {
            as_of_date: Utc::now().to_rfc3339(),
            accounts: account_dtos,
            total_debits,
            total_credits,
            is_balanced: (total_debits - total_credits).abs() < 0.01,
        })
    }

    /// Get income statement (estado de resultados)
    pub fn get_income_statement(
        &self,
        period_start: &str,
        period_end: &str,
    ) -> Result<IncomeStatementDto, String> {
        // Get income and expense accounts
        let income_accounts = self
            .category_repo
            .get_balances_by_type(CategoryType::Income)?;
        let expense_accounts = self
            .category_repo
            .get_balances_by_type(CategoryType::Expense)?;
        let cost_accounts = self
            .category_repo
            .get_balances_by_type(CategoryType::Cost)?;

        let total_income: f64 = income_accounts.iter().map(|a| a.balance).sum();
        let total_expenses: f64 = expense_accounts.iter().map(|a| a.balance).sum();
        let total_costs: f64 = cost_accounts.iter().map(|a| a.balance).sum();

        let income_by_category: Vec<CategoryTotalDto> = income_accounts
            .into_iter()
            .map(|a| CategoryTotalDto {
                category_id: a.id,
                category_name: a.name,
                total: a.balance,
            })
            .collect();

        let expenses_by_category: Vec<CategoryTotalDto> = expense_accounts
            .into_iter()
            .chain(cost_accounts.into_iter())
            .map(|a| CategoryTotalDto {
                category_id: a.id,
                category_name: a.name,
                total: a.balance,
            })
            .collect();

        let net_result = total_income - total_expenses - total_costs;

        Ok(IncomeStatementDto {
            period_start: period_start.to_string(),
            period_end: period_end.to_string(),
            total_income,
            total_expenses,
            total_costs,
            net_result,
            is_profitable: net_result > 0.0,
            income_by_category,
            expenses_by_category,
        })
    }

    /// List account categories (chart of accounts)
    pub fn list_accounts(
        &self,
        category_type: Option<CategoryType>,
        active_only: bool,
    ) -> Result<Vec<AccountCategoryDto>, String> {
        let accounts = self.category_repo.list(category_type, active_only)?;
        Ok(accounts.into_iter().map(AccountCategoryDto::from).collect())
    }

    /// Get account categories as tree
    pub fn get_account_tree(&self) -> Result<Vec<AccountCategoryTreeNode>, String> {
        let root_accounts = self.category_repo.list_roots()?;

        let mut tree = Vec::new();
        for root in root_accounts {
            let children = self.get_children(&root.id)?;
            tree.push(AccountCategoryTreeNode {
                category: AccountCategoryDto::from(root),
                children,
            });
        }

        Ok(tree)
    }

    fn get_children(&self, parent_id: &str) -> Result<Vec<AccountCategoryTreeNode>, String> {
        let children = self.category_repo.list_children(parent_id)?;

        let mut result = Vec::new();
        for child in children {
            let grandchildren = self.get_children(&child.id)?;
            result.push(AccountCategoryTreeNode {
                category: AccountCategoryDto::from(child),
                children: grandchildren,
            });
        }

        Ok(result)
    }

    /// Get accounting summary for dashboard
    pub fn get_summary(&self) -> Result<AccountingSummary, String> {
        let entries = self.entry_repo.list(None, None, None)?;
        let accounts = self.category_repo.list(None, true)?;

        let total_debits: f64 = entries.iter().map(|e| e.amount).sum();
        let total_credits: f64 = entries.iter().map(|e| e.amount).sum();
        let entry_count = entries.len() as i64;

        // Get recent entries (last 10)
        let recent: Vec<AccountingEntryDto> = entries
            .into_iter()
            .take(10)
            .map(|e| {
                let dto = AccountingEntryDto::from(e);
                // Could add account names here if needed
                dto
            })
            .collect();

        Ok(AccountingSummary {
            total_debits,
            total_credits,
            account_count: accounts.len() as i64,
            entry_count,
            recent_entries: recent,
        })
    }
}
