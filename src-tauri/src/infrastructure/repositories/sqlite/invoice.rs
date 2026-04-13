//! Invoice SQLite Repository
//!
//! Implements InvoiceRepository and InvoiceLineRepository using SQLite.

use crate::application::ports::invoice::{InvoiceLineRepository, InvoiceRepository};
use crate::domain::entities::invoice::{Invoice, InvoiceLine, InvoicePaymentMethod, InvoiceStatus};
use crate::infrastructure::database::SqlitePool;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// SQLite implementation of InvoiceRepository
pub struct SqliteInvoiceRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteInvoiceRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    fn row_to_invoice(row: &rusqlite::Row<'_>) -> rusqlite::Result<Invoice> {
        let status_str: String = row.get(11)?;
        let payment_method_str: Option<String> = row.get(12)?;
        let emission_str: String = row.get(13)?;
        let due_str: String = row.get(14)?;
        let paid_str: Option<String> = row.get(15)?;
        let created_str: String = row.get(16)?;

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
            subtotal: row.get(6)?,
            igv: row.get(7)?,
            total: row.get(8)?,
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
            created_by: row.get(9)?,
        })
    }

    fn status_to_string(status: InvoiceStatus) -> &'static str {
        status.as_str()
    }

    fn payment_method_to_string(pm: Option<InvoicePaymentMethod>) -> Option<String> {
        pm.map(|p| p.as_str().to_string())
    }
}

