//! Accounting SQLite Repository
//!
//! Implements AccountCategoryRepository and AccountingEntryRepository using SQLite.

use crate::application::ports::accounting::{AccountCategoryRepository, AccountingEntryRepository};
use crate::domain::entities::accounting::{
    AccountCategory, AccountingEntry, CategoryType, EntryType,
};
use crate::infrastructure::database::SqlitePool;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// SQLite implementation of AccountCategoryRepository
#[derive(Clone)]
pub struct SqliteAccountCategoryRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteAccountCategoryRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    fn row_to_account_category(row: &rusqlite::Row<'_>) -> rusqlite::Result<AccountCategory> {
        let category_type_str: String = row.get(3)?;
        let active_i32: i32 = row.get(6)?;
        let created_str: String = row.get(7)?;
        let updated_str: String = row.get(8)?;

        let category_type =
            CategoryType::from_str(&category_type_str).unwrap_or(CategoryType::Asset);

        Ok(AccountCategory {
            id: row.get(0)?,
            code: row.get(1)?,
            name: row.get(2)?,
            category_type,
            parent_id: row.get(4)?,
            balance: row.get(5)?,
            active: active_i32 != 0,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    fn category_type_to_string(ct: CategoryType) -> &'static str {
        ct.as_str()
    }
}

impl AccountCategoryRepository for SqliteAccountCategoryRepository {
    fn create(&self, category: AccountCategory) -> Result<AccountCategory, String> {
        let sql = "INSERT INTO account_categories (
                      id, code, name, category_type, parent_id, balance, active, created_at, updated_at
                  ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";

        self.pool
            .execute(
                sql,
                &[
                    &category.id,
                    &category.code,
                    &category.name,
                    &Self::category_type_to_string(category.category_type),
                    &category.parent_id,
                    &category.balance.to_string(),
                    &(if category.active { 1 } else { 0 }).to_string(),
                    &category.created_at.to_rfc3339(),
                    &category.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| e.to_string())?;

        Ok(category)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<AccountCategory>, String> {
        let sql = "SELECT id, code, name, category_type, parent_id, balance, active, created_at, updated_at
                  FROM account_categories WHERE id = ?";

        self.pool
            .query_row(sql, &[&id], Self::row_to_account_category)
            .map_err(|e| e.to_string())
    }

    fn get_by_code(&self, code: &str) -> Result<Option<AccountCategory>, String> {
        let sql = "SELECT id, code, name, category_type, parent_id, balance, active, created_at, updated_at
                  FROM account_categories WHERE code = ?";

        self.pool
            .query_row(sql, &[&code], Self::row_to_account_category)
            .map_err(|e| e.to_string())
    }

    fn list(
        &self,
        category_type: Option<CategoryType>,
        active_only: bool,
    ) -> Result<Vec<AccountCategory>, String> {
        let mut sql = "SELECT id, code, name, category_type, parent_id, balance, active, created_at, updated_at
                    FROM account_categories WHERE 1=1".to_string();
        let mut params: Vec<String> = Vec::new();

        if let Some(ct) = category_type {
            sql.push_str(" AND category_type = ?");
            params.push(Self::category_type_to_string(ct).to_string());
        }

        if active_only {
            sql.push_str(" AND active = 1");
        }

        sql.push_str(" ORDER BY code");

        self.pool
            .query(&sql, &[], Self::row_to_account_category)
            .map_err(|e| e.to_string())
    }

    fn list_roots(&self) -> Result<Vec<AccountCategory>, String> {
        let sql = "SELECT id, code, name, category_type, parent_id, balance, active, created_at, updated_at
                  FROM account_categories WHERE parent_id IS NULL ORDER BY code";

        self.pool
            .query(sql, &[], Self::row_to_account_category)
            .map_err(|e| e.to_string())
    }

    fn list_children(&self, parent_id: &str) -> Result<Vec<AccountCategory>, String> {
        let sql = "SELECT id, code, name, category_type, parent_id, balance, active, created_at, updated_at
                  FROM account_categories WHERE parent_id = ? ORDER BY code";

        self.pool
            .query(sql, &[&parent_id], Self::row_to_account_category)
            .map_err(|e| e.to_string())
    }

    fn update(&self, category: AccountCategory) -> Result<AccountCategory, String> {
        let sql = "UPDATE account_categories 
                  SET code = ?, name = ?, category_type = ?, parent_id = ?, balance = ?, active = ?, updated_at = ?
                  WHERE id = ?";

        let affected = self
            .pool
            .execute(
                sql,
                &[
                    &category.code,
                    &category.name,
                    &Self::category_type_to_string(category.category_type),
                    &category.parent_id,
                    &category.balance.to_string(),
                    &(if category.active { 1 } else { 0 }).to_string(),
                    &Utc::now().to_rfc3339(),
                    &category.id,
                ],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("AccountCategory not found: {}", category.id));
        }

        Ok(category)
    }

    fn update_balance(&self, id: &str, amount: f64) -> Result<(), String> {
        let sql =
            "UPDATE account_categories SET balance = balance + ?, updated_at = ? WHERE id = ?";

        self.pool
            .execute(sql, &[&amount.to_string(), &Utc::now().to_rfc3339(), &id])
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let sql = "UPDATE account_categories SET active = 0, updated_at = ? WHERE id = ?";

        let affected = self
            .pool
            .execute(sql, &[&Utc::now().to_rfc3339(), &id])
            .map_err(|e| e.to_string())?;

        Ok(affected > 0)
    }

    fn get_balances_by_type(
        &self,
        category_type: CategoryType,
    ) -> Result<Vec<AccountCategory>, String> {
        let sql = "SELECT id, code, name, category_type, parent_id, balance, active, created_at, updated_at
                  FROM account_categories 
                  WHERE category_type = ? AND active = 1
                  ORDER BY code";

        self.pool
            .query(
                &sql,
                &[&Self::category_type_to_string(category_type).to_string()],
                Self::row_to_account_category,
            )
            .map_err(|e| e.to_string())
    }
}

/// SQLite implementation of AccountingEntryRepository
#[derive(Clone)]
pub struct SqliteAccountingEntryRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteAccountingEntryRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    fn row_to_accounting_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<AccountingEntry> {
        // SQL: id, date, reference, description, debit_account, credit_account, amount,
        //      entry_type, related_id, related_type, created_by, created_at
        let entry_type_str: String = row.get(7)?;
        let date_str: String = row.get(1)?;
        let created_str: String = row.get(11)?;

        let entry_type = EntryType::from_str(&entry_type_str).unwrap_or(EntryType::Manual);

        Ok(AccountingEntry {
            id: row.get(0)?,
            date: DateTime::parse_from_rfc3339(&date_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            reference: row.get(2)?,
            description: row.get(3)?,
            debit_account: row.get(4)?,
            credit_account: row.get(5)?,
            amount: row.get(6)?,
            entry_type,
            related_id: row.get(8)?,
            related_type: row.get(9)?,
            created_by: row.get(10)?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    fn entry_type_to_string(et: EntryType) -> &'static str {
        et.as_str()
    }
}

impl AccountingEntryRepository for SqliteAccountingEntryRepository {
    fn create(&self, entry: AccountingEntry) -> Result<AccountingEntry, String> {
        let sql = "INSERT INTO accounting_entries (
                      id, date, reference, description, debit_account, credit_account, amount,
                      entry_type, related_id, related_type, created_by, created_at
                  ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        self.pool
            .execute(
                sql,
                &[
                    &entry.id,
                    &entry.date.to_rfc3339(),
                    &entry.reference,
                    &entry.description,
                    &entry.debit_account,
                    &entry.credit_account,
                    &entry.amount.to_string(),
                    &Self::entry_type_to_string(entry.entry_type),
                    &entry.related_id,
                    &entry.related_type,
                    &entry.created_by,
                    &entry.created_at.to_rfc3339(),
                ],
            )
            .map_err(|e| e.to_string())?;

        Ok(entry)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<AccountingEntry>, String> {
        let sql = "SELECT id, date, reference, description, debit_account, credit_account, amount,
                         entry_type, related_id, related_type, created_by, created_at
                  FROM accounting_entries WHERE id = ?";

        self.pool
            .query_row(sql, &[&id], Self::row_to_accounting_entry)
            .map_err(|e| e.to_string())
    }

    fn list(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
        entry_type: Option<EntryType>,
    ) -> Result<Vec<AccountingEntry>, String> {
        let mut sql =
            "SELECT id, date, reference, description, debit_account, credit_account, amount,
                          entry_type, related_id, related_type, created_by, created_at
                    FROM accounting_entries WHERE 1=1"
                .to_string();
        let mut params: Vec<String> = Vec::new();

        if let Some(df) = date_from {
            sql.push_str(" AND date >= ?");
            params.push(df.to_string());
        }

        if let Some(dt) = date_to {
            sql.push_str(" AND date <= ?");
            params.push(dt.to_string());
        }

        if let Some(et) = entry_type {
            sql.push_str(" AND entry_type = ?");
            params.push(Self::entry_type_to_string(et).to_string());
        }

        sql.push_str(" ORDER BY date DESC, reference");

        self.pool
            .query(&sql, &[], Self::row_to_accounting_entry)
            .map_err(|e| e.to_string())
    }

    fn get_by_related(
        &self,
        related_id: &str,
        related_type: &str,
    ) -> Result<Vec<AccountingEntry>, String> {
        let sql = "SELECT id, date, reference, description, debit_account, credit_account, amount,
                          entry_type, related_id, related_type, created_by, created_at
                    FROM accounting_entries 
                    WHERE related_id = ? AND related_type = ?
                    ORDER BY date DESC";

        self.pool
            .query(
                sql,
                &[&related_id, &related_type],
                Self::row_to_accounting_entry,
            )
            .map_err(|e| e.to_string())
    }

    fn get_by_account(&self, account_id: &str) -> Result<Vec<AccountingEntry>, String> {
        let sql = "SELECT id, date, reference, description, debit_account, credit_account, amount,
                          entry_type, related_id, related_type, created_by, created_at
                    FROM accounting_entries 
                    WHERE debit_account = ? OR credit_account = ?
                    ORDER BY date DESC";

        self.pool
            .query(
                sql,
                &[&account_id, &account_id],
                Self::row_to_accounting_entry,
            )
            .map_err(|e| e.to_string())
    }

    fn get_by_date_range(
        &self,
        date_from: &str,
        date_to: &str,
    ) -> Result<Vec<AccountingEntry>, String> {
        let sql = "SELECT id, date, reference, description, debit_account, credit_account, amount,
                          entry_type, related_id, related_type, created_by, created_at
                    FROM accounting_entries 
                    WHERE date >= ? AND date <= ?
                    ORDER BY date DESC, reference";

        self.pool
            .query(sql, &[&date_from, &date_to], Self::row_to_accounting_entry)
            .map_err(|e| e.to_string())
    }

    fn update(&self, entry: AccountingEntry) -> Result<AccountingEntry, String> {
        let sql = "UPDATE accounting_entries 
                  SET date = ?, reference = ?, description = ?, 
                      debit_account = ?, credit_account = ?, amount = ?
                  WHERE id = ?";

        let affected = self
            .pool
            .execute(
                sql,
                &[
                    &entry.date.to_rfc3339(),
                    &entry.reference,
                    &entry.description,
                    &entry.debit_account,
                    &entry.credit_account,
                    &entry.amount.to_string(),
                    &entry.id,
                ],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("AccountingEntry not found: {}", entry.id));
        }

        Ok(entry)
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let sql = "DELETE FROM accounting_entries WHERE id = ?";

        let affected = self.pool.execute(sql, &[&id]).map_err(|e| e.to_string())?;

        Ok(affected > 0)
    }

    fn get_total_debits(&self, date_from: &str, date_to: &str) -> Result<f64, String> {
        let sql = "SELECT COALESCE(SUM(amount), 0) FROM accounting_entries 
                  WHERE date >= ? AND date <= ? AND entry_type != 'reversal'";
        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        let total: f64 = conn
            .query_row(sql, &[date_from, date_to], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        Ok(total)
    }

    fn get_total_credits(&self, date_from: &str, date_to: &str) -> Result<f64, String> {
        let sql = "SELECT COALESCE(SUM(amount), 0) FROM accounting_entries 
                  WHERE date >= ? AND date <= ?";
        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        let total: f64 = conn
            .query_row(sql, &[date_from, date_to], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        Ok(total)
    }

    fn get_next_reference(&self, prefix: &str) -> Result<u32, String> {
        let sql = format!(
            "SELECT COALESCE(MAX(CAST(SUBSTR(reference, {}) AS INTEGER)), 0) + 1 
             FROM accounting_entries 
             WHERE reference LIKE ?",
            prefix.len() + 2
        );
        let pattern = format!("{}%", prefix);
        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        let next: u32 = conn
            .query_row(&sql, [&pattern], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        Ok(next)
    }
}
