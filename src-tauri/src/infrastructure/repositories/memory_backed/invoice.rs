//! MemoryBacked Invoice Repository
//!
//! Implements InvoiceRepository using a MemoryBuffer write-back cache
//! backed by the user's Turso database via ConnectionManager.
//! Phase 5c: Complex repositories.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use async_trait::async_trait;
use crate::application::ports::invoice::InvoiceRepository;
use crate::domain::entities::invoice::{Invoice, InvoicePaymentMethod, InvoiceStatus};
use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use crate::infrastructure::turso::session::CurrentSession;
use chrono::{DateTime, Utc};

pub struct MemoryBackedInvoiceRepository {
    connection_manager: Arc<Mutex<ConnectionManager>>,
    memory_buffer: Arc<Mutex<MemoryBuffer>>,
    session: Arc<Mutex<CurrentSession>>,
}

impl MemoryBackedInvoiceRepository {
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

    fn invoice_to_data(invoice: &Invoice) -> HashMap<String, String> {
        let mut data = HashMap::new();
        data.insert("id".to_string(), invoice.id.clone());
        data.insert("series".to_string(), invoice.series.clone());
        data.insert("number".to_string(), invoice.number.clone());
        data.insert("client_name".to_string(), invoice.client_name.clone());
        data.insert("client_ruc".to_string(), invoice.client_ruc.clone());
        if let Some(ref addr) = invoice.client_address {
            data.insert("client_address".to_string(), addr.clone());
        }
        data.insert("emission_date".to_string(), invoice.emission_date.to_rfc3339());
        data.insert("due_date".to_string(), invoice.due_date.to_rfc3339());
        data.insert("subtotal".to_string(), invoice.subtotal.to_string());
        data.insert("igv".to_string(), invoice.igv.to_string());
        data.insert("total".to_string(), invoice.total.to_string());
        data.insert("status".to_string(), invoice.status.as_str().to_string());
        if let Some(ref pm) = invoice.payment_method {
            data.insert("payment_method".to_string(), pm.as_str().to_string());
        }
        if let Some(pd) = invoice.paid_date {
            data.insert("paid_date".to_string(), pd.to_rfc3339());
        }
        data.insert("created_at".to_string(), invoice.created_at.to_rfc3339());
        data.insert("created_by".to_string(), invoice.created_by.clone());
        data
    }

    fn invoice_from_data(data: &HashMap<String, String>) -> Result<Invoice, String> {
        let id = data.get("id").ok_or_else(|| "missing id".to_string())?.clone();
        let series = data.get("series").ok_or_else(|| "missing series".to_string())?.clone();
        let number = data.get("number").ok_or_else(|| "missing number".to_string())?.clone();
        let client_name = data.get("client_name").ok_or_else(|| "missing client_name".to_string())?.clone();
        let client_ruc = data.get("client_ruc").ok_or_else(|| "missing client_ruc".to_string())?.clone();
        let client_address = data.get("client_address").cloned();

        let emission_date = data.get("emission_date")
            .ok_or_else(|| "missing emission_date".to_string())?
            .parse::<DateTime<Utc>>()
            .map_err(|e| e.to_string())?;

        let due_date = data.get("due_date")
            .ok_or_else(|| "missing due_date".to_string())?
            .parse::<DateTime<Utc>>()
            .map_err(|e| e.to_string())?;

        let subtotal: f64 = data.get("subtotal").ok_or_else(|| "missing subtotal".to_string())?.parse::<f64>()
            .map_err(|e| e.to_string())?;
        let igv: f64 = data.get("igv").ok_or_else(|| "missing igv".to_string())?.parse::<f64>()
            .map_err(|e| e.to_string())?;
        let total: f64 = data.get("total").ok_or_else(|| "missing total".to_string())?.parse::<f64>()
            .map_err(|e| e.to_string())?;

        let status = InvoiceStatus::from_str(
            data.get("status").ok_or_else(|| "missing status".to_string())?
        ).ok_or_else(|| "invalid status".to_string())?;

        let payment_method_str = data.get("payment_method").cloned();
        let payment_method = payment_method_str.as_deref().and_then(InvoicePaymentMethod::from_str);

        let paid_date = data.get("paid_date").as_ref().and_then(|s| s.parse::<DateTime<Utc>>().ok());

        let created_at = data.get("created_at")
            .ok_or_else(|| "missing created_at".to_string())?
            .parse::<DateTime<Utc>>()
            .map_err(|e| e.to_string())?;

        let created_by = data.get("created_by").cloned().unwrap_or_default();

        Ok(Invoice {
            id, series, number, client_name, client_ruc, client_address,
            emission_date, due_date, subtotal, igv, total, status,
            payment_method, paid_date, created_at, created_by,
        })
    }
}

