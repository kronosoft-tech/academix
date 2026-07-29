//! Accounting SQLite Repository - Simplified income/expense model

use async_trait::async_trait;
use crate::application::ports::accounting::AccountingEntryRepository;
use crate::domain::entities::accounting::{AccountingCategory, AccountingEntry, EntryType};
use crate::infrastructure::local_db;

/// SQLite implementation of AccountingEntryRepository
#[derive(Clone)]
pub struct SqliteAccountingEntryRepository;

impl SqliteAccountingEntryRepository {
    pub fn new() -> Self {
        Self
    }

    fn row_to_accounting_entry(row: &libsql::Row) -> Result<AccountingEntry, String> {
        let entry_type_str: String = row.get(2).map_err(|e| e.to_string())?;
        let category_str: String = row.get(3).map_err(|e| e.to_string())?;

        let entry_type = EntryType::from_str(&entry_type_str).unwrap_or(EntryType::Income);
        let category = AccountingCategory::from_str(&category_str, &entry_type)
            .unwrap_or(AccountingCategory::OtherIncome);

        Ok(AccountingEntry {
            id: row.get(0).map_err(|e| e.to_string())?,
            date: row.get(1).map_err(|e| e.to_string())?,
            entry_type,
            category,
            description: row.get(4).map_err(|e| e.to_string())?,
            amount: row.get(5).map_err(|e| e.to_string())?,
            reference: row.get(6).map_err(|e| e.to_string())?,
            created_at: row.get(7).map_err(|e| e.to_string())?,
        })
    }
}

#[async_trait]
impl AccountingEntryRepository for SqliteAccountingEntryRepository {
    async fn create(&self, entry: AccountingEntry) -> Result<AccountingEntry, String> {
        let sql = "INSERT INTO accounting_entries (
                      id, date, type, category, description, amount, reference, created_at
                  ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        conn.execute(
            sql,
            libsql::params![
                entry.id.clone(),
                entry.date.clone(),
                entry.entry_type.as_str().to_string(),
                entry.category.as_str().to_string(),
                entry.description.clone(),
                entry.amount,
                entry.reference.clone(),
                entry.created_at.clone(),
            ],
        )
        .await
        .map_err(|e| e.to_string())?;

        Ok(entry)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<AccountingEntry>, String> {
        let sql = "SELECT id, date, type, category, description, amount, reference, created_at
                  FROM accounting_entries WHERE id = ?";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let mut rows = conn.query(sql, libsql::params![id.to_owned()]).await.map_err(|e| e.to_string())?;
        match rows.next().await.map_err(|e| e.to_string())? {
            Some(row) => Ok(Some(Self::row_to_accounting_entry(&row)?)),
            None => Ok(None),
        }
    }

    async fn list(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
        entry_type: Option<EntryType>,
    ) -> Result<Vec<AccountingEntry>, String> {
        let mut sql =
            "SELECT id, date, type, category, description, amount, reference, created_at
                    FROM accounting_entries WHERE 1=1"
                .to_string();

        if let Some(df) = date_from {
            sql.push_str(&format!(" AND date >= '{}'", df));
        }
        if let Some(dt) = date_to {
            sql.push_str(&format!(" AND date <= '{}'", dt));
        }
        if let Some(et) = entry_type {
            sql.push_str(&format!(" AND type = '{}'", et.as_str()));
        }
        sql.push_str(" ORDER BY date DESC, created_at DESC");

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let mut rows = conn.query(&sql, ()).await.map_err(|e| e.to_string())?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
            results.push(Self::row_to_accounting_entry(&row)?);
        }
        Ok(results)
    }

    async fn delete(&self, id: &str) -> Result<bool, String> {
        let sql = "DELETE FROM accounting_entries WHERE id = ?";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let affected = conn.execute(sql, libsql::params![id.to_owned()]).await.map_err(|e| e.to_string())?;
        Ok(affected > 0)
    }

    async fn get_total_income(&self, date_from: &str, date_to: &str) -> Result<f64, String> {
        let sql = "SELECT COALESCE(SUM(amount), 0) FROM accounting_entries 
                  WHERE date >= ? AND date <= ? AND type = 'income'";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let mut rows = conn.query(sql, libsql::params![date_from.to_owned(), date_to.to_owned()]).await.map_err(|e| e.to_string())?;
        match rows.next().await.map_err(|e| e.to_string())? {
            Some(row) => {
                let total: f64 = row.get(0).map_err(|e| e.to_string())?;
                Ok(total)
            }
            None => Ok(0.0),
        }
    }

    async fn get_total_expenses(&self, date_from: &str, date_to: &str) -> Result<f64, String> {
        let sql = "SELECT COALESCE(SUM(amount), 0) FROM accounting_entries 
                  WHERE date >= ? AND date <= ? AND type = 'expense'";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let mut rows = conn.query(sql, libsql::params![date_from.to_owned(), date_to.to_owned()]).await.map_err(|e| e.to_string())?;
        match rows.next().await.map_err(|e| e.to_string())? {
            Some(row) => {
                let total: f64 = row.get(0).map_err(|e| e.to_string())?;
                Ok(total)
            }
            None => Ok(0.0),
        }
    }

    async fn get_next_reference(&self, prefix: &str) -> Result<u32, String> {
        let sql = format!(
            "SELECT COALESCE(MAX(CAST(SUBSTR(reference, {}) AS INTEGER)), 0) + 1 
             FROM accounting_entries 
             WHERE reference LIKE ?",
            prefix.len() + 2
        );
        let pattern = format!("{}%", prefix);

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let mut rows = conn.query(&sql, libsql::params![pattern]).await.map_err(|e| e.to_string())?;
        match rows.next().await.map_err(|e| e.to_string())? {
            Some(row) => {
                let next: u32 = row.get(0).map_err(|e| e.to_string())?;
                Ok(next)
            }
            None => Ok(1),
        }
    }
}
