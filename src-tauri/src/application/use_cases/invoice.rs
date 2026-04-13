//! Invoice Service
//!
//! Use case for invoice operations - create, list, register payments.

use crate::application::dto::invoice::{
    CreateInvoiceRequest, InvoiceDto, InvoiceLineDto, InvoiceSummary, InvoiceWithLinesDto,
    RegisterPaymentRequest,
};
use crate::application::ports::invoice::{InvoiceLineRepository, InvoiceRepository};
use crate::domain::entities::invoice::{
    calculate_igv, calculate_total, Invoice, InvoiceLine, InvoicePaymentMethod, InvoiceStatus,
};
use chrono::{DateTime, Utc};

/// Invoice service - orchestrates invoice operations
pub struct InvoiceService<R: InvoiceRepository, L: InvoiceLineRepository> {
    invoice_repo: R,
    line_repo: L,
}

impl<R: InvoiceRepository, L: InvoiceLineRepository> InvoiceService<R, L> {
    pub fn new(invoice_repo: R, line_repo: L) -> Self {
        Self {
            invoice_repo,
            line_repo,
        }
    }

    /// Create a new invoice with lines
    pub fn create_invoice(
        &self,
        request: CreateInvoiceRequest,
    ) -> Result<InvoiceWithLinesDto, String> {
        // Validate required fields
        if request.client_name.trim().is_empty() {
            return Err("Client name is required".to_string());
        }
        if request.client_ruc.trim().is_empty() {
            return Err("Client RUC is required".to_string());
        }
        if request.lines.is_empty() {
            return Err("At least one line item is required".to_string());
        }

        // Validate RUC format (11 digits)
        if !crate::domain::entities::invoice::validate_ruc(&request.client_ruc) {
            return Err("Invalid RUC format (must be 11 digits)".to_string());
        }

        // Parse dates
        let emission_date = DateTime::parse_from_rfc3339(&request.emission_date)
            .map_err(|e| format!("Invalid emission_date: {}", e))?
            .with_timezone(&Utc);
        let due_date = DateTime::parse_from_rfc3339(&request.due_date)
            .map_err(|e| format!("Invalid due_date: {}", e))?
            .with_timezone(&Utc);

        // Get next invoice number
        let series = "F001".to_string(); // Default series
        let number = self.invoice_repo.get_next_number(&series)?;

        // Calculate totals
        let subtotal: f64 = request
            .lines
            .iter()
            .map(|l| l.quantity * l.unit_price)
            .sum();
        let igv = calculate_igv(subtotal);
        let total = calculate_total(subtotal);

        // Create invoice
        let mut invoice = Invoice::new(
            String::new(),
            series,
            number,
            request.client_name,
            request.client_ruc,
            emission_date,
            due_date,
            request.created_by,
        );

        if let Some(address) = request.client_address {
            invoice.set_client_address(address);
        }

        invoice.calculate_totals(subtotal);

        // Save invoice
        let invoice = self.invoice_repo.create(invoice)?;

        // Create invoice lines
        let mut line_dtos = Vec::new();
        for line_req in &request.lines {
            if line_req.description.trim().is_empty() {
                return Err("Line description is required".to_string());
            }
            if line_req.quantity <= 0.0 {
                return Err("Line quantity must be greater than 0".to_string());
            }
            if line_req.unit_price <= 0.0 {
                return Err("Line unit price must be greater than 0".to_string());
            }

            let line = InvoiceLine::new(
                String::new(),
                invoice.id.clone(),
                line_req.description.clone(),
                line_req.quantity,
                line_req.unit_price,
            );

            let saved_line = self.line_repo.create(line)?;
            line_dtos.push(InvoiceLineDto::from(saved_line));
        }

        Ok(InvoiceWithLinesDto {
            invoice: InvoiceDto::from(invoice),
            lines: line_dtos,
        })
    }

