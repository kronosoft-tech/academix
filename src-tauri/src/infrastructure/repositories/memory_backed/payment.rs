//! MemoryBuffer-backed Payment Repository
//!
//! Writes buffer to MemoryBuffer, reads check buffer cache first then fallback to SQLite.

use crate::application::ports::PaymentRepository;
use crate::domain::entities::payment::{Payment, PaymentMethod, PaymentStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::database;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, CachedEntity, MemoryBuffer};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MemoryBackedPaymentRepository {
    buffer: Arc<Mutex<MemoryBuffer>>,
    user_id: String,
}

impl MemoryBackedPaymentRepository {
    pub fn new(buffer: Arc<Mutex<MemoryBuffer>>, user_id: String) -> Self {
        Self { buffer, user_id }
    }

    fn to_cached(payment: &Payment) -> CachedEntity {
        CachedEntity {
            id: payment.id.clone(),
            data: HashMap::from([
                ("id".to_string(), payment.id.clone()),
                ("student_id".to_string(), payment.student_id.clone()),
                ("group_id".to_string(), payment.group_id.clone()),
                ("amount".to_string(), payment.amount.to_string()),
                ("method".to_string(), payment.method.as_str().to_string()),
                ("status".to_string(), payment.status.as_str().to_string()),
                ("due_date".to_string(), payment.due_date.clone().unwrap_or_default()),
                ("paid_at".to_string(), payment.paid_at.map(|dt| dt.to_rfc3339()).unwrap_or_default()),
                ("reference".to_string(), payment.reference.clone().unwrap_or_default()),
                ("description".to_string(), payment.description.clone().unwrap_or_default()),
                ("created_at".to_string(), payment.created_at.to_rfc3339()),
                ("updated_at".to_string(), payment.updated_at.to_rfc3339()),
            ]),
        }
    }

    fn from_cached(cached: &CachedEntity) -> Option<Payment> {
        Some(Payment {
            id: cached.data.get("id")?.clone(),
            student_id: cached.data.get("student_id")?.clone(),
            group_id: cached.data.get("group_id")?.clone(),
            amount: cached.data.get("amount")?.parse().unwrap_or(0.0),
            method: {
                let s = cached.data.get("method")?;
                PaymentMethod::from_str(s).unwrap_or(PaymentMethod::Cash)
            },
            status: {
                let s = cached.data.get("status")?;
                PaymentStatus::from_str(s).unwrap_or(PaymentStatus::Pending)
            },
            due_date: Self::opt_string(cached, "due_date"),
            paid_at: cached.data.get("paid_at")
                .and_then(|s| if s.is_empty() { None } else { DateTime::parse_from_rfc3339(s).ok() })
                .map(|dt| dt.with_timezone(&Utc)),
            reference: Self::opt_string(cached, "reference"),
            description: Self::opt_string(cached, "description"),
            created_at: cached.data.get("created_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
            updated_at: cached.data.get("updated_at")
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
        })
    }

    fn opt_string(cached: &CachedEntity, key: &str) -> Option<String> {
        cached.data.get(key).and_then(|v| {
            if v.is_empty() { None } else { Some(v.clone()) }
        })
    }

    fn row_to_payment(row: &rusqlite::Row<'_>) -> rusqlite::Result<Payment> {
        let due_date_str: Option<String> = row.get(4)?;
        let paid_at_str: Option<String> = row.get(5)?;
        let status_str: String = row.get(6)?;
        let method_str: String = row.get(7)?;
        let reference_str: Option<String> = row.get(8)?;
        let description_str: Option<String> = row.get(9)?;
        let created_str: String = row.get(10)?;
        let updated_str: String = row.get(11)?;

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

impl PaymentRepository for MemoryBackedPaymentRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Payment>, DomainError> {
        let cache_key = format!("payment:{}", id);
        {
            let buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
            if let Some(cached) = buf.get_cached(&self.user_id, &cache_key) {
                if let Some(entity) = Self::from_cached(cached) {
                    return Ok(Some(entity));
                }
            }
        }
        // Fallback to SQLite
        let sql = "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method, 
                           reference, description, created_at, updated_at
                    FROM payments WHERE id = ?";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [id], Self::row_to_payment) {
            Ok(payment) => Ok(Some(payment)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn find_by_student_id(&self, student_id: &str) -> Result<Vec<Payment>, DomainError> {
        // No caching for list queries - go directly to SQLite
        let sql = "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method, 
                           reference, description, created_at, updated_at
                    FROM payments WHERE student_id = ? ORDER BY created_at DESC";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![student_id], Self::row_to_payment)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn find_by_group_id(&self, group_id: &str) -> Result<Vec<Payment>, DomainError> {
        // No caching for list queries - go directly to SQLite
        let sql = "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method, 
                           reference, description, created_at, updated_at
                    FROM payments WHERE group_id = ? ORDER BY created_at DESC";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![group_id], Self::row_to_payment)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn save(&self, payment: &Payment) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        let data = Self::to_cached(payment).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Insert {
            table: "payments".to_string(),
            data,
        });
        Ok(())
    }

    fn update(&self, payment: &Payment) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        let data = Self::to_cached(payment).data;
        buf.buffer_write(&self.user_id, BufferedOperation::Update {
            table: "payments".to_string(),
            id: payment.id.clone(),
            data,
        });
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let mut buf = self.buffer.lock().map_err(|e| DomainError::Database(e.to_string()))?;
        buf.buffer_write(&self.user_id, BufferedOperation::Delete {
            table: "payments".to_string(),
            id: id.to_string(),
        });
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Payment>, DomainError> {
        // No caching for list queries - go directly to SQLite
        let sql = "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method, 
                           reference, description, created_at, updated_at
                    FROM payments ORDER BY created_at DESC";
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map([], Self::row_to_payment)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }
}
