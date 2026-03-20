//! Payment DTOs

use serde::{Deserialize, Serialize};

/// Create payment request
#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub student_id: String,
    pub group_id: String,
    pub amount: f64,
    pub method: Option<String>,
    pub description: Option<String>,
}

/// Update payment status request
#[derive(Debug, Deserialize)]
pub struct UpdatePaymentRequest {
    pub status: Option<String>,
}

/// Payment DTO
#[derive(Debug, Serialize)]
pub struct PaymentDto {
    pub id: String,
    pub student_id: String,
    pub group_id: String,
    pub amount: f64,
    pub method: String,
    pub status: String,
    pub paid_at: Option<String>,
    pub description: Option<String>,
}
