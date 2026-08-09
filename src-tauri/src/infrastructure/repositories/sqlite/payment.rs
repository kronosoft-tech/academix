use crate::application::ports::PaymentRepository;
use crate::domain::entities::payment::{Payment, PaymentMethod, PaymentStatus, PaymentType};
use crate::domain::errors::DomainError;
use crate::infrastructure::local_db;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct SqlitePaymentRepository;

impl SqlitePaymentRepository {
    pub fn new() -> Self {
        Self
    }

    fn row_to_payment(row: &libsql::Row) -> Result<Payment, DomainError> {
        let due_date_str: Option<String> = row
            .get(4)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let paid_at_str: Option<String> = row
            .get(5)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let status_str: String = row
            .get(6)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let method_str: String = row
            .get(7)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let reference_str: Option<String> = row
            .get(8)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let description_str: Option<String> = row
            .get(9)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let payment_type_str: String = row.get(10).unwrap_or_else(|_| "tuition".to_string());
        let created_str: String = row
            .get(11)
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let updated_str: String = row
            .get(12)
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(Payment {
            id: row
                .get(0)
                .map_err(|e| DomainError::Database(e.to_string()))?,
            student_id: row
                .get(1)
                .map_err(|e| DomainError::Database(e.to_string()))?,
            group_id: row
                .get(2)
                .map_err(|e| DomainError::Database(e.to_string()))?,
            amount: row
                .get(3)
                .map_err(|e| DomainError::Database(e.to_string()))?,
            method: PaymentMethod::from_str(&method_str).unwrap_or(PaymentMethod::Cash),
            payment_type: PaymentType::from_str(&payment_type_str).unwrap_or(PaymentType::Tuition),
            status: PaymentStatus::from_str(&status_str).unwrap_or(PaymentStatus::Pending),
            due_date: due_date_str,
            paid_at: paid_at_str
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            reference: reference_str,
            description: description_str,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

impl Default for SqlitePaymentRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PaymentRepository for SqlitePaymentRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Payment>, DomainError> {
        let sql = "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method, 
                           reference, description, payment_type, created_at, updated_at
                    FROM payments WHERE id = ?1";

        let conn = local_db::get_db()
            .connect()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn
            .query(sql, libsql::params![id])
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        match rows
            .next()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?
        {
            Some(row) => {
                let payment = Self::row_to_payment(&row)?;
                Ok(Some(payment))
            }
            None => Ok(None),
        }
    }

    async fn find_by_student_id(&self, student_id: &str) -> Result<Vec<Payment>, DomainError> {
        let sql = "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method, 
                           reference, description, payment_type, created_at, updated_at
                    FROM payments WHERE student_id = ?1 ORDER BY created_at DESC";

        let conn = local_db::get_db()
            .connect()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn
            .query(sql, libsql::params![student_id])
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?
        {
            results.push(Self::row_to_payment(&row)?);
        }
        Ok(results)
    }

    async fn find_by_group_id(&self, group_id: &str) -> Result<Vec<Payment>, DomainError> {
        let sql = "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method, 
                           reference, description, payment_type, created_at, updated_at
                    FROM payments WHERE group_id = ?1 ORDER BY created_at DESC";

        let conn = local_db::get_db()
            .connect()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn
            .query(sql, libsql::params![group_id])
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?
        {
            results.push(Self::row_to_payment(&row)?);
        }
        Ok(results)
    }

    async fn save(&self, payment: &Payment) -> Result<(), DomainError> {
        let sql = "INSERT INTO payments (id, student_id, group_id, amount, due_date, paid_date, status, method, 
                           reference, description, payment_type, created_at, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)";

        let paid_date = payment.paid_at.map(|dt| dt.to_rfc3339());
        let due_date = payment.due_date.clone().unwrap_or_else(|| {
            chrono::Utc::now()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string()
        });

        let amount_str = payment.amount.to_string();
        let status_str = payment.status.as_str().to_string();
        let method_str = payment.method.as_str().to_string();
        let payment_type_str = payment.payment_type.as_str().to_string();
        let created_str = payment.created_at.to_rfc3339();
        let updated_str = payment.updated_at.to_rfc3339();

        eprintln!("[DEBUG INSERT] about to execute with 13 params");

        let conn = local_db::get_db()
            .connect()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let result = conn
            .execute(
                sql,
                libsql::params![
                    payment.id.clone(),
                    payment.student_id.clone(),
                    payment.group_id.clone(),
                    amount_str,
                    due_date,
                    paid_date,
                    status_str,
                    method_str,
                    payment.reference.clone(),
                    payment.description.clone(),
                    payment_type_str,
                    created_str,
                    updated_str,
                ],
            )
            .await;

        match result {
            Ok(affected) => {
                eprintln!("[DEBUG INSERT] Success! Affected rows: {}", affected);
            }
            Err(e) => {
                eprintln!("[DEBUG INSERT] ERROR: {}", e);
                return Err(DomainError::Validation(e.to_string()));
            }
        }

        // Verify the insert actually worked
        let verify_sql = "SELECT COUNT(*) FROM payments";
        let conn2 = local_db::get_db()
            .connect()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        if let Ok(mut count_rows) = conn2.query(verify_sql, libsql::params![]).await {
            if let Ok(Some(count_row)) = count_rows.next().await {
                if let Ok(count) = count_row.get::<i32>(0) {
                    eprintln!("[DEBUG INSERT] Total payments in DB: {}", count);
                }
            }
        } else {
            eprintln!("[ERROR] Could not verify payment count");
        }

        Ok(())
    }

    async fn update(&self, payment: &Payment) -> Result<(), DomainError> {
        let sql = "UPDATE payments 
                    SET student_id = ?1, group_id = ?2, amount = ?3, due_date = ?4, paid_date = ?5, 
                        status = ?6, method = ?7, reference = ?8, description = ?9, 
                        payment_type = ?10, created_at = ?11, updated_at = ?12
                    WHERE id = ?13";

        let paid_date = payment.paid_at.map(|dt| dt.to_rfc3339());
        let conn = local_db::get_db()
            .connect()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(
                sql,
                libsql::params![
                    payment.student_id.clone(),
                    payment.group_id.clone(),
                    payment.amount.to_string(),
                    payment.due_date.clone(),
                    paid_date,
                    payment.status.as_str().to_string(),
                    payment.method.as_str().to_string(),
                    payment.reference.clone(),
                    payment.description.clone(),
                    payment.payment_type.as_str().to_string(),
                    payment.created_at.to_rfc3339(),
                    payment.updated_at.to_rfc3339(),
                    payment.id.clone(),
                ],
            )
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Payment", &payment.id));
        }
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "DELETE FROM payments WHERE id = ?1";

        let conn = local_db::get_db()
            .connect()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(sql, libsql::params![id])
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Payment", id));
        }
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Payment>, DomainError> {
        let sql = "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method, 
                           reference, description, payment_type, created_at, updated_at
                    FROM payments ORDER BY created_at DESC";

        let conn = local_db::get_db()
            .connect()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn
            .query(sql, libsql::params![])
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?
        {
            results.push(Self::row_to_payment(&row)?);
        }
        Ok(results)
    }
}
