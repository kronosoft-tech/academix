//! Payment SQLite Repository
//!
//! Implements PaymentRepository using SQLite.

use crate::application::ports::PaymentRepository;
use crate::domain::entities::payment::{Payment, PaymentMethod, PaymentStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::database::SqlitePool;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// SQLite implementation of PaymentRepository
pub struct SqlitePaymentRepository {
    pool: Arc<SqlitePool>,
}

impl SqlitePaymentRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    fn row_to_payment(row: &rusqlite::Row<'_>) -> rusqlite::Result<Payment> {
        let status_str: String = row.get(5)?;
        let method_str: String = row.get(4)?;
        let due_date_str: Option<String> = row.get(6)?;
        let paid_at_str: Option<String> = row.get(7)?;
        let reference_str: Option<String> = row.get(8)?;
        let created_str: String = row.get(9)?;
        let updated_str: String = row.get(10)?;

        Ok(Payment {
            id: row.get(0)?,
            student_id: row.get(1)?,
            group_id: row.get(2)?,
            amount: row.get(3)?,
            method: PaymentMethod::from_str(&method_str).unwrap_or(PaymentMethod::Cash),
            status: PaymentStatus::from_str(&status_str).unwrap_or(PaymentStatus::Pending),
            due_date: due_date_str,
            paid_at: paid_at_str
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            reference: reference_str,
            description: row.get(11)?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

impl PaymentRepository for SqlitePaymentRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Payment>, DomainError> {
        let sql = "SELECT id, student_id, group_id, amount, method, status, due_date, paid_date, 
                          description, created_at, updated_at
                   FROM payments WHERE id = ?";

        self.pool
            .query_row(sql, &[&id], Self::row_to_payment)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn find_by_student_id(&self, student_id: &str) -> Result<Vec<Payment>, DomainError> {
        let sql = "SELECT id, student_id, group_id, amount, method, status, due_date, paid_date, 
                          description, created_at, updated_at
                   FROM payments WHERE student_id = ? ORDER BY created_at DESC";

        self.pool
            .query(sql, &[&student_id], Self::row_to_payment)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn find_by_group_id(&self, group_id: &str) -> Result<Vec<Payment>, DomainError> {
        let sql = "SELECT id, student_id, group_id, amount, method, status, due_date, paid_date, 
                          description, created_at, updated_at
                   FROM payments WHERE group_id = ? ORDER BY created_at DESC";

        self.pool
            .query(sql, &[&group_id], Self::row_to_payment)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn save(&self, payment: &Payment) -> Result<(), DomainError> {
        let sql = "INSERT INTO payments (id, student_id, group_id, amount, method, status, 
                                     due_date, paid_date, description, created_at, updated_at)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        let paid_date = payment.paid_at.map(|dt| dt.to_rfc3339());

        self.pool
            .execute(
                sql,
                &[
                    &payment.id,
                    &payment.student_id,
                    &payment.group_id,
                    &payment.amount.to_string(),
                    &payment.method.as_str().to_string(),
                    &payment.status.as_str().to_string(),
                    &payment.due_date,
                    &paid_date,
                    &payment.description,
                    &payment.created_at.to_rfc3339(),
                    &payment.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    fn update(&self, payment: &Payment) -> Result<(), DomainError> {
        let sql = "UPDATE payments 
                   SET student_id = ?, group_id = ?, amount = ?, method = ?, status = ?, 
                       due_date = ?, paid_date = ?, description = ?, updated_at = ?
                   WHERE id = ?";

        let paid_date = payment.paid_at.map(|dt| dt.to_rfc3339());

        let affected = self
            .pool
            .execute(
                sql,
                &[
                    &payment.student_id,
                    &payment.group_id,
                    &payment.amount.to_string(),
                    &payment.method.as_str().to_string(),
                    &payment.status.as_str().to_string(),
                    &payment.due_date,
                    &paid_date,
                    &payment.description,
                    &Utc::now().to_rfc3339(),
                    &payment.id,
                ],
            )
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Payment", &payment.id));
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "DELETE FROM payments WHERE id = ?";

        let affected = self
            .pool
            .execute(sql, &[&id])
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Payment", id));
        }
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Payment>, DomainError> {
        let sql = "SELECT id, student_id, group_id, amount, method, status, due_date, paid_date, 
                          description, created_at, updated_at
                   FROM payments ORDER BY created_at DESC";

        self.pool
            .query(sql, &[], Self::row_to_payment)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }
}
