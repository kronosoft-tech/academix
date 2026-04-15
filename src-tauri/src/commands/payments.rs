//! Payment Commands
//!
//! Tauri commands for payment management.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::accounting::CreateEntryRequest;
use crate::application::dto::{
    CreatePaymentRequest, PaymentDto, PaymentStatusDto, UpdatePaymentRequest,
};
use crate::application::use_cases::AccountingService;
use crate::application::use_cases::PaymentService;
use crate::infrastructure::repositories::{
    SqliteAccountCategoryRepository, SqliteAccountingEntryRepository, SqliteCourseRepository,
    SqliteGroupRepository, SqlitePaymentRepository,
};

pub type PaymentServiceState =
    PaymentService<SqlitePaymentRepository, SqliteGroupRepository, SqliteCourseRepository>;

/// Create payment request payload
#[derive(Debug, Deserialize)]
pub struct CreatePaymentCommand {
    pub student_id: String,
    pub group_id: String,
    pub amount: f64,
    pub method: Option<String>,
    pub due_date: Option<String>,
    pub description: Option<String>,
    pub paid: Option<bool>, // If true, payment is created as "paid" immediately
}

/// Update payment request payload
#[derive(Debug, Deserialize, Default)]
pub struct UpdatePaymentCommand {
    pub status: Option<String>,
    pub reference: Option<String>,
    pub paid_date: Option<String>,
}

/// Payment response payload
#[derive(Debug, Serialize)]
pub struct PaymentCommandResponse {
    pub success: bool,
    pub data: Option<PaymentDto>,
    pub error: Option<String>,
}

/// Payment list response
#[derive(Debug, Serialize)]
pub struct PaymentListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<PaymentDto>>,
    pub error: Option<String>,
}

/// Create payment command
#[tauri::command]
pub fn create_payment(
    state: State<PaymentServiceState>,
    request: CreatePaymentCommand,
) -> PaymentCommandResponse {
    eprintln!(
        "[DEBUG] create_payment called with: student_id={}, group_id={}, amount={}",
        request.student_id, request.group_id, request.amount
    );

    match state.create(CreatePaymentRequest {
        student_id: request.student_id,
        group_id: request.group_id,
        amount: request.amount,
        method: request.method,
        due_date: request.due_date.unwrap_or_default(),
        description: request.description,
        paid: request.paid,
    }) {
        Ok(payment) => {
            eprintln!("[DEBUG] Payment created successfully: {}", payment.id);
            PaymentCommandResponse {
                success: true,
                data: Some(payment),
                error: None,
            }
        }
        Err(e) => {
            eprintln!("[DEBUG] Payment creation failed: {}", e);
            PaymentCommandResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }
        }
    }
}

