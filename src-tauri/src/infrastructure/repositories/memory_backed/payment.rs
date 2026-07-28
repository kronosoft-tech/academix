//! MemoryBacked Payment Repository
//!
//! Implements PaymentRepository using a MemoryBuffer write-back cache
//! backed by the user's Turso database via ConnectionManager.
//! Phase 5b: 6 MemoryBacked repositories.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::application::ports::PaymentRepository;
use crate::domain::entities::payment::{Payment, PaymentMethod, PaymentStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use crate::infrastructure::turso::session::CurrentSession;
use chrono::{DateTime, Utc};

/// MemoryBuffer-backed implementation of PaymentRepository.
pub struct MemoryBackedPaymentRepository {
    connection_manager: Arc<Mutex<ConnectionManager>>,
    memory_buffer: Arc<Mutex<MemoryBuffer>>,
    session: Arc<Mutex<CurrentSession>>,
}

impl MemoryBackedPaymentRepository {
    pub fn new(
        connection_manager: Arc<Mutex<ConnectionManager>>,
        memory_buffer: Arc<Mutex<MemoryBuffer>>,
        session: Arc<Mutex<CurrentSession>>,
    ) -> Self {
        Self {
            connection_manager,
            memory_buffer,
            session,
        }
    }

    fn row_to_payment(row: &libsql::Row) -> Result<Payment, DomainError> {
        let due_date_str: Option<String> = row.get(4).map_err(|e| DomainError::Database(e.to_string()))?;
        let paid_at_str: Option<String> = row.get(5).map_err(|e| DomainError::Database(e.to_string()))?;
        let status_str: String = row.get(6).map_err(|e| DomainError::Database(e.to_string()))?;
        let method_str: String = row.get(7).map_err(|e| DomainError::Database(e.to_string()))?;
        let reference_str: Option<String> = row.get(8).map_err(|e| DomainError::Database(e.to_string()))?;
        let description_str: Option<String> = row.get(9).map_err(|e| DomainError::Database(e.to_string()))?;
        let created_str: String = row.get(10).map_err(|e| DomainError::Database(e.to_string()))?;
        let updated_str: String = row.get(11).map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(Payment {
            id: row.get(0).map_err(|e| DomainError::Database(e.to_string()))?,
            student_id: row.get(1).map_err(|e| DomainError::Database(e.to_string()))?,
            group_id: row.get(2).map_err(|e| DomainError::Database(e.to_string()))?,
            amount: row.get(3).map_err(|e| DomainError::Database(e.to_string()))?,
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

    fn payment_from_data(data: &HashMap<String, String>) -> Result<Payment, DomainError> {
        let status_str = data.get("status").ok_or_else(|| DomainError::Database("missing status".into()))?;
        let method_str = data.get("method").ok_or_else(|| DomainError::Database("missing method".into()))?;
        let created_str = data.get("created_at").ok_or_else(|| DomainError::Database("missing created_at".into()))?;
        let updated_str = data.get("updated_at").ok_or_else(|| DomainError::Database("missing updated_at".into()))?;

        let paid_at_str: Option<&String> = data.get("paid_at");
        let paid_at = paid_at_str
            .filter(|s| !s.is_empty())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(Payment {
            id: data.get("id").ok_or_else(|| DomainError::Database("missing id".into()))?.clone(),
            student_id: data.get("student_id").ok_or_else(|| DomainError::Database("missing student_id".into()))?.clone(),
            group_id: data.get("group_id").ok_or_else(|| DomainError::Database("missing group_id".into()))?.clone(),
            amount: data.get("amount").ok_or_else(|| DomainError::Database("missing amount".into()))?.parse().unwrap_or(0.0),
            method: PaymentMethod::from_str(method_str).unwrap_or(PaymentMethod::Cash),
            status: PaymentStatus::from_str(status_str).unwrap_or(PaymentStatus::Pending),
            due_date: data.get("due_date").and_then(|v| if v.is_empty() { None } else { Some(v.clone()) }),
            paid_at,
            reference: data.get("reference").and_then(|v| if v.is_empty() { None } else { Some(v.clone()) }),
            description: data.get("description").and_then(|v| if v.is_empty() { None } else { Some(v.clone()) }),
            created_at: DateTime::parse_from_rfc3339(created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    fn to_data_map(payment: &Payment) -> HashMap<String, String> {
        let mut data = HashMap::new();
        data.insert("id".to_string(), payment.id.clone());
        data.insert("student_id".to_string(), payment.student_id.clone());
        data.insert("group_id".to_string(), payment.group_id.clone());
        data.insert("amount".to_string(), payment.amount.to_string());
        data.insert("due_date".to_string(), payment.due_date.clone().unwrap_or_default());
        data.insert("paid_at".to_string(), payment.paid_at.map(|d| d.to_rfc3339()).unwrap_or_default());
        data.insert("status".to_string(), payment.status.as_str().to_string());
        data.insert("method".to_string(), payment.method.as_str().to_string());
        data.insert("reference".to_string(), payment.reference.clone().unwrap_or_default());
        data.insert("description".to_string(), payment.description.clone().unwrap_or_default());
        data.insert("created_at".to_string(), payment.created_at.to_rfc3339());
        data.insert("updated_at".to_string(), payment.updated_at.to_rfc3339());
        data
    }

    async fn get_user_id(&self) -> Result<String, DomainError> {
        let session = self.session.lock().await;
        session
            .user_id
            .clone()
            .ok_or_else(|| DomainError::Authentication("Not authenticated".to_string()))
    }

    async fn query_turso(&self, user_id: &str, sql: &str, params: impl libsql::params::IntoParams) -> Result<libsql::Rows, DomainError> {
        let db = {
            let cm = self.connection_manager.lock().await;
            cm.get_connection(user_id)
                .map(|c| c.db.clone())
                .ok_or_else(|| DomainError::Database("No connection for user".to_string()))?
        };
        let conn = db
            .connect()
            .map_err(|e| DomainError::Database(e.to_string()))?;
        conn.query(sql, params)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))
    }
}

#[async_trait]
impl PaymentRepository for MemoryBackedPaymentRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Payment>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Check pending inserts/updates first
        {
            let buf = self.memory_buffer.lock().await;
            if let Some(op) = buf.find_pending_insert(&user_id, "payments", id) {
                if let BufferedOperation::Insert { data, .. } = op {
                    return Ok(Some(Self::payment_from_data(data)?));
                }
            }
            if let Some(op) = buf.find_pending_update(&user_id, "payments", id) {
                if let BufferedOperation::Update { data, .. } = op {
                    return Ok(Some(Self::payment_from_data(data)?));
                }
            }
            if buf.has_pending_delete(&user_id, "payments", id) {
                return Ok(None);
            }
        }

        // Read from Turso
        let sql = "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method,
                          reference, description, created_at, updated_at
                   FROM payments WHERE id = ?1";
        let mut rows = self.query_turso(&user_id, sql, libsql::params![id]).await?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => Ok(Some(Self::row_to_payment(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_student_id(&self, student_id: &str) -> Result<Vec<Payment>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Read from Turso
        let sql = "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method,
                          reference, description, created_at, updated_at
                   FROM payments WHERE student_id = ?1 ORDER BY created_at DESC";
        let mut rows = self.query_turso(&user_id, sql, libsql::params![student_id]).await?;

        let mut results: Vec<Payment> = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_payment(&row)?);
        }

        // Merge with pending operations
        let buf = self.memory_buffer.lock().await;

        let pending_inserts = buf.scan_pending_inserts(&user_id, "payments");
        for op in pending_inserts {
            if let BufferedOperation::Insert { data, .. } = op {
                if data.get("student_id").map(|v| v.as_str()) == Some(student_id) {
                    results.push(Self::payment_from_data(data)?);
                }
            }
        }

        // Re-sort by created_at DESC
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let pending_updates = buf.scan_pending_updates(&user_id, "payments");
        for op in &pending_updates {
            if let BufferedOperation::Update { id: update_id, data, .. } = op {
                if let Some(pos) = results.iter().position(|p| p.id == *update_id) {
                    results[pos] = Self::payment_from_data(data)?;
                }
            }
        }

        let pending_deletes = buf.scan_pending_deletes(&user_id, "payments");
        for op in &pending_deletes {
            if let BufferedOperation::Delete { id: del_id, .. } = op {
                results.retain(|p| p.id != *del_id);
            }
        }

        Ok(results)
    }

    async fn find_by_group_id(&self, group_id: &str) -> Result<Vec<Payment>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Read from Turso
        let sql = "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method,
                          reference, description, created_at, updated_at
                   FROM payments WHERE group_id = ?1 ORDER BY created_at DESC";
        let mut rows = self.query_turso(&user_id, sql, libsql::params![group_id]).await?;

        let mut results: Vec<Payment> = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_payment(&row)?);
        }

        // Merge with pending operations
        let buf = self.memory_buffer.lock().await;

        let pending_inserts = buf.scan_pending_inserts(&user_id, "payments");
        for op in pending_inserts {
            if let BufferedOperation::Insert { data, .. } = op {
                if data.get("group_id").map(|v| v.as_str()) == Some(group_id) {
                    results.push(Self::payment_from_data(data)?);
                }
            }
        }

        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let pending_updates = buf.scan_pending_updates(&user_id, "payments");
        for op in &pending_updates {
            if let BufferedOperation::Update { id: update_id, data, .. } = op {
                if let Some(pos) = results.iter().position(|p| p.id == *update_id) {
                    results[pos] = Self::payment_from_data(data)?;
                }
            }
        }

        let pending_deletes = buf.scan_pending_deletes(&user_id, "payments");
        for op in &pending_deletes {
            if let BufferedOperation::Delete { id: del_id, .. } = op {
                results.retain(|p| p.id != *del_id);
            }
        }

        Ok(results)
    }

    async fn save(&self, payment: &Payment) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;
        let data = Self::to_data_map(payment);
        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(&user_id, BufferedOperation::Insert {
            table: "payments".to_string(),
            data,
        });
        Ok(())
    }

    async fn update(&self, payment: &Payment) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        let mut data = HashMap::new();
        data.insert("student_id".to_string(), payment.student_id.clone());
        data.insert("group_id".to_string(), payment.group_id.clone());
        data.insert("amount".to_string(), payment.amount.to_string());
        data.insert("due_date".to_string(), payment.due_date.clone().unwrap_or_default());
        data.insert("paid_at".to_string(), payment.paid_at.map(|d| d.to_rfc3339()).unwrap_or_default());
        data.insert("status".to_string(), payment.status.as_str().to_string());
        data.insert("method".to_string(), payment.method.as_str().to_string());
        data.insert("reference".to_string(), payment.reference.clone().unwrap_or_default());
        data.insert("description".to_string(), payment.description.clone().unwrap_or_default());
        data.insert("created_at".to_string(), payment.created_at.to_rfc3339());
        data.insert("updated_at".to_string(), payment.updated_at.to_rfc3339());
        data.insert("id".to_string(), payment.id.clone()); // for deserialization

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(&user_id, BufferedOperation::Update {
            table: "payments".to_string(),
            id: payment.id.clone(),
            data,
        });
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(&user_id, BufferedOperation::Delete {
            table: "payments".to_string(),
            id: id.to_string(),
        });
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Payment>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Read from Turso
        let sql = "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method,
                          reference, description, created_at, updated_at
                   FROM payments ORDER BY created_at DESC";
        let mut rows = self.query_turso(&user_id, sql, libsql::params![]).await?;

        let mut results: Vec<Payment> = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_payment(&row)?);
        }

        // Merge with pending operations
        let buf = self.memory_buffer.lock().await;

        let pending_inserts = buf.scan_pending_inserts(&user_id, "payments");
        for op in pending_inserts {
            if let BufferedOperation::Insert { data, .. } = op {
                results.push(Self::payment_from_data(data)?);
            }
        }

        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let pending_updates = buf.scan_pending_updates(&user_id, "payments");
        for op in &pending_updates {
            if let BufferedOperation::Update { id: update_id, data, .. } = op {
                if let Some(pos) = results.iter().position(|p| p.id == *update_id) {
                    results[pos] = Self::payment_from_data(data)?;
                }
            }
        }

        let pending_deletes = buf.scan_pending_deletes(&user_id, "payments");
        for op in &pending_deletes {
            if let BufferedOperation::Delete { id: del_id, .. } = op {
                results.retain(|p| p.id != *del_id);
            }
        }

        Ok(results)
    }
}
