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
#[derive(Debug, Deserialize, Default)]
pub struct UpdatePaymentRequest {
    pub status: Option<String>,
    pub reference: Option<String>,
    pub paid_date: Option<String>,
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
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "studentId")]
    pub student_id: String,
    #[serde(rename = "groupId")]
    pub group_id: String,
    #[serde(rename = "amount")]
    pub amount: f64,
    #[serde(rename = "method")]
    pub method: String,
    #[serde(rename = "status")]
    pub status: String, // Uses domain PaymentStatus (pending, paid, etc.)
    #[serde(rename = "dueDate")]
    pub due_date: String,
    #[serde(rename = "paidAt")]
    pub paid_at: Option<String>,
    #[serde(rename = "description")]
    pub description: Option<String>,
}
