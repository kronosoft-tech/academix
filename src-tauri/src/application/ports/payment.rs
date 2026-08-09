//! Payment Repository Port

use async_trait::async_trait;
use crate::domain::entities::Payment;
use crate::domain::errors::DomainError;

/// Payment repository port - defines operations for payment persistence
#[async_trait]
pub trait PaymentRepository: Send + Sync {
    /// Find payment by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<Payment>, DomainError>;

    /// Find payments by student ID
    async fn find_by_student_id(&self, student_id: &str) -> Result<Vec<Payment>, DomainError>;

    /// Find payments by group ID
    async fn find_by_group_id(&self, group_id: &str) -> Result<Vec<Payment>, DomainError>;

    /// Save a new payment
    async fn save(&self, payment: &Payment) -> Result<(), DomainError>;

    /// Update an existing payment
    async fn update(&self, payment: &Payment) -> Result<(), DomainError>;

    /// Delete payment
    async fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// List all payments
    async fn find_all(&self) -> Result<Vec<Payment>, DomainError>;
}
