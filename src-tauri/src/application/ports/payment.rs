//! Payment Repository Port

use crate::domain::entities::Payment;
use crate::domain::errors::DomainError;

/// Payment repository port - defines operations for payment persistence
pub trait PaymentRepository: Send + Sync {
    /// Find payment by ID
    fn find_by_id(&self, id: &str) -> Result<Option<Payment>, DomainError>;

    /// Find payments by student ID
    fn find_by_student_id(&self, student_id: &str) -> Result<Vec<Payment>, DomainError>;

    /// Find payments by group ID
    fn find_by_group_id(&self, group_id: &str) -> Result<Vec<Payment>, DomainError>;

    /// Save a new payment
    fn save(&self, payment: &Payment) -> Result<(), DomainError>;

    /// Update an existing payment
    fn update(&self, payment: &Payment) -> Result<(), DomainError>;

    /// Delete payment
    fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// List all payments
    fn find_all(&self) -> Result<Vec<Payment>, DomainError>;
}
