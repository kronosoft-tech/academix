//! Payment Commands
//!
//! Tauri commands for payment management.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::application::dto::{
    CreatePaymentRequest, PaymentDto, PaymentStatusDto, UpdatePaymentRequest,
};
use crate::application::use_cases::PaymentService;
use crate::infrastructure::repositories::{
    SqliteCourseRepository, SqliteGroupRepository, SqlitePaymentRepository,
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
}

/// Update payment request payload
#[derive(Debug, Deserialize)]
pub struct UpdatePaymentCommand {
    pub status: Option<String>,
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
    match state.create(CreatePaymentRequest {
        student_id: request.student_id,
        group_id: request.group_id,
        amount: request.amount,
        method: request.method,
        due_date: request.due_date.unwrap_or_default(),
        description: request.description,
    }) {
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
    match state.list() {
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

/// Update payment
#[tauri::command]
pub fn update_payment(
    state: State<PaymentServiceState>,
    id: String,
    request: UpdatePaymentCommand,
) -> PaymentCommandResponse {
    match state.update(
        &id,
        UpdatePaymentRequest {
            status: request.status,
        },
    ) {
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

/// Payment status command response
#[derive(Debug, Serialize)]
pub struct PaymentStatusCommandResponse {
    pub success: bool,
    pub data: Option<PaymentStatusDto>,
    pub error: Option<String>,
}

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
