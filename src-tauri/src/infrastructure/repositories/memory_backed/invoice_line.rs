//! MemoryBacked Invoice Line Repository
//!
//! Implements InvoiceLineRepository using a MemoryBuffer write-back cache
//! backed by the user's Turso database via ConnectionManager.
//! Phase 5c: Complex repositories.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use async_trait::async_trait;
use crate::application::ports::invoice::InvoiceLineRepository;
use crate::domain::entities::invoice::InvoiceLine;
use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use crate::infrastructure::turso::session::CurrentSession;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct MemoryBackedInvoiceLineRepository {
    connection_manager: Arc<Mutex<ConnectionManager>>,
    memory_buffer: Arc<Mutex<MemoryBuffer>>,
    session: Arc<Mutex<CurrentSession>>,
}

impl MemoryBackedInvoiceLineRepository {
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

    fn line_to_data(line: &InvoiceLine) -> HashMap<String, String> {
        let mut data = HashMap::new();
        data.insert("id".to_string(), line.id.clone());
        data.insert("invoice_id".to_string(), line.invoice_id.clone());
        data.insert("description".to_string(), line.description.clone());
        data.insert("quantity".to_string(), line.quantity.to_string());
        data.insert("unit_price".to_string(), line.unit_price.to_string());
        data.insert("total".to_string(), line.total.to_string());
        data.insert("created_at".to_string(), line.created_at.to_rfc3339());
        data
    }
}

#[async_trait]
impl InvoiceLineRepository for MemoryBackedInvoiceLineRepository {
    async fn create(&self, line: InvoiceLine) -> Result<InvoiceLine, String> {
        let user_id = self.session.lock().await.user_id.clone().ok_or("Not authenticated")?;
        self.memory_buffer.lock().await.buffer_write(
            &user_id,
            BufferedOperation::Insert {
                table: "invoice_lines".to_string(),
                data: Self::line_to_data(&line),
            },
        );
        Ok(line)
    }

    async fn update(&self, line: InvoiceLine) -> Result<InvoiceLine, String> {
        let user_id = self.session.lock().await.user_id.clone().ok_or("Not authenticated")?;
        self.memory_buffer.lock().await.buffer_write(
            &user_id,
            BufferedOperation::Update {
                table: "invoice_lines".to_string(),
                id: line.id.clone(),
                data: Self::line_to_data(&line),
            },
        );
        Ok(line)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<InvoiceLine>, String> {
        let user_id = self.session.lock().await.user_id.clone().ok_or("Not authenticated")?;

        // Check buffer first
        {
            let buf = self.memory_buffer.lock().await;
            if let Some(op) = buf.find_pending_insert(&user_id, "invoice_lines", id) {
                if let BufferedOperation::Insert { data, .. } = op {
                    return Self::line_from_data(data).map(Some);
                }
            }
            if let Some(op) = buf.find_pending_update(&user_id, "invoice_lines", id) {
                if let BufferedOperation::Update { data, .. } = op {
                    return Self::line_from_data(data).map(Some);
                }
            }
        }

        let cm = self.connection_manager.lock().await;
        let conn = cm.get_connection(&user_id).ok_or("No connection".to_string())?;
        let db = conn.db.clone();
        let conn = db.connect().map_err(|e| e.to_string())?;
        let sql = "SELECT id, invoice_id, description, quantity, unit_price, total, created_at FROM invoice_lines WHERE id = ?1";
        let mut rows = conn.query(sql, libsql::params![id]).await.map_err(|e| e.to_string())?;
        match rows.next().await.map_err(|e| e.to_string())? {
            Some(row) => Self::row_to_line(&row).map(Some),
            None => Ok(None),
        }
    }

    async fn get_by_invoice(&self, invoice_id: &str) -> Result<Vec<InvoiceLine>, String> {
        let user_id = self.session.lock().await.user_id.clone().ok_or("Not authenticated")?;

        let cm = self.connection_manager.lock().await;
        let conn = cm.get_connection(&user_id).ok_or("No connection".to_string())?;
        let db = conn.db.clone();
        let conn = db.connect().map_err(|e| e.to_string())?;
        let sql = "SELECT id, invoice_id, description, quantity, unit_price, total, created_at FROM invoice_lines WHERE invoice_id = ?1";
        let mut rows = conn.query(sql, libsql::params![invoice_id]).await.map_err(|e| e.to_string())?;
        let mut results: Vec<InvoiceLine> = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
            results.push(Self::row_to_line(&row)?);
        }

        let buf = self.memory_buffer.lock().await;
        let pending_data: Vec<HashMap<String, String>> = buf
            .scan_pending_inserts(&user_id, "invoice_lines")
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
            if let Ok(line) = Self::line_from_data(&data) {
                results.push(line);
            }
        }

        Ok(results)
    }

    async fn delete(&self, id: &str) -> Result<bool, String> {
        let user_id = self.session.lock().await.user_id.clone().ok_or("Not authenticated")?;
        self.memory_buffer.lock().await.buffer_write(
            &user_id,
            BufferedOperation::Delete {
                table: "invoice_lines".to_string(),
                id: id.to_string(),
            },
        );
        Ok(true)
    }

    async fn delete_by_invoice(&self, invoice_id: &str) -> Result<bool, String> {
        let user_id = self.session.lock().await.user_id.clone().ok_or("Not authenticated")?;
        self.memory_buffer.lock().await.buffer_write(
            &user_id,
            BufferedOperation::Delete {
                table: "invoice_lines".to_string(),
                id: invoice_id.to_string(),
            },
        );
        Ok(true)
    }
}

impl MemoryBackedInvoiceLineRepository {
    fn row_to_line(row: &libsql::Row) -> Result<InvoiceLine, String> {
        let id: String = row.get(0).map_err(|e| e.to_string())?;
        let invoice_id: String = row.get(1).map_err(|e| e.to_string())?;
        let description: String = row.get(2).map_err(|e| e.to_string())?;
        let quantity: f64 = row.get(3).map_err(|e| e.to_string())?;
        let unit_price: f64 = row.get(4).map_err(|e| e.to_string())?;
        let total: f64 = row.get(5).map_err(|e| e.to_string())?;
        let created_at_str: String = row.get(6).map_err(|e| e.to_string())?;
        let created_at = created_at_str.parse::<DateTime<Utc>>()
            .map_err(|e| e.to_string())?;

        Ok(InvoiceLine {
            id, invoice_id, description, quantity, unit_price, total, created_at,
        })
    }

    fn line_from_data(data: &HashMap<String, String>) -> Result<InvoiceLine, String> {
        let id = data.get("id").ok_or_else(|| "missing id".to_string())?.clone();
        let invoice_id = data.get("invoice_id").ok_or_else(|| "missing invoice_id".to_string())?.clone();
        let description = data.get("description").cloned().unwrap_or_default();
        let quantity: f64 = data.get("quantity").ok_or_else(|| "missing quantity".to_string())?.parse::<f64>()
            .map_err(|e| e.to_string())?;
        let unit_price: f64 = data.get("unit_price").ok_or_else(|| "missing unit_price".to_string())?.parse::<f64>()
            .map_err(|e| e.to_string())?;
        let total: f64 = data.get("total").ok_or_else(|| "missing total".to_string())?.parse::<f64>()
            .map_err(|e| e.to_string())?;
        let created_at_str = data.get("created_at").ok_or_else(|| "missing created_at".to_string())?;
        let created_at = created_at_str.parse::<DateTime<Utc>>()
            .map_err(|e| e.to_string())?;

        Ok(InvoiceLine {
            id, invoice_id, description, quantity, unit_price, total, created_at,
        })
    }
}