#[async_trait]
impl InvoiceRepository for MemoryBackedInvoiceRepository {
    async fn create(&self, invoice: Invoice) -> Result<Invoice, String> {
        let user_id = self.session.lock().await.user_id.clone().ok_or("Not authenticated")?;
        self.memory_buffer.lock().await.buffer_write(
            &user_id,
            BufferedOperation::Insert {
                table: "invoices".to_string(),
                data: Self::invoice_to_data(&invoice),
            },
        );
        Ok(invoice)
    }

    async fn update(&self, invoice: Invoice) -> Result<Invoice, String> {
        let user_id = self.session.lock().await.user_id.clone().ok_or("Not authenticated")?;
        self.memory_buffer.lock().await.buffer_write(
            &user_id,
            BufferedOperation::Update {
                table: "invoices".to_string(),
                id: invoice.id.clone(),
                data: Self::invoice_to_data(&invoice),
            },
        );
        Ok(invoice)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Invoice>, String> {
        let user_id = self.session.lock().await.user_id.clone().ok_or("Not authenticated")?;

        // Check buffer first
        {
            let buf = self.memory_buffer.lock().await;
            if let Some(op) = buf.find_pending_insert(&user_id, "invoices", id) {
                if let BufferedOperation::Insert { data, .. } = op {
                    return Self::invoice_from_data(data).map(Some);
                }
            }
            if let Some(op) = buf.find_pending_update(&user_id, "invoices", id) {
                if let BufferedOperation::Update { data, .. } = op {
                    return Self::invoice_from_data(data).map(Some);
                }
            }
        }

        let cm = self.connection_manager.lock().await;
        let conn = cm.get_connection(&user_id).ok_or("No connection".to_string())?;
        let db = conn.db.clone();
        let conn = db.connect().map_err(|e| e.to_string())?;
        let sql = "SELECT id, series, number, client_name, client_ruc, client_address, emission_date, due_date, subtotal, igv, total, status, payment_method, paid_date, created_at, created_by FROM invoices WHERE id = ?1";
        let mut rows = conn.query(sql, libsql::params![id]).await.map_err(|e| e.to_string())?;
        match rows.next().await.map_err(|e| e.to_string())? {
            Some(row) => Self::row_to_invoice(&row).map(Some),
            None => Ok(None),
        }
    }

    async fn get_by_series_number(&self, series: &str, number: &str) -> Result<Option<Invoice>, String> {
        let user_id = self.session.lock().await.user_id.clone().ok_or("Not authenticated")?;

        let cm = self.connection_manager.lock().await;
        let conn = cm.get_connection(&user_id).ok_or("No connection".to_string())?;
        let db = conn.db.clone();
        let conn = db.connect().map_err(|e| e.to_string())?;
        let sql = "SELECT id, series, number, client_name, client_ruc, client_address, emission_date, due_date, subtotal, igv, total, status, payment_method, paid_date, created_at, created_by FROM invoices WHERE series = ?1 AND number = ?2";
        let mut rows = conn.query(sql, libsql::params![series, number]).await.map_err(|e| e.to_string())?;
        match rows.next().await.map_err(|e| e.to_string())? {
            Some(row) => Self::row_to_invoice(&row).map(Some),
            None => Ok(None),
        }
    }

    async fn list(&self, status: Option<InvoiceStatus>, client_ruc: Option<&str>, date_from: Option<&str>, date_to: Option<&str>) -> Result<Vec<Invoice>, String> {
        let user_id = self.session.lock().await.user_id.clone().ok_or("Not authenticated")?;

        let cm = self.connection_manager.lock().await;
        let conn = cm.get_connection(&user_id).ok_or("No connection".to_string())?;
        let db = conn.db.clone();
        let conn = db.connect().map_err(|e| e.to_string())?;
        let sql = "SELECT id, series, number, client_name, client_ruc, client_address, emission_date, due_date, subtotal, igv, total, status, payment_method, paid_date, created_at, created_by FROM invoices ORDER BY created_at DESC";
        let mut rows = conn.query(sql, libsql::params![]).await.map_err(|e| e.to_string())?;
        let mut results: Vec<Invoice> = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
            results.push(Self::row_to_invoice(&row)?);
        }

        if let Some(s) = status {
            results.retain(|inv| inv.status == s);
        }
        if let Some(ruc) = client_ruc {
            results.retain(|inv| inv.client_ruc == ruc);
        }
        if let Some(from) = date_from {
            results.retain(|inv| inv.emission_date.to_rfc3339().as_str() >= from);
        }
        if let Some(to) = date_to {
            results.retain(|inv| inv.emission_date.to_rfc3339().as_str() <= to);
        }

        // Merge pending inserts
        let buf = self.memory_buffer.lock().await;
        let pending_data: Vec<HashMap<String, String>> = buf
            .scan_pending_inserts(&user_id, "invoices")
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
            if let Ok(invoice) = Self::invoice_from_data(&data) {
                results.push(invoice);
            }
        }