    /// Get invoice by ID with lines
    pub fn get_invoice(&self, id: &str) -> Result<Option<InvoiceWithLinesDto>, String> {
        let invoice = self.invoice_repo.get_by_id(id)?;

        if let Some(invoice) = invoice {
            let lines = self.line_repo.get_by_invoice(&invoice.id)?;
            let line_dtos: Vec<InvoiceLineDto> =
                lines.into_iter().map(InvoiceLineDto::from).collect();

            Ok(Some(InvoiceWithLinesDto {
                invoice: InvoiceDto::from(invoice),
                lines: line_dtos,
            }))
        } else {
            Ok(None)
        }
    }

    /// List invoices with filters
    pub fn list_invoices(
        &self,
        status: Option<InvoiceStatus>,
        client_ruc: Option<&str>,
    ) -> Result<Vec<InvoiceDto>, String> {
        let invoices = self.invoice_repo.list(status, client_ruc, None, None)?;
        Ok(invoices.into_iter().map(InvoiceDto::from).collect())
    }

    /// Register payment for an invoice
    pub fn register_payment(
        &self,
        id: &str,
        request: RegisterPaymentRequest,
    ) -> Result<InvoiceDto, String> {
        let mut invoice = self
            .invoice_repo
            .get_by_id(id)?
            .ok_or_else(|| format!("Invoice not found: {}", id))?;

        if invoice.status == InvoiceStatus::Paid {
            return Err("Invoice is already paid".to_string());
        }
        if invoice.status == InvoiceStatus::Cancelled {
            return Err("Cannot register payment for a cancelled invoice".to_string());
        }

        let paid_date = DateTime::parse_from_rfc3339(&request.paid_date)
            .map_err(|e| format!("Invalid paid_date: {}", e))?
            .with_timezone(&Utc);

        invoice.register_payment(request.payment_method, paid_date);

        let updated = self.invoice_repo.update(invoice)?;
        Ok(InvoiceDto::from(updated))
    }

    /// Cancel an invoice
    pub fn cancel_invoice(&self, id: &str) -> Result<InvoiceDto, String> {
        let mut invoice = self
            .invoice_repo
            .get_by_id(id)?
            .ok_or_else(|| format!("Invoice not found: {}", id))?;

        if invoice.status == InvoiceStatus::Paid {
            return Err("Cannot cancel a paid invoice".to_string());
        }

        invoice.cancel();

        let updated = self.invoice_repo.update(invoice)?;
        Ok(InvoiceDto::from(updated))
    }

    /// Get invoice summary for dashboard
    pub fn get_summary(&self) -> Result<InvoiceSummary, String> {
        let invoices = self.invoice_repo.list(None, None, None, None)?;

        let total_invoiced: f64 = invoices.iter().map(|i| i.total).sum();
        let total_pending: f64 = invoices
            .iter()
            .filter(|i| i.status == InvoiceStatus::Pending)
            .map(|i| i.total)
            .sum();
        let total_paid: f64 = invoices
            .iter()
            .filter(|i| i.status == InvoiceStatus::Paid)
            .map(|i| i.total)
            .sum();
        let total_overdue: f64 = invoices
            .iter()
            .filter(|i| {
                i.status == InvoiceStatus::Overdue
                    || (i.status == InvoiceStatus::Pending && i.is_overdue())
            })
            .map(|i| i.total)
            .sum();

        let invoice_count = invoices.len() as i64;
        let pending_count = invoices
            .iter()
            .filter(|i| i.status == InvoiceStatus::Pending)
            .count() as i64;
        let paid_count = invoices
            .iter()
            .filter(|i| i.status == InvoiceStatus::Paid)
            .count() as i64;
        let overdue_count = invoices
            .iter()
            .filter(|i| i.status == InvoiceStatus::Overdue)
            .count() as i64;

        Ok(InvoiceSummary {
            total_invoiced,
            total_pending,
            total_paid,
            total_overdue,
            invoice_count,
            pending_count,
            paid_count,
            overdue_count,
        })
    }
}
