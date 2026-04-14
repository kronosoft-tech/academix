//! Invoice Commands - Tauri Command Handlers
//!
//! Expose invoice operations to the frontend.

use crate::application::dto::invoice::{
    CreateInvoiceRequest, InvoiceDto, InvoiceSummary, InvoiceWithLinesDto, RegisterPaymentRequest,
};
use crate::application::use_cases::InvoiceService;
use crate::infrastructure::repositories::{
    InMemoryInvoiceLineRepository, InMemoryInvoiceRepository,
};
use tauri::command;

/// Create a new invoice with lines
#[command]
pub fn create_invoice(request: CreateInvoiceRequest) -> Result<InvoiceWithLinesDto, String> {
    let invoice_repo = InMemoryInvoiceRepository::new();
    let line_repo = InMemoryInvoiceLineRepository::new();

    let service = InvoiceService::new(invoice_repo, line_repo);
    service.create_invoice(request)
}

/// Get invoice by ID with lines
#[command]
pub fn get_invoice(id: String) -> Result<Option<InvoiceWithLinesDto>, String> {
    let invoice_repo = InMemoryInvoiceRepository::new();
    let line_repo = InMemoryInvoiceLineRepository::new();

    let service = InvoiceService::new(invoice_repo, line_repo);
    service.get_invoice(&id)
}

/// List invoices with filters
#[command]
pub fn list_invoices(
    status: Option<String>,
    client_ruc: Option<String>,
) -> Result<Vec<InvoiceDto>, String> {
    use crate::domain::entities::invoice::InvoiceStatus;

    let status_filter = status.and_then(|s| InvoiceStatus::from_str(&s));

    let invoice_repo = InMemoryInvoiceRepository::new();
    let line_repo = InMemoryInvoiceLineRepository::new();

    let service = InvoiceService::new(invoice_repo, line_repo);
    service.list_invoices(status_filter, client_ruc.as_deref())
}

/// Register payment for an invoice
#[command]
pub fn register_payment(id: String, request: RegisterPaymentRequest) -> Result<InvoiceDto, String> {
    let invoice_repo = InMemoryInvoiceRepository::new();
    let line_repo = InMemoryInvoiceLineRepository::new();

    let service = InvoiceService::new(invoice_repo, line_repo);
    service.register_payment(&id, request)
}

/// Cancel an invoice
#[command]
pub fn cancel_invoice(id: String) -> Result<InvoiceDto, String> {
    let invoice_repo = InMemoryInvoiceRepository::new();
    let line_repo = InMemoryInvoiceLineRepository::new();

    let service = InvoiceService::new(invoice_repo, line_repo);
    service.cancel_invoice(&id)
}

/// Get invoice summary for dashboard
#[command]
pub fn get_invoice_summary() -> Result<InvoiceSummary, String> {
    let invoice_repo = InMemoryInvoiceRepository::new();
    let line_repo = InMemoryInvoiceLineRepository::new();

    let service = InvoiceService::new(invoice_repo, line_repo);
    service.get_summary()
}