        Ok(results)
    }

    async fn list_by_client(&self, client_ruc: &str) -> Result<Vec<Invoice>, String> {
        self.list(None, Some(client_ruc), None, None).await
    }

    async fn delete(&self, id: &str) -> Result<bool, String> {
        let user_id = self.session.lock().await.user_id.clone().ok_or("Not authenticated")?;
        self.memory_buffer.lock().await.buffer_write(
            &user_id,
            BufferedOperation::Delete {
                table: "invoices".to_string(),
                id: id.to_string(),
            },
        );
        Ok(true)
    }

    async fn get_next_number(&self, _series: &str) -> Result<String, String> {
        Ok("00001".to_string())
    }

    async fn get_total_pending(&self) -> Result<f64, String> {
        let invoices = self.list(Some(InvoiceStatus::Pending), None, None, None).await?;
        Ok(invoices.iter().map(|i| i.total).sum())
    }

    async fn get_total_by_status(&self, status: InvoiceStatus) -> Result<f64, String> {
        let invoices = self.list(Some(status), None, None, None).await?;
        Ok(invoices.iter().map(|i| i.total).sum())
    }
}

impl MemoryBackedInvoiceRepository {
    fn row_to_invoice(row: &libsql::Row) -> Result<Invoice, String> {
        let id: String = row.get(0).map_err(|e| e.to_string())?;
        let series: String = row.get(1).map_err(|e| e.to_string())?;
        let number: String = row.get(2).map_err(|e| e.to_string())?;
        let client_name: String = row.get(3).map_err(|e| e.to_string())?;
        let client_ruc: String = row.get(4).map_err(|e| e.to_string())?;
        let client_address: Option<String> = row.get(5).map_err(|e| e.to_string())?;

        let emission_date_str: String = row.get(6).map_err(|e| e.to_string())?;
        let emission_date = emission_date_str.parse::<DateTime<Utc>>()
            .map_err(|e| e.to_string())?;

        let due_date_str: String = row.get(7).map_err(|e| e.to_string())?;
        let due_date = due_date_str.parse::<DateTime<Utc>>()
            .map_err(|e| e.to_string())?;

        let subtotal: f64 = row.get(8).map_err(|e| e.to_string())?;
        let igv: f64 = row.get(9).map_err(|e| e.to_string())?;
        let total: f64 = row.get(10).map_err(|e| e.to_string())?;

        let status_str: String = row.get(11).map_err(|e| e.to_string())?;
        let status = InvoiceStatus::from_str(&status_str)
            .ok_or_else(|| format!("invalid status: {}", status_str))?;

        let payment_method_str: Option<String> = row.get(12).map_err(|e| e.to_string())?;
        let payment_method = payment_method_str.as_deref().and_then(InvoicePaymentMethod::from_str);

        let paid_date_str: Option<String> = row.get(13).map_err(|e| e.to_string())?;
        let paid_date = paid_date_str.as_deref().and_then(|s| s.parse::<DateTime<Utc>>().ok());

        let created_at_str: String = row.get(14).map_err(|e| e.to_string())?;
        let created_at = created_at_str.parse::<DateTime<Utc>>()
            .map_err(|e| e.to_string())?;

        let created_by: String = row.get(15).map_err(|e| e.to_string())?;

        Ok(Invoice {
            id, series, number, client_name, client_ruc, client_address,
            emission_date, due_date, subtotal, igv, total, status,
            payment_method, paid_date, created_at, created_by,
        })
    }
}
