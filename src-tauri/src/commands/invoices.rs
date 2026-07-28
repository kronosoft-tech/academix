use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::invoice::RegisterPaymentRequest;
use crate::application::dto::invoice::{
    CreateInvoiceLineRequest, InvoiceDto, InvoiceSummary, InvoiceWithLinesDto,
};
use crate::application::use_cases::InvoiceService;
use crate::domain::entities::invoice::InvoicePaymentMethod;
use crate::infrastructure::repositories::{SqliteInvoiceLineRepository, SqliteInvoiceRepository};

pub type InvoiceServiceState = InvoiceService<SqliteInvoiceRepository, SqliteInvoiceLineRepository>;

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

#[derive(Debug, Deserialize)]
pub struct InvoiceLineCommand {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
}

#[derive(Debug, Deserialize)]
pub struct RegisterPaymentCommand {
    pub payment_method: String,
    pub paid_date: String,
}

#[derive(Debug, Serialize)]
pub struct InvoiceCommandResponse {
    pub success: bool,
    pub data: Option<InvoiceWithLinesDto>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InvoiceListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<InvoiceDto>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InvoiceSummaryCommandResponse {
    pub success: bool,
    pub data: Option<InvoiceSummary>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn create_invoice(
    state: State<'_, InvoiceServiceState>,
    request: CreateInvoiceCommand,
) -> Result<InvoiceCommandResponse, String> {
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

    match state.create_invoice(create_request).await {
        Ok(invoice) => Ok(InvoiceCommandResponse {
            success: true,
            data: Some(invoice),
            error: None,
        }),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn get_invoice(state: State<'_, InvoiceServiceState>, id: String) -> Result<InvoiceCommandResponse, String> {
    match state.get_invoice(&id).await {
        Ok(invoice) => Ok(InvoiceCommandResponse {
            success: true,
            data: invoice,
            error: None,
        }),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn list_invoices(
    state: State<'_, InvoiceServiceState>,
    status: Option<String>,
    client_ruc: Option<String>,
) -> Result<InvoiceListCommandResponse, String> {
    use crate::domain::entities::invoice::InvoiceStatus;

    let status_filter = status.and_then(|s| InvoiceStatus::from_str(&s));

    match state.list_invoices(status_filter, client_ruc.as_deref()).await {
        Ok(invoices) => Ok(InvoiceListCommandResponse {
            success: true,
            data: Some(invoices),
            error: None,
        }),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn register_payment(
    state: State<'_, InvoiceServiceState>,
    invoice_id: String,
    request: RegisterPaymentCommand,
) -> Result<InvoiceCommandResponse, String> {
    let request = RegisterPaymentRequest {
        payment_method: InvoicePaymentMethod::from_str(&request.payment_method)
            .unwrap_or(InvoicePaymentMethod::Cash),
        paid_date: request.paid_date,
    };

    match state.register_payment(&invoice_id, request).await {
        Ok(_invoice) => {
            match state.get_invoice(&invoice_id).await {
                Ok(invoice_with_lines) => Ok(InvoiceCommandResponse {
                    success: true,
                    data: invoice_with_lines,
                    error: None,
                }),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn cancel_invoice(state: State<'_, InvoiceServiceState>, id: String) -> Result<InvoiceCommandResponse, String> {
    match state.cancel_invoice(&id).await {
        Ok(invoice_dto) => {
            match state.get_invoice(&id).await {
                Ok(invoice_with_lines) => Ok(InvoiceCommandResponse {
                    success: true,
                    data: invoice_with_lines,
                    error: None,
                }),
                Err(_) => {
                    Ok(InvoiceCommandResponse {
                        success: true,
                        data: Some(InvoiceWithLinesDto {
                            invoice: invoice_dto,
                            lines: vec![],
                        }),
                        error: None,
                    })
                }
            }
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn get_invoice_summary(state: State<'_, InvoiceServiceState>) -> Result<InvoiceSummaryCommandResponse, String> {
    match state.get_summary().await {
        Ok(summary) => Ok(InvoiceSummaryCommandResponse {
            success: true,
            data: Some(summary),
            error: None,
        }),
        Err(e) => Err(e),
    }
}
