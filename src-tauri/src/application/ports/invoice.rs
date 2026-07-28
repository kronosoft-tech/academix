//! Invoice Repository Port
//!
//! Port interface for invoice repository operations.

use async_trait::async_trait;
use crate::domain::entities::invoice::{Invoice, InvoiceLine, InvoiceStatus};

/// Invoice repository trait (port)
#[async_trait]
pub trait InvoiceRepository: Send + Sync {
    /// Create a new invoice
    async fn create(&self, invoice: Invoice) -> Result<Invoice, String>;

    /// Get invoice by ID
    async fn get_by_id(&self, id: &str) -> Result<Option<Invoice>, String>;

    /// Get invoice by series and number
    async fn get_by_series_number(&self, series: &str, number: &str) -> Result<Option<Invoice>, String>;

    /// List all invoices with optional filters
    async fn list(
        &self,
        status: Option<InvoiceStatus>,
        client_ruc: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<Vec<Invoice>, String>;

    /// List invoices by client RUC
    async fn list_by_client(&self, client_ruc: &str) -> Result<Vec<Invoice>, String>;

    /// Update invoice
    async fn update(&self, invoice: Invoice) -> Result<Invoice, String>;

    /// Delete invoice (soft delete - cancel)
    async fn delete(&self, id: &str) -> Result<bool, String>;

    /// Get next invoice number for series
    async fn get_next_number(&self, series: &str) -> Result<String, String>;

    /// Get total pending amount
    async fn get_total_pending(&self) -> Result<f64, String>;

    /// Get total by status
    async fn get_total_by_status(&self, status: InvoiceStatus) -> Result<f64, String>;
}

/// Invoice line repository trait (port)
#[async_trait]
pub trait InvoiceLineRepository: Send + Sync {
    /// Create a new invoice line
    async fn create(&self, line: InvoiceLine) -> Result<InvoiceLine, String>;

    /// Get invoice line by ID
    async fn get_by_id(&self, id: &str) -> Result<Option<InvoiceLine>, String>;

    /// Get lines by invoice ID
    async fn get_by_invoice(&self, invoice_id: &str) -> Result<Vec<InvoiceLine>, String>;

    /// Update invoice line
    async fn update(&self, line: InvoiceLine) -> Result<InvoiceLine, String>;

    /// Delete invoice line
    async fn delete(&self, id: &str) -> Result<bool, String>;

    /// Delete all lines for an invoice
    async fn delete_by_invoice(&self, invoice_id: &str) -> Result<bool, String>;
}
