//! Invoice Commands - Tauri Command Handlers
//!
//! Expose invoice operations to the frontend.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::invoice::RegisterPaymentRequest;
use crate::application::dto::invoice::{
    CreateInvoiceLineRequest, InvoiceDto, InvoiceSummary, InvoiceWithLinesDto,
};
use crate::application::use_cases::InvoiceService;
use crate::domain::entities::invoice::InvoicePaymentMethod;
use crate::infrastructure::repositories::{SqliteInvoiceLineRepository, SqliteInvoiceRepository};

/// Invoice service state type
pub type InvoiceServiceState = InvoiceService<SqliteInvoiceRepository, SqliteInvoiceLineRepository>;

/// Create invoice request
#[derive(Debug, Deserialize)]
pub struct CreateInvoiceCommand {
    pub client_name: String,
    pub client_ruc: String,
    pub client_address: Option<String>,
    pub emission_date: String,
    pub due_date: String,
    pub created_by: String,
    pub lines: Vec<InvoiceLineCommand>,
}

/// Invoice line command
#[derive(Debug, Deserialize)]
pub struct InvoiceLineCommand {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
}

/// Register payment request
#[derive(Debug, Deserialize)]
pub struct RegisterPaymentCommand {
    pub payment_method: String,
    pub paid_date: String,
}

/// Invoice response
#[derive(Debug, Serialize)]
pub struct InvoiceCommandResponse {
    pub success: bool,
    pub data: Option<InvoiceWithLinesDto>,
    pub error: Option<String>,
}

/// Invoice list response
#[derive(Debug, Serialize)]
pub struct InvoiceListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<InvoiceDto>>,
    pub error: Option<String>,
}

/// Invoice summary response
#[derive(Debug, Serialize)]
pub struct InvoiceSummaryCommandResponse {
    pub success: bool,
    pub data: Option<InvoiceSummary>,
    pub error: Option<String>,
}

/// Create a new invoice with lines
#[tauri::command]
pub fn create_invoice(
    state: State<InvoiceServiceState>,
    request: CreateInvoiceCommand,
) -> InvoiceCommandResponse {
    let lines: Vec<CreateInvoiceLineRequest> = request
        .lines
        .iter()
        .map(|l| CreateInvoiceLineRequest {
            description: l.description.clone(),
            quantity: l.quantity,
            unit_price: l.unit_price,
        })
        .collect();

    let create_request = crate::application::dto::invoice::CreateInvoiceRequest {
        client_name: request.client_name,
        client_ruc: request.client_ruc,
        client_address: request.client_address,
        emission_date: request.emission_date,
        due_date: request.due_date,
        lines,
        created_by: request.created_by,
    };

    match state.create_invoice(create_request) {
        Ok(invoice) => InvoiceCommandResponse {
            success: true,
            data: Some(invoice),
            error: None,
        },
        Err(e) => InvoiceCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}

/// Get invoice by ID with lines
#[tauri::command]
pub fn get_invoice(state: State<InvoiceServiceState>, id: String) -> InvoiceCommandResponse {
    match state.get_invoice(&id) {
        Ok(invoice) => InvoiceCommandResponse {
            success: true,
            data: invoice,
            error: None,
        },
        Err(e) => InvoiceCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}

/// List invoices with filters
#[tauri::command]
pub fn list_invoices(
    state: State<InvoiceServiceState>,
    status: Option<String>,
    client_ruc: Option<String>,
) -> InvoiceListCommandResponse {
    use crate::domain::entities::invoice::InvoiceStatus;

    let status_filter = status.and_then(|s| InvoiceStatus::from_str(&s));

    match state.list_invoices(status_filter, client_ruc.as_deref()) {
        Ok(invoices) => InvoiceListCommandResponse {
            success: true,
            data: Some(invoices),
            error: None,
        },
        Err(e) => InvoiceListCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}

/// Register payment for an invoice
#[tauri::command]
pub fn register_payment(
    state: State<InvoiceServiceState>,
    invoice_id: String,
    request: RegisterPaymentCommand,
) -> InvoiceCommandResponse {
    let request = RegisterPaymentRequest {
        payment_method: InvoicePaymentMethod::from_str(&request.payment_method)
            .unwrap_or(InvoicePaymentMethod::Cash),
        paid_date: request.paid_date,
    };

    match state.register_payment(&invoice_id, request) {
        Ok(_invoice) => {
            // Get the invoice with lines after payment
            match state.get_invoice(&invoice_id) {
                Ok(invoice_with_lines) => InvoiceCommandResponse {
                    success: true,
                    data: invoice_with_lines,
                    error: None,
                },
                Err(e) => InvoiceCommandResponse {
                    success: false,
                    data: None,
                    error: Some(e),
                },
            }
        }
        Err(e) => InvoiceCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}

/// Cancel an invoice
#[tauri::command]
pub fn cancel_invoice(state: State<InvoiceServiceState>, id: String) -> InvoiceCommandResponse {
    match state.cancel_invoice(&id) {
        Ok(invoice_dto) => {
            // Need to convert InvoiceDto to InvoiceWithLinesDto
            match state.get_invoice(&id) {
                Ok(invoice_with_lines) => InvoiceCommandResponse {
                    success: true,
                    data: invoice_with_lines,
                    error: None,
                },
                Err(_) => {
                    // Return basic invoice if we can't get lines
                    InvoiceCommandResponse {
                        success: true,
                        data: Some(InvoiceWithLinesDto {
                            invoice: invoice_dto,
                            lines: vec![],
                        }),
                        error: None,
                    }
                }
            }
        }
        Err(e) => InvoiceCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}

/// Get invoice summary
#[tauri::command]
pub fn get_invoice_summary(state: State<InvoiceServiceState>) -> InvoiceSummaryCommandResponse {
    match state.get_summary() {
        Ok(summary) => InvoiceSummaryCommandResponse {
            success: true,
            data: Some(summary),
            error: None,
        },
        Err(e) => InvoiceSummaryCommandResponse {
            success: false,
            data: None,
            error: Some(e),
        },
    }
}
