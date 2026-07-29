//! Invoice SQLite Repository
//!
//! Implements InvoiceRepository and InvoiceLineRepository using SQLite.

use async_trait::async_trait;
use crate::application::ports::invoice::{InvoiceLineRepository, InvoiceRepository};
use crate::domain::entities::invoice::{Invoice, InvoiceLine, InvoicePaymentMethod, InvoiceStatus};
use crate::infrastructure::local_db;
use chrono::{DateTime, Utc};

/// SQLite implementation of InvoiceRepository
#[derive(Clone)]
pub struct SqliteInvoiceRepository;

impl SqliteInvoiceRepository {
    pub fn new() -> Self {
        Self
    }

    fn row_to_invoice(row: &libsql::Row) -> Result<Invoice, String> {
        let status_str: String = row.get(11).map_err(|e| e.to_string())?;
        let payment_method_str: Option<String> = row.get(12).map_err(|e| e.to_string())?;
        let emission_str: String = row.get(6).map_err(|e| e.to_string())?;
        let due_str: String = row.get(7).map_err(|e| e.to_string())?;
        let paid_str: Option<String> = row.get(13).map_err(|e| e.to_string())?;
        let created_str: String = row.get(14).map_err(|e| e.to_string())?;

        let status = InvoiceStatus::from_str(&status_str).unwrap_or(InvoiceStatus::Pending);
        let payment_method = payment_method_str.and_then(|s| InvoicePaymentMethod::from_str(&s));

        Ok(Invoice {
            id: row.get(0).map_err(|e| e.to_string())?,
            series: row.get(1).map_err(|e| e.to_string())?,
            number: row.get(2).map_err(|e| e.to_string())?,
            client_name: row.get(3).map_err(|e| e.to_string())?,
            client_ruc: row.get(4).map_err(|e| e.to_string())?,
            client_address: row.get(5).map_err(|e| e.to_string())?,
            emission_date: DateTime::parse_from_rfc3339(&emission_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            due_date: DateTime::parse_from_rfc3339(&due_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            subtotal: row.get(8).map_err(|e| e.to_string())?,
            igv: row.get(9).map_err(|e| e.to_string())?,
            total: row.get(10).map_err(|e| e.to_string())?,
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
            created_by: row.get(15).map_err(|e| e.to_string())?,
        })
    }
}

impl Default for SqliteInvoiceRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InvoiceRepository for SqliteInvoiceRepository {
    async fn create(&self, invoice: Invoice) -> Result<Invoice, String> {
        let sql = "INSERT INTO invoices (
                      id, series, number, client_name, client_ruc, client_address,
                      emission_date, due_date, subtotal, igv, total,
                      status, payment_method, paid_date, created_at, created_by
                  ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        let paid_date = invoice.paid_date.map(|dt| dt.to_rfc3339());
        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        conn.execute(
            sql,
            libsql::params![
                invoice.id.clone(),
                invoice.series.clone(),
                invoice.number.clone(),
                invoice.client_name.clone(),
                invoice.client_ruc.clone(),
                invoice.client_address.clone(),
                invoice.emission_date.to_rfc3339(),
                invoice.due_date.to_rfc3339(),
                invoice.subtotal,
                invoice.igv,
                invoice.total,
                invoice.status.as_str().to_string(),
                invoice.payment_method.map(|pm| pm.as_str().to_string()),
                paid_date,
                invoice.created_at.to_rfc3339(),
                invoice.created_by.clone(),
            ],
        )
        .await
        .map_err(|e| e.to_string())?;

        Ok(invoice)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Invoice>, String> {
        let sql = "SELECT id, series, number, client_name, client_ruc, client_address,
                          emission_date, due_date, subtotal, igv, total,
                          status, payment_method, paid_date, created_at, created_by
                   FROM invoices WHERE id = ?";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let mut rows = conn.query(sql, libsql::params![id.to_owned()]).await.map_err(|e| e.to_string())?;
        match rows.next().await.map_err(|e| e.to_string())? {
            Some(row) => Ok(Some(Self::row_to_invoice(&row)?)),
            None => Ok(None),
        }
    }

    async fn get_by_series_number(&self, series: &str, number: &str) -> Result<Option<Invoice>, String> {
        let sql = "SELECT id, series, number, client_name, client_ruc, client_address,
                          emission_date, due_date, subtotal, igv, total,
                          status, payment_method, paid_date, created_at, created_by
                   FROM invoices WHERE series = ? AND number = ?";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let mut rows = conn.query(sql, libsql::params![series.to_owned(), number.to_owned()]).await.map_err(|e| e.to_string())?;
        match rows.next().await.map_err(|e| e.to_string())? {
            Some(row) => Ok(Some(Self::row_to_invoice(&row)?)),
            None => Ok(None),
        }
    }

    async fn list(
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

        if let Some(s) = status {
            sql.push_str(&format!(" AND status = '{}'", s.as_str()));
        }

        if let Some(ruc) = client_ruc {
            sql.push_str(&format!(" AND client_ruc = '{}'", ruc));
        }

        if let Some(df) = date_from {
            sql.push_str(&format!(" AND emission_date >= '{}'", df));
        }

        if let Some(dt) = date_to {
            sql.push_str(&format!(" AND emission_date <= '{}'", dt));
        }

        sql.push_str(" ORDER BY series, CAST(number AS INTEGER) DESC");

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let mut rows = conn.query(&sql, ()).await.map_err(|e| e.to_string())?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
            results.push(Self::row_to_invoice(&row)?);
        }
        Ok(results)
    }

    async fn list_by_client(&self, client_ruc: &str) -> Result<Vec<Invoice>, String> {
        let sql = "SELECT id, series, number, client_name, client_ruc, client_address,
                          emission_date, due_date, subtotal, igv, total,
                          status, payment_method, paid_date, created_at, created_by
                    FROM invoices WHERE client_ruc = ? ORDER BY emission_date DESC";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let mut rows = conn.query(sql, libsql::params![client_ruc.to_owned()]).await.map_err(|e| e.to_string())?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
            results.push(Self::row_to_invoice(&row)?);
        }
        Ok(results)
    }

    async fn update(&self, invoice: Invoice) -> Result<Invoice, String> {
        let sql = "UPDATE invoices 
                  SET series = ?, number = ?, client_name = ?, client_ruc = ?, client_address = ?,
                      emission_date = ?, due_date = ?, subtotal = ?, igv = ?, total = ?,
                      status = ?, payment_method = ?, paid_date = ?
                  WHERE id = ?";

        let paid_date = invoice.paid_date.map(|dt| dt.to_rfc3339());
        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let affected = conn
            .execute(
                sql,
                libsql::params![
                    invoice.series.clone(),
                    invoice.number.clone(),
                    invoice.client_name.clone(),
                    invoice.client_ruc.clone(),
                    invoice.client_address.clone(),
                    invoice.emission_date.to_rfc3339(),
                    invoice.due_date.to_rfc3339(),
                    invoice.subtotal,
                    invoice.igv,
                    invoice.total,
                    invoice.status.as_str().to_string(),
                    invoice.payment_method.map(|pm| pm.as_str().to_string()),
                    paid_date,
                    invoice.id.clone(),
                ],
            )
            .await
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("Invoice not found: {}", invoice.id));
        }

