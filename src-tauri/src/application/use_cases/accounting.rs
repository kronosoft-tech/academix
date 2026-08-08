//! Accounting Service - Simplified income/expense model
//!
//! Use case for accounting operations.

use crate::application::dto::accounting::{
    AccountingEntryDto, AccountingSummaryDto, CategoryTotalDto, CreateEntryRequest,
    MonthlyDataPointDto,
};
use crate::application::ports::accounting::AccountingEntryRepository;
use crate::domain::entities::accounting::{AccountingCategory, AccountingEntry, EntryType};
use uuid::Uuid;

/// Accounting service - orchestrates accounting operations
pub struct AccountingService<R: AccountingEntryRepository> {
    entry_repo: R,
}

impl<R: AccountingEntryRepository> AccountingService<R> {
    pub fn new(entry_repo: R) -> Self {
        Self { entry_repo }
    }

    /// Create a new accounting entry
    pub async fn create_entry(
        &self,
        request: CreateEntryRequest,
    ) -> Result<AccountingEntryDto, String> {
        // Validate amount > 0
        if request.amount <= 0.0 {
            return Err("Amount must be greater than 0".to_string());
        }

        // Validate description not empty
        if request.description.trim().is_empty() {
            return Err("Description is required".to_string());
        }

        // Parse entry type
        let entry_type = EntryType::from_str(&request.entry_type)
            .ok_or_else(|| format!("Invalid entry type: {}", request.entry_type))?;

        // Parse category based on entry type
        let category =
            AccountingCategory::from_str(&request.category, &entry_type).ok_or_else(|| {
                format!(
                    "Invalid category '{}' for {}",
                    request.category, request.entry_type
                )
            })?;

        // Generate UUID v7 for the entry
        let ts = uuid::Timestamp::now(uuid::NoContext);
        let entry_id = Uuid::new_v7(ts).to_string();

        // Generate reference if not provided
        let reference = if request.reference.as_deref().unwrap_or("").is_empty() {
            self.entry_repo
                .get_next_reference("AS")
                .await
                .map(|n| format!("AS-{:04}", n))
                .ok()
        } else {
            request.reference.clone()
        };

        let entry = AccountingEntry {
            id: entry_id,
            date: request.date,
            entry_type,
            category,
            description: request.description,
            amount: request.amount,
            reference,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let created = self.entry_repo.create(entry).await?;

        Ok(AccountingEntryDto::from(created))
    }

    /// Get accounting entry by ID
    pub async fn get_entry(&self, id: &str) -> Result<Option<AccountingEntryDto>, String> {
        let entry = self.entry_repo.get_by_id(id).await?;
        Ok(entry.map(AccountingEntryDto::from))
    }

    /// List accounting entries with filters
    pub async fn list_entries(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
        entry_type: Option<EntryType>,
    ) -> Result<Vec<AccountingEntryDto>, String> {
        let entries = self.entry_repo.list(date_from, date_to, entry_type).await?;
        Ok(entries.into_iter().map(AccountingEntryDto::from).collect())
    }

    /// Delete accounting entry
    pub async fn delete_entry(&self, id: &str) -> Result<bool, String> {
        self.entry_repo.delete(id).await
    }

    /// Get accounting summary for dashboard
    pub async fn get_summary(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<AccountingSummaryDto, String> {
        // Default to last 6 months if no dates provided
        let (start_date, end_date) = if let (Some(from), Some(to)) = (date_from, date_to) {
            (from.to_string(), to.to_string())
        } else {
            let now = chrono::Utc::now();
            let six_months_ago = now - chrono::Duration::days(180);
            let start = six_months_ago.format("%Y-%m-01").to_string();
            let end = now.format("%Y-%m-%d").to_string();
            (start, end)
        };

        let entries = self.entry_repo.list(None, None, None).await?;

        // Filter by date range in-memory (repo loads all, we narrow here)
        let entries: Vec<_> = entries
            .into_iter()
            .filter(|e| {
                let d = if e.date.len() >= 10 {
                    &e.date[..10]
                } else {
                    e.date.as_str()
                };
                d >= start_date.as_str() && d <= end_date.as_str()
            })
            .collect();

        // Calculate totals
        let total_income: f64 = entries
            .iter()
            .filter(|e| e.entry_type == EntryType::Income)
            .map(|e| e.amount)
            .sum();

        let total_expenses: f64 = entries
            .iter()
            .filter(|e| e.entry_type == EntryType::Expense)
            .map(|e| e.amount)
            .sum();

        let net_balance = total_income - total_expenses;
        let entry_count = entries.len() as i64;

        // Build monthly data (last 6 months)
        let months = [
            "Ene", "Feb", "Mar", "Abr", "May", "Jun", "Jul", "Ago", "Sep", "Oct", "Nov", "Dic",
        ];
        let mut monthly_income: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        let mut monthly_expenses: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();

        for entry in &entries {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(&entry.date, "%Y-%m-%d") {
                let month_idx = date.format("%m").to_string().parse::<usize>().unwrap_or(1) - 1;
                let month_label = months.get(month_idx).unwrap_or(&"???").to_string();

                match entry.entry_type {
                    EntryType::Income => {
                        *monthly_income.entry(month_label).or_insert(0.0) += entry.amount;
                    }
                    EntryType::Expense => {
                        *monthly_expenses.entry(month_label).or_insert(0.0) += entry.amount;
                    }
                }
            }
        }

        let now = chrono::Utc::now();
        let monthly_data: Vec<MonthlyDataPointDto> = (0..6)
            .rev()
            .map(|i| {
                let date = now - chrono::Duration::days(30 * i);
                let idx = date.format("%m").to_string().parse::<usize>().unwrap_or(1) - 1;
                let m = months.get(idx).unwrap_or(&"???").to_string();
                MonthlyDataPointDto {
                    month: m.clone(),
                    income: *monthly_income.get(&m).unwrap_or(&0.0),
                    expenses: *monthly_expenses.get(&m).unwrap_or(&0.0),
                }
            })
            .collect();

        // Build expense breakdown by category
        let mut expenses_by_cat: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        for entry in entries
            .iter()
            .filter(|e| e.entry_type == EntryType::Expense)
        {
            let cat_name = entry.category.display_name().to_string();
            *expenses_by_cat.entry(cat_name).or_insert(0.0) += entry.amount;
        }

        let mut expenses_by_category: Vec<CategoryTotalDto> = expenses_by_cat
            .into_iter()
            .map(|(name, amount)| CategoryTotalDto {
                category_name: name,
                amount,
            })
            .collect();
        expenses_by_category.sort_by(|a, b| {
            b.amount
                .partial_cmp(&a.amount)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Build income breakdown by category
        let mut income_by_cat: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        for entry in entries.iter().filter(|e| e.entry_type == EntryType::Income) {
            let cat_name = entry.category.display_name().to_string();
            *income_by_cat.entry(cat_name).or_insert(0.0) += entry.amount;
        }

        let mut income_by_category: Vec<CategoryTotalDto> = income_by_cat
            .into_iter()
            .map(|(name, amount)| CategoryTotalDto {
                category_name: name,
                amount,
            })
            .collect();
        income_by_category.sort_by(|a, b| {
            b.amount
                .partial_cmp(&a.amount)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Get recent entries (last 10)
        let recent_entries: Vec<AccountingEntryDto> = entries
            .into_iter()
            .take(10)
            .map(AccountingEntryDto::from)
            .collect();

        Ok(AccountingSummaryDto {
            total_income,
            total_expenses,
            net_balance,
            entry_count,
            recent_entries,
            monthly_data,
            expenses_by_category,
            income_by_category,
        })
    }
}
