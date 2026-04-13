//! Payment DTOs

use serde::{Deserialize, Serialize};

/// Payment delinquency status (different from domain PaymentStatus)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PaymentDelinquencyStatus {
    #[serde(rename = "current")]
    Current,
    #[serde(rename = "delinquent")]
    Delinquent,
    #[serde(rename = "ahead")]
    Ahead,
}

/// Create payment request
#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub student_id: String,
    pub group_id: String,
    pub amount: f64,
    pub method: Option<String>,
    pub due_date: String,
    pub description: Option<String>,
}

/// Update payment status request
#[derive(Debug, Deserialize)]
pub struct UpdatePaymentRequest {
    pub status: Option<String>,
}

/// Payment status DTO
#[derive(Debug, Serialize)]
pub struct PaymentStatusDto {
    pub student_id: String,
    pub student_name: String,
    pub group_name: String,
    pub group_id: String,
    pub due_date: String,
    pub next_payment_date: Option<String>,
    pub status: PaymentDelinquencyStatus,
    pub days_delayed: i32,
    pub total_paid: f64,
    pub course_price: f64,
    pub debt_amount: f64,
    pub months_paid: i32,
}

/// Payment DTO
#[derive(Debug, Serialize)]
pub struct PaymentDto {
    pub id: String,
    pub student_id: String,
    pub group_id: String,
    pub amount: f64,
    pub method: String,
    pub status: String, // Uses domain PaymentStatus (pending, paid, etc.)
    pub due_date: String,
    pub paid_at: Option<String>,
    pub description: Option<String>,
}