        Ok(invoice)
    }

    async fn delete(&self, id: &str) -> Result<bool, String> {
        let sql = "UPDATE invoices SET status = 'cancelled' WHERE id = ?";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let affected = conn.execute(sql, libsql::params![id.to_owned()]).await.map_err(|e| e.to_string())?;
        Ok(affected > 0)
    }

    async fn get_next_number(&self, series: &str) -> Result<String, String> {
        let sql = "SELECT COALESCE(MAX(CAST(number AS INTEGER)), 0) + 1 
                  FROM invoices WHERE series = ?";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let mut rows = conn.query(sql, libsql::params![series.to_owned()]).await.map_err(|e| e.to_string())?;
        match rows.next().await.map_err(|e| e.to_string())? {
            Some(row) => {
                let next: i64 = row.get(0).map_err(|e| e.to_string())?;
                Ok(next.to_string())
            }
            None => Ok("1".to_string()),
        }
    }

    async fn get_total_pending(&self) -> Result<f64, String> {
        let sql = "SELECT COALESCE(SUM(total), 0) FROM invoices WHERE status = 'pending'";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let mut rows = conn.query(sql, ()).await.map_err(|e| e.to_string())?;
        match rows.next().await.map_err(|e| e.to_string())? {
            Some(row) => {
                let total: f64 = row.get(0).map_err(|e| e.to_string())?;
                Ok(total)
            }
            None => Ok(0.0),
        }
    }

    async fn get_total_by_status(&self, status: InvoiceStatus) -> Result<f64, String> {
        let sql = "SELECT COALESCE(SUM(total), 0) FROM invoices WHERE status = ?";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let mut rows = conn.query(sql, libsql::params![status.as_str().to_string()]).await.map_err(|e| e.to_string())?;
        match rows.next().await.map_err(|e| e.to_string())? {
            Some(row) => {
                let total: f64 = row.get(0).map_err(|e| e.to_string())?;
                Ok(total)
            }
            None => Ok(0.0),
        }
    }
}