impl InvoiceRepository for SqliteInvoiceRepository {
    fn create(&self, invoice: Invoice) -> Result<Invoice, String> {
        let sql = "INSERT INTO invoices (
                      id, series, number, client_name, client_ruc, client_address,
                      emission_date, due_date, subtotal, igv, total,
                      status, payment_method, paid_date, created_at, created_by
                  ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        let paid_date = invoice.paid_date.map(|dt| dt.to_rfc3339());

        self.pool
            .execute(
                sql,
                &[
                    &invoice.id,
                    &invoice.series,
                    &invoice.number,
                    &invoice.client_name,
                    &invoice.client_ruc,
                    &invoice.client_address,
                    &invoice.emission_date.to_rfc3339(),
                    &invoice.due_date.to_rfc3339(),
                    &invoice.subtotal.to_string(),
                    &invoice.igv.to_string(),
                    &invoice.total.to_string(),
                    &Self::status_to_string(invoice.status),
                    &Self::payment_method_to_string(invoice.payment_method),
                    &paid_date,
                    &invoice.created_at.to_rfc3339(),
                    &invoice.created_by,
                ],
            )
            .map_err(|e| e.to_string())?;

        Ok(invoice)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Invoice>, String> {
        let sql = "SELECT id, series, number, client_name, client_ruc, client_address,
                         emission_date, due_date, subtotal, igv, total,
                         status, payment_method, paid_date, created_at, created_by
                  FROM invoices WHERE id = ?";

        self.pool
            .query_row(sql, &[&id], Self::row_to_invoice)
            .map_err(|e| e.to_string())
    }

    fn get_by_series_number(&self, series: &str, number: &str) -> Result<Option<Invoice>, String> {
        let sql = "SELECT id, series, number, client_name, client_ruc, client_address,
                         emission_date, due_date, subtotal, igv, total,
                         status, payment_method, paid_date, created_at, created_by
                  FROM invoices WHERE series = ? AND number = ?";

        self.pool
            .query_row(sql, &[&series, &number], Self::row_to_invoice)
            .map_err(|e| e.to_string())
    }

    fn list(
        &self,
        status: Option<InvoiceStatus>,
        client_ruc: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<Vec<Invoice>, String> {
        let mut sql = "SELECT id, series, number, client_name, client_ruc, client_address,
                          emission_date, due_date, subtotal, igv, total,
                          status, payment_method, paid_date, created_at, created_by
                    FROM invoices WHERE 1=1"
            .to_string();
        let mut params: Vec<String> = Vec::new();

        if let Some(s) = status {
            sql.push_str(" AND status = ?");
            params.push(Self::status_to_string(s).to_string());
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

        self.pool
            .query(&sql, &[], Self::row_to_invoice)
            .map_err(|e| e.to_string())
    }

    fn list_by_client(&self, client_ruc: &str) -> Result<Vec<Invoice>, String> {
        let sql = "SELECT id, series, number, client_name, client_ruc, client_address,
                          emission_date, due_date, subtotal, igv, total,
                          status, payment_method, paid_date, created_at, created_by
                    FROM invoices WHERE client_ruc = ? ORDER BY emission_date DESC";

        self.pool
            .query(sql, &[&client_ruc], Self::row_to_invoice)
            .map_err(|e| e.to_string())
    }

    fn update(&self, invoice: Invoice) -> Result<Invoice, String> {
        let sql = "UPDATE invoices 
                  SET series = ?, number = ?, client_name = ?, client_ruc = ?, client_address = ?,
                      emission_date = ?, due_date = ?, subtotal = ?, igv = ?, total = ?,
                      status = ?, payment_method = ?, paid_date = ?
                  WHERE id = ?";

        let paid_date = invoice.paid_date.map(|dt| dt.to_rfc3339());

        let affected = self
            .pool
            .execute(
                sql,
                &[
                    &invoice.series,
                    &invoice.number,
                    &invoice.client_name,
                    &invoice.client_ruc,
                    &invoice.client_address,
                    &invoice.emission_date.to_rfc3339(),
                    &invoice.due_date.to_rfc3339(),
                    &invoice.subtotal.to_string(),
                    &invoice.igv.to_string(),
                    &invoice.total.to_string(),
                    &Self::status_to_string(invoice.status),
                    &Self::payment_method_to_string(invoice.payment_method),
                    &paid_date,
                    &invoice.id,
                ],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("Invoice not found: {}", invoice.id));
        }

        Ok(invoice)
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let sql = "UPDATE invoices SET status = 'cancelled' WHERE id = ?";

        let affected = self.pool.execute(sql, &[&id]).map_err(|e| e.to_string())?;

        Ok(affected > 0)
    }

    fn get_next_number(&self, series: &str) -> Result<String, String> {
        let sql = "SELECT COALESCE(MAX(CAST(number AS INTEGER)), 0) + 1 
                  FROM invoices WHERE series = ?";

        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        let next: String = conn
            .query_row(sql, [series], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        Ok(next)
    }

    fn get_total_pending(&self) -> Result<f64, String> {
        let sql = "SELECT COALESCE(SUM(total), 0) FROM invoices WHERE status = 'pending'";

        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        let total: f64 = conn
            .query_row(sql, [], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        Ok(total)
    }

    fn get_total_by_status(&self, status: InvoiceStatus) -> Result<f64, String> {
        let sql = "SELECT COALESCE(SUM(total), 0) FROM invoices WHERE status = ?";

        let conn_ref = self.pool.connection();
        let conn = conn_ref.lock().unwrap();
        let total: f64 = conn
            .query_row(sql, [Self::status_to_string(status)], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        Ok(total)
    }
}

/// SQLite implementation of InvoiceLineRepository
pub struct SqliteInvoiceLineRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteInvoiceLineRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
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

impl InvoiceLineRepository for SqliteInvoiceLineRepository {
    fn create(&self, line: InvoiceLine) -> Result<InvoiceLine, String> {
        let sql = "INSERT INTO invoice_lines (
                      id, invoice_id, description, quantity, unit_price, total, created_at
                  ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)";

        self.pool
            .execute(
                sql,
                &[
                    &line.id,
                    &line.invoice_id,
                    &line.description,
                    &line.quantity.to_string(),
                    &line.unit_price.to_string(),
                    &line.total.to_string(),
                    &line.created_at.to_rfc3339(),
                ],
            )
            .map_err(|e| e.to_string())?;

        Ok(line)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<InvoiceLine>, String> {
        let sql = "SELECT id, invoice_id, description, quantity, unit_price, total, created_at
                  FROM invoice_lines WHERE id = ?";

        self.pool
            .query_row(sql, &[&id], Self::row_to_invoice_line)
            .map_err(|e| e.to_string())
    }

    fn get_by_invoice(&self, invoice_id: &str) -> Result<Vec<InvoiceLine>, String> {
        let sql = "SELECT id, invoice_id, description, quantity, unit_price, total, created_at
                  FROM invoice_lines WHERE invoice_id = ? ORDER BY id";

        self.pool
            .query(sql, &[&invoice_id], Self::row_to_invoice_line)
            .map_err(|e| e.to_string())
    }

    fn update(&self, line: InvoiceLine) -> Result<InvoiceLine, String> {
        let sql = "UPDATE invoice_lines 
                  SET description = ?, quantity = ?, unit_price = ?, total = ?
                  WHERE id = ?";

        let affected = self
            .pool
            .execute(
                sql,
                &[
                    &line.description,
                    &line.quantity.to_string(),
                    &line.unit_price.to_string(),
                    &line.total.to_string(),
                    &line.id,
                ],
            )
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("InvoiceLine not found: {}", line.id));
        }

        Ok(line)
    }

    fn delete(&self, id: &str) -> Result<bool, String> {
        let sql = "DELETE FROM invoice_lines WHERE id = ?";

        let affected = self.pool.execute(sql, &[&id]).map_err(|e| e.to_string())?;

        Ok(affected > 0)
    }

    fn delete_by_invoice(&self, invoice_id: &str) -> Result<bool, String> {
        let sql = "DELETE FROM invoice_lines WHERE invoice_id = ?";

        let affected = self
            .pool
            .execute(sql, &[&invoice_id])
            .map_err(|e| e.to_string())?;

        Ok(affected > 0)
    }
}
