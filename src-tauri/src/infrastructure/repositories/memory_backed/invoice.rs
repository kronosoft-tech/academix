//! MemoryBuffer-backed Invoice Repositories
//!
//! Writes buffer to MemoryBuffer, reads check buffer cache first then fallback to SQLite.

use crate::application::ports::invoice::{InvoiceLineRepository, InvoiceRepository};
use crate::domain::entities::invoice::{Invoice, InvoiceLine, InvoicePaymentMethod, InvoiceStatus};
use crate::infrastructure::database;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, CachedEntity, MemoryBuffer};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ── Invoice Repository ──────────────────────────────────────────────

pub struct MemoryBackedInvoiceRepository {
    buffer: Arc<Mutex<MemoryBuffer>>,
    user_id: String,
}

impl MemoryBackedInvoiceRepository {
    pub fn new(buffer: Arc<Mutex<MemoryBuffer>>, user_id: String) -> Self {
        Self { buffer, user_id }
    }

    fn to_cached(invoice: &Invoice) -> CachedEntity {
        CachedEntity {
            id: invoice.id.clone(),
            data: HashMap::from([
                ("id".to_string(), invoice.id.clone()),
                ("series".to_string(), invoice.series.clone()),
                ("number".to_string(), invoice.number.clone()),
                ("client_name".to_string(), invoice.client_name.clone()),
                ("client_ruc".to_string(), invoice.client_ruc.clone()),
                ("client_address".to_string(), invoice.client_address.clone().unwrap_or_default()),
                ("emission_date".to_string(), invoice.emission_date.to_rfc3339()),
                ("due_date".to_string(), invoice.due_date.to_rfc3339()),
                ("subtotal".to_string(), invoice.subtotal.to_string()),
                ("igv".to_string(), invoice.igv.to_string()),
                ("total".to_string(), invoice.total.to_string()),
                ("status".to_string(), invoice.status.as_str().to_string()),
                ("payment_method".to_string(), invoice.payment_method.map(|pm| pm.as_str().to_string()).unwrap_or_default()),
                ("paid_date".to_string(), invoice.paid_date.map(|dt| dt.to_rfc3339()).unwrap_or_default()),
                ("created_at".to_string(), invoice.created_at.to_rfc3339()),
                ("created_by".to_string(), invoice.created_by.clone()),
            ]),
        }
    }