/// SQLite implementation of InvoiceLineRepository
#[derive(Clone)]
pub struct SqliteInvoiceLineRepository;

impl SqliteInvoiceLineRepository {
    pub fn new() -> Self {
        Self
    }

    fn row_to_invoice_line(row: &libsql::Row) -> Result<InvoiceLine, String> {
        let created_str: String = row.get(5).map_err(|e| e.to_string())?;

        Ok(InvoiceLine {
            id: row.get(0).map_err(|e| e.to_string())?,
            invoice_id: row.get(1).map_err(|e| e.to_string())?,
            description: row.get(2).map_err(|e| e.to_string())?,
            quantity: row.get(3).map_err(|e| e.to_string())?,
            unit_price: row.get(4).map_err(|e| e.to_string())?,
            total: row.get(6).map_err(|e| e.to_string())?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

impl Default for SqliteInvoiceLineRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InvoiceLineRepository for SqliteInvoiceLineRepository {
    async fn create(&self, line: InvoiceLine) -> Result<InvoiceLine, String> {
        let sql = "INSERT INTO invoice_lines (
                      id, invoice_id, description, quantity, unit_price, total, created_at
                  ) VALUES (?, ?, ?, ?, ?, ?, ?)";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        conn.execute(
            sql,
            libsql::params![
                line.id.clone(),
                line.invoice_id.clone(),
                line.description.clone(),
                line.quantity,
                line.unit_price,
                line.total,
                line.created_at.to_rfc3339(),
            ],
        )
        .await
        .map_err(|e| e.to_string())?;

        Ok(line)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<InvoiceLine>, String> {
        let sql = "SELECT id, invoice_id, description, quantity, unit_price, total, created_at
                  FROM invoice_lines WHERE id = ?";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let mut rows = conn.query(sql, libsql::params![id.to_owned()]).await.map_err(|e| e.to_string())?;
        match rows.next().await.map_err(|e| e.to_string())? {
            Some(row) => Ok(Some(Self::row_to_invoice_line(&row)?)),
            None => Ok(None),
        }
    }

    async fn get_by_invoice(&self, invoice_id: &str) -> Result<Vec<InvoiceLine>, String> {
        let sql = "SELECT id, invoice_id, description, quantity, unit_price, total, created_at
                  FROM invoice_lines WHERE invoice_id = ? ORDER BY id";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let mut rows = conn.query(sql, libsql::params![invoice_id.to_owned()]).await.map_err(|e| e.to_string())?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
            results.push(Self::row_to_invoice_line(&row)?);
        }
        Ok(results)
    }

    async fn update(&self, line: InvoiceLine) -> Result<InvoiceLine, String> {
        let sql = "UPDATE invoice_lines 
                  SET description = ?, quantity = ?, unit_price = ?, total = ?
                  WHERE id = ?";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let affected = conn
            .execute(
                sql,
                libsql::params![
                    line.description.clone(),
                    line.quantity,
                    line.unit_price,
                    line.total,
                    line.id.clone(),
                ],
            )
            .await
            .map_err(|e| e.to_string())?;

        if affected == 0 {
            return Err(format!("InvoiceLine not found: {}", line.id));
        }

        Ok(line)
    }

    async fn delete(&self, id: &str) -> Result<bool, String> {
        let sql = "DELETE FROM invoice_lines WHERE id = ?";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let affected = conn.execute(sql, libsql::params![id.to_owned()]).await.map_err(|e| e.to_string())?;
        Ok(affected > 0)
    }

    async fn delete_by_invoice(&self, invoice_id: &str) -> Result<bool, String> {
        let sql = "DELETE FROM invoice_lines WHERE invoice_id = ?";

        let conn = local_db::get_db().connect().map_err(|e| e.to_string())?;
        let affected = conn.execute(sql, libsql::params![invoice_id.to_owned()]).await.map_err(|e| e.to_string())?;
        Ok(affected > 0)
    }
}
