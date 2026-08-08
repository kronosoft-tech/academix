//! MemoryBacked Accounting Entry Repository
//!
//! Implements AccountingEntryRepository using a MemoryBuffer write-back cache
//! backed by the user's Turso database via ConnectionManager.
//! Phase 5c: Complex repositories.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::application::ports::AccountingEntryRepository;
use crate::domain::entities::accounting::{AccountingCategory, AccountingEntry, EntryType};
use crate::domain::errors::DomainError;
use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use crate::infrastructure::turso::session::CurrentSession;
use async_trait::async_trait;

#[derive(Clone)]
pub struct MemoryBackedAccountingEntryRepository {
    connection_manager: Arc<Mutex<ConnectionManager>>,
    memory_buffer: Arc<Mutex<MemoryBuffer>>,
    session: Arc<Mutex<CurrentSession>>,
}

impl MemoryBackedAccountingEntryRepository {
    pub fn new(
        connection_manager: Arc<Mutex<ConnectionManager>>,
        memory_buffer: Arc<Mutex<MemoryBuffer>>,
        session: Arc<Mutex<CurrentSession>>,
    ) -> Self {
        Self {
            connection_manager,
            memory_buffer,
            session,
        }
    }

    fn entry_to_data(entry: &AccountingEntry) -> HashMap<String, String> {
        let mut data = HashMap::new();
        data.insert("id".to_string(), entry.id.clone());
        data.insert("date".to_string(), entry.date.clone());
        data.insert("type".to_string(), entry.entry_type.as_str().to_string());
        data.insert("category".to_string(), entry.category.as_str().to_string());
        data.insert("description".to_string(), entry.description.clone());
        data.insert("amount".to_string(), entry.amount.to_string());
        if let Some(ref r) = entry.reference {
            data.insert("reference".to_string(), r.clone());
        }
        data.insert("created_at".to_string(), entry.created_at.clone());
        data
    }

    fn entry_from_data(data: &HashMap<String, String>) -> Result<AccountingEntry, String> {
        let id = data
            .get("id")
            .ok_or_else(|| "missing id".to_string())?
            .clone();
        let date = data
            .get("date")
            .ok_or_else(|| "missing date".to_string())?
            .clone();
        let entry_type_str = data.get("type").ok_or_else(|| "missing type".to_string())?;
        let entry_type = EntryType::from_str(entry_type_str)
            .ok_or_else(|| format!("invalid entry_type: {}", entry_type_str))?;
        let category_str = data
            .get("category")
            .ok_or_else(|| "missing category".to_string())?;
        let category = AccountingCategory::from_str(category_str, &entry_type)
            .ok_or_else(|| format!("invalid category: {}", category_str))?;
        let description = data.get("description").cloned().unwrap_or_default();
        let amount: f64 = data
            .get("amount")
            .ok_or_else(|| "missing amount".to_string())?
            .parse::<f64>()
            .map_err(|e| e.to_string())?;
        let reference = data.get("reference").cloned();
        let created_at = data.get("created_at").cloned().unwrap_or_default();

        Ok(AccountingEntry {
            id,
            date,
            entry_type,
            category,
            description,
            amount,
            reference,
            created_at,
        })
    }

