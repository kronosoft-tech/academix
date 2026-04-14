//! Payment Entity - Domain Model
//!
//! Pure domain entity with no persistence concerns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Payment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Paid,
    Failed,
    Refunded,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Pending => "pending",
            PaymentStatus::Paid => "paid",
            PaymentStatus::Failed => "failed",
            PaymentStatus::Refunded => "refunded",
        }
    }

    pub fn from_str(s: &str) -> Option<PaymentStatus> {
        match s.to_lowercase().as_str() {
            "pending" => Some(PaymentStatus::Pending),
            "paid" => Some(PaymentStatus::Paid),
            "failed" => Some(PaymentStatus::Failed),
            "refunded" => Some(PaymentStatus::Refunded),
            _ => None,
        }
    }
}

/// Payment method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentMethod {
    Cash,
    Card,
    Transfer,
}

impl PaymentMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentMethod::Cash => "cash",
            PaymentMethod::Card => "card",
            PaymentMethod::Transfer => "transfer",
        }
    }

    pub fn from_str(s: &str) -> Option<PaymentMethod> {
        match s.to_lowercase().as_str() {
            "cash" => Some(PaymentMethod::Cash),
            "card" => Some(PaymentMethod::Card),
            "transfer" => Some(PaymentMethod::Transfer),
            _ => None,
        }
    }
}

/// Payment entity - represents a payment for enrollment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub student_id: String,
    pub group_id: String,
    pub amount: f64,
    pub method: PaymentMethod,
    pub status: PaymentStatus,
    pub due_date: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Payment {
    /// Create a new payment (default status is PAID since all payments are confirmed)
    pub fn new(
        id: String,
        student_id: String,
        group_id: String,
        amount: f64,
        method: PaymentMethod,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            student_id,
            group_id,
            amount,
            method,
            status: PaymentStatus::Pending, // Default to Pending until payment is confirmed
            due_date: None,
            paid_at: None, // Not paid yet
            reference: None,
            description: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set due date for the payment
    pub fn set_due_date(&mut self, due_date: String) {
        self.due_date = Some(due_date);
    }

    /// Mark payment as paid
    pub fn mark_paid(&mut self) {
        self.status = PaymentStatus::Paid;
        self.paid_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark payment as failed
    pub fn mark_failed(&mut self) {
        self.status = PaymentStatus::Failed;
        self.updated_at = Utc::now();
    }

    /// Refund payment
    pub fn refund(&mut self) {
        self.status = PaymentStatus::Refunded;
        self.updated_at = Utc::now();
    }

    /// Check if payment is completed
    pub fn is_paid(&self) -> bool {
        self.status == PaymentStatus::Paid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_creation() {
        let payment = Payment::new(
            "payment-1".to_string(),
            "student-1".to_string(),
            "group-1".to_string(),
            500000.0,
            PaymentMethod::Transfer,
        );

        assert_eq!(payment.id, "payment-1");
        assert_eq!(payment.amount, 500000.0);
        assert_eq!(payment.status, PaymentStatus::Pending);
        assert_eq!(payment.method, PaymentMethod::Transfer);
        assert!(payment.paid_at.is_none());
    }

    #[test]
    fn test_mark_paid() {
        let mut payment = Payment::new(
            "payment-1".to_string(),
            "student-1".to_string(),
            "group-1".to_string(),
            500000.0,
            PaymentMethod::Transfer,
        );

        assert_eq!(payment.status, PaymentStatus::Pending);

        payment.mark_paid();

        assert_eq!(payment.status, PaymentStatus::Paid);
        assert!(payment.paid_at.is_some());
        assert!(payment.is_paid());
    }

    #[test]
    fn test_mark_failed() {
        let mut payment = Payment::new(
            "payment-1".to_string(),
            "student-1".to_string(),
            "group-1".to_string(),
            500000.0,
            PaymentMethod::Card,
        );

        payment.mark_failed();

        assert_eq!(payment.status, PaymentStatus::Failed);
        assert!(!payment.is_paid());
    }

    #[test]
    fn test_refund() {
        let mut payment = Payment::new(
            "payment-1".to_string(),
            "student-1".to_string(),
            "group-1".to_string(),
            500000.0,
            PaymentMethod::Transfer,
        );

        payment.mark_paid();
        payment.refund();

        assert_eq!(payment.status, PaymentStatus::Refunded);
    }

    #[test]
    fn test_payment_status_from_str() {
        assert_eq!(
            PaymentStatus::from_str("pending"),
            Some(PaymentStatus::Pending)
        );
        assert_eq!(PaymentStatus::from_str("PAID"), Some(PaymentStatus::Paid));
        assert_eq!(
            PaymentStatus::from_str("failed"),
            Some(PaymentStatus::Failed)
        );
        assert_eq!(
            PaymentStatus::from_str("refunded"),
            Some(PaymentStatus::Refunded)
        );
        assert_eq!(PaymentStatus::from_str("unknown"), None);
    }

    #[test]
    fn test_payment_method_from_str() {
        assert_eq!(PaymentMethod::from_str("cash"), Some(PaymentMethod::Cash));
        assert_eq!(PaymentMethod::from_str("CARD"), Some(PaymentMethod::Card));
        assert_eq!(
            PaymentMethod::from_str("transfer"),
            Some(PaymentMethod::Transfer)
        );
        assert_eq!(PaymentMethod::from_str("unknown"), None);
    }
}
