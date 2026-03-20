//! In-Memory Payment Repository

use std::collections::HashMap;
use std::sync::RwLock;

use crate::application::ports::PaymentRepository;
use crate::domain::entities::Payment;
use crate::domain::errors::DomainError;

/// In-memory payment repository implementation
pub struct InMemoryPaymentRepository {
    payments: RwLock<HashMap<String, Payment>>,
    student_payments: RwLock<HashMap<String, Vec<String>>>,
    group_payments: RwLock<HashMap<String, Vec<String>>>,
}

impl InMemoryPaymentRepository {
    pub fn new() -> Self {
        Self {
            payments: RwLock::new(HashMap::new()),
            student_payments: RwLock::new(HashMap::new()),
            group_payments: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryPaymentRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl PaymentRepository for InMemoryPaymentRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Payment>, DomainError> {
        let payments = self
            .payments
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        Ok(payments.get(id).cloned())
    }

    fn find_by_student_id(&self, student_id: &str) -> Result<Vec<Payment>, DomainError> {
        let student_payments = self
            .student_payments
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let payments = self
            .payments
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        let mut result = Vec::new();
        if let Some(payment_ids) = student_payments.get(student_id) {
            for id in payment_ids {
                if let Some(payment) = payments.get(id) {
                    result.push(payment.clone());
                }
            }
        }

        Ok(result)
    }

    fn find_by_group_id(&self, group_id: &str) -> Result<Vec<Payment>, DomainError> {
        let group_payments = self
            .group_payments
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let payments = self
            .payments
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        let mut result = Vec::new();
        if let Some(payment_ids) = group_payments.get(group_id) {
            for id in payment_ids {
                if let Some(payment) = payments.get(id) {
                    result.push(payment.clone());
                }
            }
        }

        Ok(result)
    }

    fn save(&self, payment: &Payment) -> Result<(), DomainError> {
        let mut payments = self
            .payments
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let mut student_payments = self
            .student_payments
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        let mut group_payments = self
            .group_payments
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        payments.insert(payment.id.clone(), payment.clone());

        student_payments
            .entry(payment.student_id.clone())
            .or_insert_with(Vec::new)
            .push(payment.id.clone());

        group_payments
            .entry(payment.group_id.clone())
            .or_insert_with(Vec::new)
            .push(payment.id.clone());

        Ok(())
    }

    fn update(&self, payment: &Payment) -> Result<(), DomainError> {
        let mut payments = self
            .payments
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;

        if !payments.contains_key(&payment.id) {
            return Err(DomainError::not_found("Payment", &payment.id));
        }

        payments.insert(payment.id.clone(), payment.clone());

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let mut payments = self
            .payments
            .write()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        payments.remove(id);
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Payment>, DomainError> {
        let payments = self
            .payments
            .read()
            .map_err(|_| DomainError::Validation("Lock error".to_string()))?;
        Ok(payments.values().cloned().collect())
    }
}