    /// Convert a libsql::Row into a HashMap<String, String> for cache storage.
    /// Column indices must match the SELECT statement order used in queries:
    /// id(0), date(1), type(2), category(3), description(4), amount(5), reference(6), created_at(7)
    fn row_to_hash_map(row: &libsql::Row) -> Result<HashMap<String, String>, DomainError> {
        let mut map = HashMap::new();
        map.insert(
            "id".to_string(),
            row.get::<String>(0)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "date".to_string(),
            row.get::<String>(1)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "type".to_string(),
            row.get::<String>(2)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "category".to_string(),
            row.get::<String>(3)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "description".to_string(),
            row.get::<String>(4)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "amount".to_string(),
            row.get::<f64>(5)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .to_string(),
        );
        map.insert(
            "reference".to_string(),
            row.get::<Option<String>>(6)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .unwrap_or_default(),
        );
        map.insert(
            "created_at".to_string(),
            row.get::<String>(7)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        Ok(map)
    }
}

#[async_trait]
impl AccountingEntryRepository for MemoryBackedAccountingEntryRepository {
    async fn create(&self, entry: AccountingEntry) -> Result<AccountingEntry, String> {
        let user_id = self
            .session
            .lock()
            .await
            .user_id
            .clone()
            .ok_or("Not authenticated")?;
        self.memory_buffer.lock().await.buffer_write(
            &user_id,
            BufferedOperation::Insert {
                table: "accounting_entries".to_string(),
                data: Self::entry_to_data(&entry),
            },
        );
        Ok(entry)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<AccountingEntry>, String> {
        let user_id = self
            .session
            .lock()
            .await
            .user_id
            .clone()
            .ok_or("Not authenticated")?;

        // Check pending inserts/updates first
        {
            let buf = self.memory_buffer.lock().await;
            if let Some(op) = buf.find_pending_insert(&user_id, "accounting_entries", id) {
                if let BufferedOperation::Insert { data, .. } = op {
                    return Self::entry_from_data(data).map(Some);
                }
            }
            if let Some(op) = buf.find_pending_update(&user_id, "accounting_entries", id) {
                if let BufferedOperation::Update { data, .. } = op {
                    return Self::entry_from_data(data).map(Some);
                }
            }
            if buf.has_pending_delete(&user_id, "accounting_entries", id) {
                return Ok(None);
            }
        }

        // Check entity cache or query Turso
        let row_data: Option<HashMap<String, String>> = {
            let buf = self.memory_buffer.lock().await;
            if let Some(cached) = buf.get_cached_entity(&user_id, "accounting_entries", id) {
                cached.clone()
            } else {
                drop(buf); // Release lock before network call

                let cm = self.connection_manager.lock().await;
                let conn = cm
                    .get_connection(&user_id)
                    .ok_or("No connection".to_string())?;
                let db = conn.db.clone();
                drop(cm);
                let conn = db.connect().map_err(|e| e.to_string())?;
                let sql = "SELECT id, date, type, category, description, amount, reference, created_at FROM accounting_entries WHERE id = ?1";
                let mut rows = conn
                    .query(sql, libsql::params![id])
                    .await
                    .map_err(|e| e.to_string())?;
                let data = match rows.next().await.map_err(|e| e.to_string())? {
                    Some(row) => Some(Self::row_to_hash_map(&row).map_err(|e| e.to_string())?),
                    None => None,
                };

                // Store in cache
                let mut buf = self.memory_buffer.lock().await;
                buf.set_cached_entity(&user_id, "accounting_entries", id, data.clone());
                data
            }
        };

        match row_data {
            Some(data) => Self::entry_from_data(&data).map(Some),
            None => Ok(None),
        }
    }

    async fn list(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
        entry_type: Option<EntryType>,
    ) -> Result<Vec<AccountingEntry>, String> {
        let user_id = self
            .session
            .lock()
            .await
            .user_id
            .clone()
            .ok_or("Not authenticated")?;

        // Step 1: Check cache or query Turso
        let base_rows: Vec<HashMap<String, String>> = {
            let buf = self.memory_buffer.lock().await;
            if let Some(cached) = buf.get_cached_list(&user_id, "accounting_entries") {
                eprintln!(
                    "[ACCOUNTING] Using cached list ({} rows) for user {}",
                    cached.len(),
                    user_id
                );
                cached.clone()
            } else {
                drop(buf); // Release lock before network call

                let cm = self.connection_manager.lock().await;
                let conn = cm
                    .get_connection(&user_id)
                    .ok_or("No connection".to_string())?;
                let db = conn.db.clone();
                drop(cm);
                let conn = db.connect().map_err(|e| e.to_string())?;
                let sql = "SELECT id, date, type, category, description, amount, reference, created_at FROM accounting_entries ORDER BY date";
                eprintln!(
                    "[ACCOUNTING] Querying Turso for accounting_entries (user={})",
                    user_id
                );
                let mut rows = conn.query(sql, libsql::params![]).await.map_err(|e| {
                    eprintln!("[ACCOUNTING] Query FAILED: {}", e);
                    e.to_string()
                })?;

                let mut raw_rows: Vec<HashMap<String, String>> = Vec::new();
                while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
                    raw_rows.push(Self::row_to_hash_map(&row).map_err(|e| e.to_string())?);
                }
                eprintln!("[ACCOUNTING] Got {} rows from Turso", raw_rows.len());

                // Store in cache
                let mut buf = self.memory_buffer.lock().await;
                buf.set_cached_list(&user_id, "accounting_entries", raw_rows.clone());
                raw_rows
            }
        };

        // Step 2: Convert rows to domain entities
        let mut results: Vec<AccountingEntry> = base_rows
            .iter()
            .map(|data| Self::entry_from_data(data))
            .collect::<Result<Vec<_>, _>>()?;

        // Step 3: Filter in-memory
        if let Some(et) = entry_type {
            let et_str = et.as_str().to_string();
            results.retain(|e| e.entry_type.as_str() == et_str);
        }
        if let Some(from) = date_from {
            let f = if from.len() >= 10 { &from[..10] } else { from };
            results.retain(|e| {
                let d = if e.date.len() >= 10 {
                    &e.date[..10]
                } else {
                    &e.date
                };
                d >= f
            });
        }
        if let Some(to) = date_to {
            let t = if to.len() >= 10 { &to[..10] } else { to };
            results.retain(|e| {
                let d = if e.date.len() >= 10 {
                    &e.date[..10]
                } else {
                    &e.date
                };
                d <= t
            });
        }

        // Step 4: Merge pending inserts
        let buf = self.memory_buffer.lock().await;
        let pending_data: Vec<HashMap<String, String>> = buf
            .scan_pending_inserts(&user_id, "accounting_entries")
            .into_iter()
            .filter_map(|op| {
                if let BufferedOperation::Insert { data, .. } = op {
                    Some(data.clone())
                } else {
                    None
                }
            })
            .collect();
        drop(buf);
        for data in pending_data {
            if let Ok(entry) = Self::entry_from_data(&data) {
                results.push(entry);
            }
        }

        Ok(results)
    }

    async fn delete(&self, id: &str) -> Result<bool, String> {
        let user_id = self
            .session
            .lock()
            .await
            .user_id
            .clone()
            .ok_or("Not authenticated")?;
        self.memory_buffer.lock().await.buffer_write(
            &user_id,
            BufferedOperation::Delete {
                table: "accounting_entries".to_string(),
                id: id.to_string(),
            },
        );
        Ok(true)
    }

    async fn get_total_income(&self, date_from: &str, date_to: &str) -> Result<f64, String> {
        let entries = self
            .list(Some(date_from), Some(date_to), Some(EntryType::Income))
            .await?;
        Ok(entries.iter().map(|e| e.amount).sum())
    }

    async fn get_total_expenses(&self, date_from: &str, date_to: &str) -> Result<f64, String> {
        let entries = self
            .list(Some(date_from), Some(date_to), Some(EntryType::Expense))
            .await?;
        Ok(entries.iter().map(|e| e.amount).sum())
    }

    async fn get_next_reference(&self, _prefix: &str) -> Result<u32, String> {
        Ok(1)
    }
}
