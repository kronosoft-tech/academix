//! Accounting SQLite Repository - Simplified income/expense model

use crate::application::ports::accounting::AccountingEntryRepository;
use crate::domain::entities::accounting::{AccountingCategory, AccountingEntry, EntryType};
use crate::infrastructure::database::SqlitePool;
use std::sync::Arc;

/// SQLite implementation of AccountingEntryRepository
#[derive(Clone)]
pub struct SqliteAccountingEntryRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteAccountingEntryRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> Arc<SqlitePool> {
        Arc::clone(&self.pool)
    }

    fn row_to_accounting_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<AccountingEntry> {
        // SQL: id, date, type, category, description, amount, reference, created_at
        let entry_type_str: String = row.get(2)?;
        let category_str: String = row.get(3)?;

        let entry_type = EntryType::from_str(&entry_type_str).unwrap_or(EntryType::Income);
        let category = AccountingCategory::from_str(&category_str, &entry_type)
            .unwrap_or(AccountingCategory::OtherIncome);

        Ok(AccountingEntry {
            id: row.get(0)?,
            date: row.get(1)?,
            entry_type,
            category,
            description: row.get(4)?,
            amount: row.get(5)?,
            reference: row.get(6)?,
            created_at: row.get(7)?,
        })
    }
}

impl AccountingEntryRepository for SqliteAccountingEntryRepository {
    fn create(&self, entry: AccountingEntry) -> Result<AccountingEntry, String> {
        let sql = "INSERT INTO accounting_entries (
                      id, date, type, category, description, amount, reference, created_at
                  ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)";

        self.pool
            .execute(
                sql,
                &[
                    &entry.id,
                    &entry.date,
                    &entry.entry_type.as_str(),
                    &entry.category.as_str(),
                    &entry.description,
                    &entry.amount.to_string(),
                    &entry.reference,
                    &entry.created_at,
                ],
            )
            .map_err(|e| e.to_string())?;

        Ok(entry)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<AccountingEntry>, String> {
        let sql = "SELECT id, date, type, category, description, amount, reference, created_at
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
            "SELECT id, date, type, category, description, amount, reference, created_at
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
            sql.push_str(" AND type = ?");
            params.push(et.as_str().to_string());
        }

        sql.push_str(" ORDER BY date DESC, created_at DESC");

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

        self.pool
            .query(&sql, &params_refs, Self::row_to_accounting_entry)
            .map_err(|e| e.to_string())
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let sql = "DELETE FROM accounting_entries WHERE id = ?";

        let affected = self.pool.execute(sql, &[&id]).map_err(|e| e.to_string())?;

        Ok(affected > 0)
    }

    fn get_total_income(&self, date_from: &str, date_to: &str) -> Result<f64, String> {
        let sql = "SELECT COALESCE(SUM(amount), 0) FROM accounting_entries 
                  WHERE date >= ? AND date <= ? AND type = 'income'";
        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        let total: f64 = conn
            .query_row(sql, &[date_from, date_to], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        Ok(total)
    }

    fn get_total_expenses(&self, date_from: &str, date_to: &str) -> Result<f64, String> {
        let sql = "SELECT COALESCE(SUM(amount), 0) FROM accounting_entries 
                  WHERE date >= ? AND date <= ? AND type = 'expense'";
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