    fn from_cached(cached: &CachedEntity) -> Option<Invoice> {
        Some(Invoice {
            id: cached.data.get("id")?.clone(),
            series: cached.data.get("series")?.clone(),
            number: cached.data.get("number")?.clone(),
            client_name: cached.data.get("client_name")?.clone(),
            client_ruc: cached.data.get("client_ruc")?.clone(),
            client_address: Self::opt_string(cached, "client_address"),
            emission_date: cached.data.get("emission_date")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
            due_date: cached.data.get("due_date")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
            subtotal: cached.data.get("subtotal")?.parse().unwrap_or(0.0),
            igv: cached.data.get("igv")?.parse().unwrap_or(0.0),
            total: cached.data.get("total")?.parse().unwrap_or(0.0),
            status: {
                let s = cached.data.get("status")?;
                InvoiceStatus::from_str(s).unwrap_or(InvoiceStatus::Pending)
            },
            payment_method: cached.data.get("payment_method")
                .and_then(|s| if s.is_empty() { None } else { InvoicePaymentMethod::from_str(s) }),
            paid_date: cached.data.get("paid_date")
                .and_then(|s| if s.is_empty() { None } else { DateTime::parse_from_rfc3339(s).ok() })
                .map(|dt| dt.with_timezone(&Utc)),
            created_at: cached.data.get("created_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
            created_by: cached.data.get("created_by")?.clone(),
        })
    }

    fn opt_string(cached: &CachedEntity, key: &str) -> Option<String> {
        cached.data.get(key).and_then(|v| {
            if v.is_empty() { None } else { Some(v.clone()) }
        })
    }

    fn row_to_invoice(row: &rusqlite::Row<'_>) -> rusqlite::Result<Invoice> {
        let status_str: String = row.get(11)?;
        let payment_method_str: Option<String> = row.get(12)?;
        let emission_str: String = row.get(6)?;
        let due_str: String = row.get(7)?;
        let paid_str: Option<String> = row.get(13)?;
        let created_str: String = row.get(14)?;

        let status = InvoiceStatus::from_str(&status_str).unwrap_or(InvoiceStatus::Pending);
        let payment_method = payment_method_str.and_then(|s| InvoicePaymentMethod::from_str(&s));

        Ok(Invoice {
            id: row.get(0)?,
            series: row.get(1)?,
            number: row.get(2)?,
            client_name: row.get(3)?,
            client_ruc: row.get(4)?,
            client_address: row.get(5)?,
            emission_date: DateTime::parse_from_rfc3339(&emission_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            due_date: DateTime::parse_from_rfc3339(&due_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            subtotal: row.get(8)?,
            igv: row.get(9)?,
            total: row.get(10)?,
            status,
            payment_method,
            paid_date: paid_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            }),
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            created_by: row.get(15)?,
        })
    }
}

impl InvoiceRepository for MemoryBackedInvoiceRepository {
    fn create(&self, invoice: Invoice) -> Result<Invoice, String> {
        let mut buf = self.buffer.lock().map_err(|e| e.to_string())?;
        let data = Self::to_cached(&invoice).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Insert {
            table: "invoices".to_string(),
            data,
        });
        Ok(invoice)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Invoice>, String> {
        let cache_key = format!("invoice:{}", id);
        {
            let buf = self.buffer.lock().map_err(|e| e.to_string())?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(entity) = Self::from_cached(cached) {
                    return Ok(Some(entity));
                }
            }
        }
        // Fallback to SQLite
        let sql = "SELECT id, series, number, client_name, client_ruc, client_address,
                          emission_date, due_date, subtotal, igv, total,
                          status, payment_method, paid_date, created_at, created_by
                   FROM invoices WHERE id = ?";
        let conn = database::open_connection()?;
        match conn.query_row(sql, [id], Self::row_to_invoice) {
            Ok(invoice) => Ok(Some(invoice)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    fn get_by_series_number(&self, series: &str, number: &str) -> Result<Option<Invoice>, String> {
        let cache_key = format!("invoice:{}:{}", series, number);
        {
            let buf = self.buffer.lock().map_err(|e| e.to_string())?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(entity) = Self::from_cached(cached) {
                    return Ok(Some(entity));
                }
            }
        }
        // Fallback to SQLite
        let sql = "SELECT id, series, number, client_name, client_ruc, client_address,
                          emission_date, due_date, subtotal, igv, total,
                          status, payment_method, paid_date, created_at, created_by
                   FROM invoices WHERE series = ? AND number = ?";
        let conn = database::open_connection()?;
        match conn.query_row(sql, rusqlite::params![series, number], Self::row_to_invoice) {
            Ok(invoice) => Ok(Some(invoice)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    fn list(
        &self,
        status: Option<InvoiceStatus>,
        client_ruc: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<Vec<Invoice>, String> {
        // No caching for list queries - go directly to SQLite
        let mut sql = "SELECT id, series, number, client_name, client_ruc, client_address,
                          emission_date, due_date, subtotal, igv, total,
                          status, payment_method, paid_date, created_at, created_by
                    FROM invoices WHERE 1=1"
            .to_string();
        let mut params: Vec<String> = Vec::new();

        if let Some(s) = status {
            sql.push_str(" AND status = ?");
            params.push(s.as_str().to_string());
        }

        if let Some(ruc) = client_ruc {
            sql.push_str(" AND client_ruc = ?");
            params.push(ruc.to_string());
        }

        if let Some(df) = date_from {
            sql.push_str(" AND emission_date >= ?");
            params.push(df.to_string());
        }

        if let Some(dt) = date_to {
            sql.push_str(" AND emission_date <= ?");
            params.push(dt.to_string());
        }

        sql.push_str(" ORDER BY series, CAST(number AS INTEGER) DESC");

        let conn = database::open_connection()?;
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        let rows = stmt
            .query_map(params_refs.as_slice(), Self::row_to_invoice)
            .map_err(|e| e.to_string())?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| e.to_string())
    }

    fn list_by_client(&self, client_ruc: &str) -> Result<Vec<Invoice>, String> {
        // No caching for list queries - go directly to SQLite
        let sql = "SELECT id, series, number, client_name, client_ruc, client_address,
                          emission_date, due_date, subtotal, igv, total,
                          status, payment_method, paid_date, created_at, created_by
                    FROM invoices WHERE client_ruc = ? ORDER BY emission_date DESC";
        let conn = database::open_connection()?;
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![client_ruc], Self::row_to_invoice)
            .map_err(|e| e.to_string())?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| e.to_string())
    }

    fn update(&self, invoice: Invoice) -> Result<Invoice, String> {
        let mut buf = self.buffer.lock().map_err(|e| e.to_string())?;
        let data = Self::to_cached(&invoice).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Update {
            table: "invoices".to_string(),
            id: invoice.id.clone(),
            data,
        });
        Ok(invoice)
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let mut buf = self.buffer.lock().map_err(|e| e.to_string())?;
        buf.buffer_write(&self.user_id, BufferedOperation::Update {
            table: "invoices".to_string(),
            id: id.to_string(),
            data: HashMap::from([
                ("status".to_string(), "cancelled".to_string()),
            ]),
        });
        Ok(true)
    }

    fn get_next_number(&self, series: &str) -> Result<String, String> {
        // No caching for aggregate queries - go directly to SQLite
        let sql = "SELECT COALESCE(MAX(CAST(number AS INTEGER)), 0) + 1 
                  FROM invoices WHERE series = ?";
        let conn = database::open_connection()?;
        let next: String = conn
            .query_row(sql, [series], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        Ok(next)
    }

    fn get_total_pending(&self) -> Result<f64, String> {
        // No caching for aggregate queries - go directly to SQLite
        let sql = "SELECT COALESCE(SUM(total), 0) FROM invoices WHERE status = 'pending'";
        let conn = database::open_connection()?;
        let total: f64 = conn
            .query_row(sql, [], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        Ok(total)
    }

    fn get_total_by_status(&self, status: InvoiceStatus) -> Result<f64, String> {
        // No caching for aggregate queries - go directly to SQLite
        let sql = "SELECT COALESCE(SUM(total), 0) FROM invoices WHERE status = ?";
        let conn = database::open_connection()?;
        let total: f64 = conn
            .query_row(sql, [status.as_str()], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        Ok(total)
    }
}

// ── InvoiceLine Repository ──────────────────────────────────────────

pub struct MemoryBackedInvoiceLineRepository {
    buffer: Arc<Mutex<MemoryBuffer>>,
    user_id: String,
}

impl MemoryBackedInvoiceLineRepository {
    pub fn new(buffer: Arc<Mutex<MemoryBuffer>>, user_id: String) -> Self {
        Self { buffer, user_id }
    }

    fn to_cached(line: &InvoiceLine) -> CachedEntity {
        CachedEntity {
            id: line.id.clone(),
            data: HashMap::from([
                ("id".to_string(), line.id.clone()),
                ("invoice_id".to_string(), line.invoice_id.clone()),
                ("description".to_string(), line.description.clone()),
                ("quantity".to_string(), line.quantity.to_string()),
                ("unit_price".to_string(), line.unit_price.to_string()),
                ("total".to_string(), line.total.to_string()),
                ("created_at".to_string(), line.created_at.to_rfc3339()),
            ]),
        }
    }

    fn from_cached(cached: &CachedEntity) -> Option<InvoiceLine> {
        Some(InvoiceLine {
            id: cached.data.get("id")?.clone(),
            invoice_id: cached.data.get("invoice_id")?.clone(),
            description: cached.data.get("description")?.clone(),
            quantity: cached.data.get("quantity")?.parse().unwrap_or(0.0),
            unit_price: cached.data.get("unit_price")?.parse().unwrap_or(0.0),
            total: cached.data.get("total")?.parse().unwrap_or(0.0),
            created_at: cached.data.get("created_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
        })
    }

    fn row_to_invoice_line(row: &rusqlite::Row<'_>) -> rusqlite::Result<InvoiceLine> {
        let created_str: String = row.get(5)?;

        Ok(InvoiceLine {
            id: row.get(0)?,
            invoice_id: row.get(1)?,
            description: row.get(2)?,
            quantity: row.get(3)?,
            unit_price: row.get(4)?,
            total: row.get(6)?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

impl InvoiceLineRepository for MemoryBackedInvoiceLineRepository {
    fn create(&self, line: InvoiceLine) -> Result<InvoiceLine, String> {
        let mut buf = self.buffer.lock().map_err(|e| e.to_string())?;
        let data = Self::to_cached(&line).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Insert {
            table: "invoice_lines".to_string(),
            data,
        });
        Ok(line)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<InvoiceLine>, String> {
        let cache_key = format!("invoice_line:{}", id);
        {
            let buf = self.buffer.lock().map_err(|e| e.to_string())?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(entity) = Self::from_cached(cached) {
                    return Ok(Some(entity));
                }
            }
        }
        // Fallback to SQLite
        let sql = "SELECT id, invoice_id, description, quantity, unit_price, total, created_at
                  FROM invoice_lines WHERE id = ?";
        let conn = database::open_connection()?;
        match conn.query_row(sql, [id], Self::row_to_invoice_line) {
            Ok(line) => Ok(Some(line)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    fn get_by_invoice(&self, invoice_id: &str) -> Result<Vec<InvoiceLine>, String> {
        // No caching for list queries - go directly to SQLite
        let sql = "SELECT id, invoice_id, description, quantity, unit_price, total, created_at
                  FROM invoice_lines WHERE invoice_id = ? ORDER BY id";
        let conn = database::open_connection()?;
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![invoice_id], Self::row_to_invoice_line)
            .map_err(|e| e.to_string())?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| e.to_string())
    }

    fn update(&self, line: InvoiceLine) -> Result<InvoiceLine, String> {
        let mut buf = self.buffer.lock().map_err(|e| e.to_string())?;
        let data = Self::to_cached(&line).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Update {
            table: "invoice_lines".to_string(),
            id: line.id.clone(),
            data,
        });
        Ok(line)
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let mut buf = self.buffer.lock().map_err(|e| e.to_string())?;
        buf.buffer_write(&self.user_id, BufferedOperation::Delete {
            table: "invoice_lines".to_string(),
            id: id.to_string(),
        });
        Ok(true)
    }

    fn delete_by_invoice(&self, invoice_id: &str) -> Result<bool, String> {
        let mut buf = self.buffer.lock().map_err(|e| e.to_string())?;
        buf.buffer_write(&self.user_id, BufferedOperation::Delete {
            table: "invoice_lines".to_string(),
            id: format!("__by_invoice:{}", invoice_id),
        });
        Ok(true)
    }
}