/// Get payment by ID
#[tauri::command]
pub fn get_payment(state: State<PaymentServiceState>, id: String) -> PaymentCommandResponse {
    match state.get_by_id(&id) {
        Ok(payment) => PaymentCommandResponse {
            success: true,
            data: Some(payment),
            error: None,
        },
        Err(e) => PaymentCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// List all payments
#[tauri::command]
pub fn list_payments(state: State<PaymentServiceState>) -> PaymentListCommandResponse {
    eprintln!("[DEBUG] list_payments called");
    match state.list() {
        Ok(payments) => {
            eprintln!("[DEBUG] list_payments returned {} payments", payments.len());
            PaymentListCommandResponse {
                success: true,
                data: Some(payments),
                error: None,
            }
        }
        Err(e) => {
            eprintln!("[DEBUG] list_payments failed: {}", e);
            PaymentListCommandResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            }
        }
    }
}

/// List payments by student
#[tauri::command]
pub fn list_payments_by_student(
    state: State<PaymentServiceState>,
    student_id: String,
) -> PaymentListCommandResponse {
    match state.list_by_student(&student_id) {
        Ok(payments) => PaymentListCommandResponse {
            success: true,
            data: Some(payments),
            error: None,
        },
        Err(e) => PaymentListCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Update payment - when status="paid", automatically creates accounting entry
#[tauri::command]
pub fn update_payment(
    state: State<PaymentServiceState>,
    id: String,
    request: UpdatePaymentRequest,
    accounting_entry_state: State<SqliteAccountingEntryRepository>,
    accounting_category_state: State<SqliteAccountCategoryRepository>,
) -> PaymentCommandResponse {
    let update_result = state.update(
        &id,
        UpdatePaymentRequest {
            status: request.status.clone(),
            reference: request.reference.clone(),
            paid_date: request.paid_date.clone(),
        },
    );

    match update_result {
        Ok(payment) => {
            // If status is "paid", automatically create accounting entry
            if request.status.as_deref() == Some("paid") {
                let accounting_service = AccountingService::new(
                    accounting_entry_state.inner().clone(),
                    accounting_category_state.inner().clone(),
                );

                // Create entry: Debit Cash (1105), Credit Income (6115 - Mensualidades)
                let entry_request = CreateEntryRequest {
                    date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                    description: format!("Pago estudiante - {}", id),
                    debit_account: "1105".to_string(),  // Caja
                    credit_account: "6115".to_string(), // Mensualidades
                    amount: payment.amount,
                    entry_type: Some(crate::domain::entities::accounting::EntryType::Automatic),
                    reference: request.reference.or(Some(format!("PAG-{}", &id[..8]))),
                    related_id: Some(payment.id.clone()),
                    related_type: Some("payment".to_string()),
                };

                match accounting_service.create_entry(entry_request, "system".to_string()) {
                    Ok(_entry) => {
                        // Success - payment updated and accounting entry created
                    }
                    Err(e) => {
                        // Payment was updated but accounting entry failed
                        return PaymentCommandResponse {
                            success: true,
                            data: Some(payment),
                            error: Some(format!("Pago actualizado pero error contable: {}", e)),
                        };
                    }
                }
            }

            PaymentCommandResponse {
                success: true,
                data: Some(payment),
                error: None,
            }
        }
        Err(e) => PaymentCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Delete payment
#[tauri::command]
pub fn delete_payment(state: State<PaymentServiceState>, id: String) -> PaymentCommandResponse {
    match state.delete(&id) {
        Ok(()) => PaymentCommandResponse {
            success: true,
            data: None,
            error: None,
        },
        Err(e) => PaymentCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Register payment and create accounting entry automatically
/// This is used when a payment is received - marks it as paid and creates
/// an accounting entry for the income (e.g., monthly fee)
#[tauri::command]
pub fn register_payment_with_income(
    state: State<PaymentServiceState>,
    payment_id: String,
    reference: String,
    accounting_entry_state: State<SqliteAccountingEntryRepository>,
    accounting_category_state: State<SqliteAccountCategoryRepository>,
) -> PaymentCommandResponse {
    // First update payment to "paid" status
    let update_result = state.update(
        &payment_id,
        UpdatePaymentRequest {
            status: Some("paid".to_string()),
            reference: Some(reference.clone()),
            paid_date: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
        },
    );

    match update_result {
        Ok(_payment) => {
            // Now create accounting entry for the income
            let accounting_service = AccountingService::new(
                accounting_entry_state.inner().clone(),
                accounting_category_state.inner().clone(),
            );

            // Create entry: Debit to Cash/Bank (1105), Credit to Income (6115 - Mensualidades)
            let entry_request = CreateEntryRequest {
                date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                description: format!("Pago {}", _payment.student_id),
                debit_account: "1105".to_string(),  // Caja
                credit_account: "6115".to_string(), // Mensualidades
                amount: _payment.amount,
                entry_type: Some(crate::domain::entities::accounting::EntryType::Automatic),
                reference: Some(format!("PAG-{}", reference)),
                related_id: Some(_payment.id.clone()),
                related_type: Some("payment".to_string()),
            };

            match accounting_service.create_entry(entry_request, "system".to_string()) {
                Ok(_entry) => PaymentCommandResponse {
                    success: true,
                    data: Some(_payment),
                    error: None,
                },
                Err(e) => {
                    // Payment was updated but accounting entry failed
                    // Still return success but log the error
                    PaymentCommandResponse {
                        success: true,
                        data: Some(_payment),
                        error: Some(format!(
                            "Warning: Payment updated but accounting entry failed: {}",
                            e
                        )),
                    }
                }
            }
        }
        Err(e) => PaymentCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Payment status command response
#[derive(Debug, Serialize)]
pub struct PaymentStatusCommandResponse {
    pub success: bool,
    pub data: Option<PaymentStatusDto>,
    pub error: Option<String>,
}

/// Get student payment status

/// Payment status list command response
#[derive(Debug, Serialize)]
pub struct PaymentStatusListCommandResponse {
    pub success: bool,
    pub data: Option<Vec<PaymentStatusDto>>,
    pub error: Option<String>,
}

/// Get student payment status
#[tauri::command]
pub fn get_student_payment_status(
    state: State<PaymentServiceState>,
    student_id: String,
    group_id: String,
) -> PaymentStatusCommandResponse {
    match state.calculate_payment_status(&student_id, &group_id) {
        Ok(status) => PaymentStatusCommandResponse {
            success: true,
            data: Some(status),
            error: None,
        },
        Err(e) => PaymentStatusCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}

/// Get all students payment summary
#[tauri::command]
pub fn get_all_students_payment_summary(
    state: State<PaymentServiceState>,
) -> PaymentStatusListCommandResponse {
    match state.get_all_students_payment_summary() {
        Ok(summaries) => PaymentStatusListCommandResponse {
            success: true,
            data: Some(summaries),
            error: None,
        },
        Err(e) => PaymentStatusListCommandResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        },
    }
}
