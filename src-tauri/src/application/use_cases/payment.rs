//! Payment Use Cases

use crate::application::dto::{CreatePaymentRequest, PaymentDto, UpdatePaymentRequest};
use crate::application::errors::ApplicationError;
use crate::application::ports::PaymentRepository;
use crate::domain::entities::{Payment, PaymentMethod};
use chrono::Utc;
use uuid::Uuid;

/// Payment service
pub struct PaymentService<R: PaymentRepository> {
    payment_repository: R,
}

impl<R: PaymentRepository> PaymentService<R> {
    pub fn new(payment_repository: R) -> Self {
        Self { payment_repository }
    }

    /// Create a new payment
    pub fn create(&self, request: CreatePaymentRequest) -> Result<PaymentDto, ApplicationError> {
        // Default to "cash" if method is not provided
        let method_str = request.method.unwrap_or_else(|| "cash".to_string());
        let method = PaymentMethod::from_str(&method_str)
            .ok_or_else(|| ApplicationError::Validation("Invalid payment method".to_string()))?;

        let payment = Payment::new(
            Uuid::new_v4().to_string(),
            request.student_id,
            request.group_id,
            request.amount,
            method,
        );

        self.payment_repository.save(&payment)?;

        Ok(PaymentDto {
            id: payment.id,
            student_id: payment.student_id,
            group_id: payment.group_id,
            amount: payment.amount,
            method: payment.method.as_str().to_string(),
            status: payment.status.as_str().to_string(),
            paid_at: payment.paid_at.map(|dt| dt.to_rfc3339()),
            description: payment.description,
        })
    }

    /// Get payment by ID
    pub fn get_by_id(&self, id: &str) -> Result<PaymentDto, ApplicationError> {
        let payment = self
            .payment_repository
            .find_by_id(id)?
            .ok_or_else(|| ApplicationError::NotFound("Payment not found".to_string()))?;

        Ok(PaymentDto {
            id: payment.id,
            student_id: payment.student_id,
            group_id: payment.group_id,
            amount: payment.amount,
            method: payment.method.as_str().to_string(),
            status: payment.status.as_str().to_string(),
            paid_at: payment.paid_at.map(|dt| dt.to_rfc3339()),
            description: payment.description,
        })
    }

    /// List all payments
    pub fn list(&self) -> Result<Vec<PaymentDto>, ApplicationError> {
        let payments = self.payment_repository.find_all()?;

        Ok(payments
            .into_iter()
            .map(|p| PaymentDto {
                id: p.id,
                student_id: p.student_id,
                group_id: p.group_id,
                amount: p.amount,
                method: p.method.as_str().to_string(),
                status: p.status.as_str().to_string(),
                paid_at: p.paid_at.map(|dt| dt.to_rfc3339()),
                description: p.description,
            })
            .collect())
    }

    /// List payments by student
    pub fn list_by_student(&self, student_id: &str) -> Result<Vec<PaymentDto>, ApplicationError> {
        let payments = self.payment_repository.find_by_student_id(student_id)?;

        Ok(payments
            .into_iter()
            .map(|p| PaymentDto {
                id: p.id,
                student_id: p.student_id,
                group_id: p.group_id,
                amount: p.amount,
                method: p.method.as_str().to_string(),
                status: p.status.as_str().to_string(),
                paid_at: p.paid_at.map(|dt| dt.to_rfc3339()),
                description: p.description,
            })
            .collect())
    }

    /// Update payment (e.g., mark as paid)
    pub fn update(
        &self,
        id: &str,
        request: UpdatePaymentRequest,
    ) -> Result<PaymentDto, ApplicationError> {
        let mut payment = self
            .payment_repository
            .find_by_id(id)?
            .ok_or_else(|| ApplicationError::NotFound("Payment not found".to_string()))?;

        if let Some(status) = request.status {
            payment.status =
                crate::domain::entities::PaymentStatus::from_str(&status).unwrap_or(payment.status);
            if status == "paid" {
                payment.paid_at = Some(Utc::now());
            }
        }

        self.payment_repository.update(&payment)?;

        Ok(PaymentDto {
            id: payment.id,
            student_id: payment.student_id,
            group_id: payment.group_id,
            amount: payment.amount,
            method: payment.method.as_str().to_string(),
            status: payment.status.as_str().to_string(),
            paid_at: payment.paid_at.map(|dt| dt.to_rfc3339()),
            description: payment.description,
        })
    }

    /// Delete payment
    pub fn delete(&self, id: &str) -> Result<(), ApplicationError> {
        self.payment_repository.delete(id)?;
        Ok(())
    }
}
