//! MemoryBuffer-backed Accounting Repository
//!
//! Writes buffer to MemoryBuffer, reads check buffer cache first then fallback to SQLite.

use crate::application::ports::accounting::AccountingEntryRepository;
use crate::domain::entities::accounting::{AccountingCategory, AccountingEntry, EntryType};
use crate::infrastructure::database;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, CachedEntity, MemoryBuffer};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MemoryBackedAccountingEntryRepository {
    buffer: Arc<Mutex<MemoryBuffer>>,
    user_id: String,
}

impl MemoryBackedAccountingEntryRepository {
    pub fn new(buffer: Arc<Mutex<MemoryBuffer>>, user_id: String) -> Self {
        Self { buffer, user_id }
    }

    fn to_cached(entry: &AccountingEntry) -> CachedEntity {
        CachedEntity {
            id: entry.id.clone(),
            data: HashMap::from([
                ("id".to_string(), entry.id.clone()),
                ("date".to_string(), entry.date.clone()),
                ("entry_type".to_string(), entry.entry_type.as_str().to_string()),
                ("category".to_string(), entry.category.as_str().to_string()),
                ("description".to_string(), entry.description.clone()),
                ("amount".to_string(), entry.amount.to_string()),
                ("reference".to_string(), entry.reference.clone().unwrap_or_default()),
                ("created_at".to_string(), entry.created_at.clone()),
            ]),
        }
    }

    fn from_cached(cached: &CachedEntity) -> Option<AccountingEntry> {
        let entry_type_str = cached.data.get("entry_type")?;
        let entry_type = EntryType::from_str(entry_type_str).unwrap_or(EntryType::Income);
        let category_str = cached.data.get("category")?;
        let category = AccountingCategory::from_str(category_str, &entry_type)
            .unwrap_or(AccountingCategory::OtherIncome);

        Some(AccountingEntry {
            id: cached.data.get("id")?.clone(),
            date: cached.data.get("date")?.clone(),
            entry_type,
            category,
            description: cached.data.get("description")?.clone(),
            amount: cached.data.get("amount")?.parse().unwrap_or(0.0),
            reference: {
                let v = cached.data.get("reference")?;
                if v.is_empty() { None } else { Some(v.clone()) }
            },
            created_at: cached.data.get("created_at")?.clone(),
        })
    }

    fn row_to_accounting_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<AccountingEntry> {
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

impl AccountingEntryRepository for MemoryBackedAccountingEntryRepository {
    fn create(&self, entry: AccountingEntry) -> Result<AccountingEntry, String> {
        let mut buf = self.buffer.lock().map_err(|e| e.to_string())?;
        let data = Self::to_cached(&entry).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Insert {
            table: "accounting_entries".to_string(),
            data,
        });
        Ok(entry)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<AccountingEntry>, String> {
        let cache_key = format!("accounting_entry:{}", id);
        {
            let buf = self.buffer.lock().map_err(|e| e.to_string())?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(entity) = Self::from_cached(cached) {
                    return Ok(Some(entity));
                }
            }
        }
        // Fallback to SQLite
        let sql = "SELECT id, date, type, category, description, amount, reference, created_at
                  FROM accounting_entries WHERE id = ?";
        let conn = database::open_connection()?;
        match conn.query_row(sql, [id], Self::row_to_accounting_entry) {
            Ok(entry) => Ok(Some(entry)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    fn list(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
        entry_type: Option<EntryType>,
    ) -> Result<Vec<AccountingEntry>, String> {
        // No caching for list queries - go directly to SQLite
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

        let conn = database::open_connection()?;
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        let rows = stmt
            .query_map(params_refs.as_slice(), Self::row_to_accounting_entry)
            .map_err(|e| e.to_string())?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| e.to_string())
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let mut buf = self.buffer.lock().map_err(|e| e.to_string())?;
        buf.buffer_write(&self.user_id, BufferedOperation::Delete {
            table: "accounting_entries".to_string(),
            id: id.to_string(),
        });
        Ok(true)
    }

    fn get_total_income(&self, date_from: &str, date_to: &str) -> Result<f64, String> {
        // No caching for aggregate queries - go directly to SQLite
        let sql = "SELECT COALESCE(SUM(amount), 0) FROM accounting_entries 
                  WHERE date >= ? AND date <= ? AND type = 'income'";
        let conn = database::open_connection()?;
        let total: f64 = conn
            .query_row(sql, rusqlite::params![date_from, date_to], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        Ok(total)
    }

    fn get_total_expenses(&self, date_from: &str, date_to: &str) -> Result<f64, String> {
        // No caching for aggregate queries - go directly to SQLite
        let sql = "SELECT COALESCE(SUM(amount), 0) FROM accounting_entries 
                  WHERE date >= ? AND date <= ? AND type = 'expense'";
        let conn = database::open_connection()?;
        let total: f64 = conn
            .query_row(sql, rusqlite::params![date_from, date_to], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        Ok(total)
    }

    fn get_next_reference(&self, prefix: &str) -> Result<u32, String> {
        // No caching for aggregate queries - go directly to SQLite
        let sql = format!(
            "SELECT COALESCE(MAX(CAST(SUBSTR(reference, {}) AS INTEGER)), 0) + 1 
             FROM accounting_entries 
             WHERE reference LIKE ?",
            prefix.len() + 2
        );
        let pattern = format!("{}%", prefix);

        let conn = database::open_connection()?;
        let next: u32 = conn
            .query_row(&sql, [&pattern], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        Ok(next)
    }
}
