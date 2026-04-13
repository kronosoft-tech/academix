//! Invoice DTOs

use crate::domain::entities::invoice::{InvoicePaymentMethod, InvoiceStatus};
use serde::{Deserialize, Serialize};

/// Create invoice request
#[derive(Debug, Deserialize)]
pub struct CreateInvoiceRequest {
    pub client_name: String,
    pub client_ruc: String,
    pub client_address: Option<String>,
    pub emission_date: String,
    pub due_date: String,
    pub lines: Vec<CreateInvoiceLineRequest>,
    pub created_by: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceLineRequest {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
}

/// Update invoice request
#[derive(Debug, Deserialize)]
pub struct UpdateInvoiceRequest {
    pub client_name: Option<String>,
    pub client_ruc: Option<String>,
    pub client_address: Option<String>,
    pub status: Option<InvoiceStatus>,
}

/// Register payment request
#[derive(Debug, Deserialize)]
pub struct RegisterPaymentRequest {
    pub payment_method: InvoicePaymentMethod,
    pub paid_date: String,
}

/// Invoice response
#[derive(Debug, Serialize)]
pub struct InvoiceDto {
    pub id: String,
    pub series: String,
    pub number: String,
    pub full_number: String,
    pub client_name: String,
    pub client_ruc: String,
    pub client_address: Option<String>,
    pub emission_date: String,
    pub due_date: String,
    pub subtotal: f64,
    pub igv: f64,
    pub total: f64,
    pub formatted_total: String,
    pub status: InvoiceStatus,
    pub payment_method: Option<InvoicePaymentMethod>,
    pub paid_date: Option<String>,
    pub created_at: String,
    pub created_by: String,
}

impl From<crate::domain::entities::invoice::Invoice> for InvoiceDto {
    fn from(inv: crate::domain::entities::invoice::Invoice) -> Self {
        let full = inv.full_number();
        let formatted = inv.formatted_total();
        Self {
            id: inv.id,
            series: inv.series,
            number: inv.number,
            full_number: full,
            client_name: inv.client_name,
            client_ruc: inv.client_ruc,
            client_address: inv.client_address,
            emission_date: inv.emission_date.to_rfc3339(),
            due_date: inv.due_date.to_rfc3339(),
            subtotal: inv.subtotal,
            igv: inv.igv,
            total: inv.total,
            formatted_total: formatted,
            status: inv.status,
            payment_method: inv.payment_method,
            paid_date: inv.paid_date.map(|d| d.to_rfc3339()),
            created_at: inv.created_at.to_rfc3339(),
            created_by: inv.created_by,
        }
    }
}

/// Invoice line response
#[derive(Debug, Serialize)]
pub struct InvoiceLineDto {
    pub id: String,
    pub invoice_id: String,
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub total: f64,
}

impl From<crate::domain::entities::invoice::InvoiceLine> for InvoiceLineDto {
    fn from(line: crate::domain::entities::invoice::InvoiceLine) -> Self {
        Self {
            id: line.id,
            invoice_id: line.invoice_id,
            description: line.description,
            quantity: line.quantity,
            unit_price: line.unit_price,
            total: line.total,
        }
    }
}

/// Invoice with lines
#[derive(Debug, Serialize)]
pub struct InvoiceWithLinesDto {
    pub invoice: InvoiceDto,
    pub lines: Vec<InvoiceLineDto>,
}

/// Invoice summary for dashboard
#[derive(Debug, Serialize)]
pub struct InvoiceSummary {
    pub total_invoiced: f64,
    pub total_pending: f64,
    pub total_paid: f64,
    pub total_overdue: f64,
    pub invoice_count: i64,
    pub pending_count: i64,
    pub paid_count: i64,
    pub overdue_count: i64,
}
