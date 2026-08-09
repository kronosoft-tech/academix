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
use crate::domain::entities::payment::{Payment, PaymentMethod, PaymentStatus, PaymentType};
use crate::domain::errors::DomainError;
use crate::infrastructure::turso::connection_manager::ConnectionManager;
use crate::infrastructure::turso::memory_buffer::{BufferedOperation, MemoryBuffer};
use crate::infrastructure::turso::session::CurrentSession;
use chrono::{DateTime, Utc};

/// MemoryBuffer-backed implementation of PaymentRepository.
#[derive(Clone)]
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

    /// Convert a libsql::Row into a HashMap<String, String> for cache storage.
    /// Column indices must match the SELECT statement order used in queries.
    fn row_to_hash_map(row: &libsql::Row) -> Result<HashMap<String, String>, DomainError> {
        let mut map = HashMap::new();
        map.insert(
            "id".to_string(),
            row.get::<String>(0)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "student_id".to_string(),
            row.get::<String>(1)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "group_id".to_string(),
            row.get::<String>(2)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "amount".to_string(),
            row.get::<f64>(3)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .to_string(),
        );
        map.insert(
            "due_date".to_string(),
            row.get::<Option<String>>(4)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .unwrap_or_default(),
        );
        map.insert(
            "paid_date".to_string(),
            row.get::<Option<String>>(5)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .unwrap_or_default(),
        );
        map.insert(
            "status".to_string(),
            row.get::<String>(6)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "method".to_string(),
            row.get::<String>(7)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "reference".to_string(),
            row.get::<Option<String>>(8)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .unwrap_or_default(),
        );
        map.insert(
            "description".to_string(),
            row.get::<Option<String>>(9)
                .map_err(|e| DomainError::Database(e.to_string()))?
                .unwrap_or_default(),
        );
        map.insert(
            "payment_type".to_string(),
            row.get::<String>(10)
                .unwrap_or_else(|_| "tuition".to_string()),
        );
        map.insert(
            "created_at".to_string(),
            row.get::<String>(11)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        map.insert(
            "updated_at".to_string(),
            row.get::<String>(12)
                .map_err(|e| DomainError::Database(e.to_string()))?,
        );
        Ok(map)
    }

    fn payment_from_data(data: &HashMap<String, String>) -> Result<Payment, DomainError> {
        let status_str = data
            .get("status")
            .ok_or_else(|| DomainError::Database("missing status".into()))?;
        let method_str = data
            .get("method")
            .ok_or_else(|| DomainError::Database("missing method".into()))?;
        let created_str = data
            .get("created_at")
            .ok_or_else(|| DomainError::Database("missing created_at".into()))?;
        let updated_str = data
            .get("updated_at")
            .ok_or_else(|| DomainError::Database("missing updated_at".into()))?;

        let paid_at_str: Option<&String> = data.get("paid_date");
        let paid_at = paid_at_str
            .filter(|s| !s.is_empty())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(Payment {
            id: data
                .get("id")
                .ok_or_else(|| DomainError::Database("missing id".into()))?
                .clone(),
            student_id: data
                .get("student_id")
                .ok_or_else(|| DomainError::Database("missing student_id".into()))?
                .clone(),
            group_id: data
                .get("group_id")
                .ok_or_else(|| DomainError::Database("missing group_id".into()))?
                .clone(),
            amount: data
                .get("amount")
                .ok_or_else(|| DomainError::Database("missing amount".into()))?
                .parse()
                .unwrap_or(0.0),
            method: PaymentMethod::from_str(method_str).unwrap_or(PaymentMethod::Cash),
            payment_type: PaymentType::from_str(
                data.get("payment_type")
                    .map(|s| s.as_str())
                    .unwrap_or("tuition"),
            )
            .unwrap_or(PaymentType::Tuition),
            status: PaymentStatus::from_str(status_str).unwrap_or(PaymentStatus::Pending),
            due_date: data.get("due_date").and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.clone())
                }
            }),
            paid_at,
            reference: data.get("reference").and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.clone())
                }
            }),
            description: data.get("description").and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.clone())
                }
            }),
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
        data.insert(
            "due_date".to_string(),
            payment.due_date.clone().unwrap_or_default(),
        );
        data.insert(
            "paid_date".to_string(),
            payment.paid_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
        );
        data.insert("status".to_string(), payment.status.as_str().to_string());
        data.insert("method".to_string(), payment.method.as_str().to_string());
        data.insert(
            "payment_type".to_string(),
            payment.payment_type.as_str().to_string(),
        );
        data.insert(
            "reference".to_string(),
            payment.reference.clone().unwrap_or_default(),
        );
        data.insert(
            "description".to_string(),
            payment.description.clone().unwrap_or_default(),
        );
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

    async fn query_turso(
        &self,
        user_id: &str,
        sql: &str,
        params: impl libsql::params::IntoParams,
    ) -> Result<libsql::Rows, DomainError> {
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

        // Check entity cache or query Turso
        let row_data: Option<HashMap<String, String>> = {
            let buf = self.memory_buffer.lock().await;
            if let Some(cached) = buf.get_cached_entity(&user_id, "payments", id) {
                cached.clone()
            } else {
                drop(buf); // Release lock before network call

                let sql =
                    "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method,
                                  reference, description, payment_type, created_at, updated_at
                           FROM payments WHERE id = ?1";
                let mut rows = self.query_turso(&user_id, sql, libsql::params![id]).await?;
                let data = match rows
                    .next()
                    .await
                    .map_err(|e| DomainError::Database(e.to_string()))?
                {
                    Some(row) => Some(Self::row_to_hash_map(&row)?),
                    None => None,
                };

                // Store in cache
                let mut buf = self.memory_buffer.lock().await;
                buf.set_cached_entity(&user_id, "payments", id, data.clone());
                data
            }
        };

        match row_data {
            Some(data) => Ok(Some(Self::payment_from_data(&data)?)),
            None => Ok(None),
        }
    }

    async fn find_by_student_id(&self, student_id: &str) -> Result<Vec<Payment>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Step 1: Check cache or query Turso for ALL payments, then filter
        let base_rows: Vec<HashMap<String, String>> = {
            let buf = self.memory_buffer.lock().await;
            if let Some(cached) = buf.get_cached_list(&user_id, "payments") {
                cached.clone()
            } else {
                drop(buf); // Release lock before network call

                let sql =
                    "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method,
                                  reference, description, payment_type, created_at, updated_at
                           FROM payments ORDER BY created_at DESC";
                let mut rows = self.query_turso(&user_id, sql, libsql::params![]).await?;

                let mut raw_rows: Vec<HashMap<String, String>> = Vec::new();
                while let Some(row) = rows
                    .next()
                    .await
                    .map_err(|e| DomainError::Database(e.to_string()))?
                {
                    raw_rows.push(Self::row_to_hash_map(&row)?);
                }

                // Store in cache
                let mut buf = self.memory_buffer.lock().await;
                buf.set_cached_list(&user_id, "payments", raw_rows.clone());
                raw_rows
            }
        };

        // Step 2: Filter by student_id and convert to domain entities
        let mut results: Vec<Payment> = base_rows
            .iter()
            .filter(|data| data.get("student_id").map(|v| v.as_str()) == Some(student_id))
            .map(|data| Self::payment_from_data(data))
            .collect::<Result<Vec<_>, _>>()?;

        // Step 3: Merge with pending operations
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
            if let BufferedOperation::Update {
                id: update_id,
                data,
                ..
            } = op
            {
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

        // Step 1: Check cache or query Turso for ALL payments, then filter
        let base_rows: Vec<HashMap<String, String>> = {
            let buf = self.memory_buffer.lock().await;
            if let Some(cached) = buf.get_cached_list(&user_id, "payments") {
                cached.clone()
            } else {
                drop(buf); // Release lock before network call

                let sql =
                    "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method,
                                  reference, description, payment_type, created_at, updated_at
                           FROM payments ORDER BY created_at DESC";
                let mut rows = self.query_turso(&user_id, sql, libsql::params![]).await?;

                let mut raw_rows: Vec<HashMap<String, String>> = Vec::new();
                while let Some(row) = rows
                    .next()
                    .await
                    .map_err(|e| DomainError::Database(e.to_string()))?
                {
                    raw_rows.push(Self::row_to_hash_map(&row)?);
                }

                // Store in cache
                let mut buf = self.memory_buffer.lock().await;
                buf.set_cached_list(&user_id, "payments", raw_rows.clone());
                raw_rows
            }
        };

        // Step 2: Filter by group_id and convert to domain entities
        let mut results: Vec<Payment> = base_rows
            .iter()
            .filter(|data| data.get("group_id").map(|v| v.as_str()) == Some(group_id))
            .map(|data| Self::payment_from_data(data))
            .collect::<Result<Vec<_>, _>>()?;

        // Step 3: Merge with pending operations
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
            if let BufferedOperation::Update {
                id: update_id,
                data,
                ..
            } = op
            {
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
        buf.buffer_write(
            &user_id,
            BufferedOperation::Insert {
                table: "payments".to_string(),
                data,
            },
        );
        Ok(())
    }

    async fn update(&self, payment: &Payment) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        let mut data = HashMap::new();
        data.insert("student_id".to_string(), payment.student_id.clone());
        data.insert("group_id".to_string(), payment.group_id.clone());
        data.insert("amount".to_string(), payment.amount.to_string());
        data.insert(
            "due_date".to_string(),
            payment.due_date.clone().unwrap_or_default(),
        );
        data.insert(
            "paid_date".to_string(),
            payment.paid_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
        );
        data.insert("status".to_string(), payment.status.as_str().to_string());
        data.insert("method".to_string(), payment.method.as_str().to_string());
        data.insert(
            "payment_type".to_string(),
            payment.payment_type.as_str().to_string(),
        );
        data.insert(
            "reference".to_string(),
            payment.reference.clone().unwrap_or_default(),
        );
        data.insert(
            "description".to_string(),
            payment.description.clone().unwrap_or_default(),
        );
        data.insert("created_at".to_string(), payment.created_at.to_rfc3339());
        data.insert("updated_at".to_string(), payment.updated_at.to_rfc3339());
        data.insert("id".to_string(), payment.id.clone()); // for deserialization

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Update {
                table: "payments".to_string(),
                id: payment.id.clone(),
                data,
            },
        );
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let user_id = self.get_user_id().await?;

        let mut buf = self.memory_buffer.lock().await;
        buf.buffer_write(
            &user_id,
            BufferedOperation::Delete {
                table: "payments".to_string(),
                id: id.to_string(),
            },
        );
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Payment>, DomainError> {
        let user_id = self.get_user_id().await?;

        // Step 1: Check cache or query Turso
        let base_rows: Vec<HashMap<String, String>> = {
            let buf = self.memory_buffer.lock().await;
            if let Some(cached) = buf.get_cached_list(&user_id, "payments") {
                cached.clone()
            } else {
                drop(buf); // Release lock before network call

                let sql =
                    "SELECT id, student_id, group_id, amount, due_date, paid_date, status, method,
                                  reference, description, payment_type, created_at, updated_at
                           FROM payments ORDER BY created_at DESC";
                let mut rows = self.query_turso(&user_id, sql, libsql::params![]).await?;

                let mut raw_rows: Vec<HashMap<String, String>> = Vec::new();
                while let Some(row) = rows
                    .next()
                    .await
                    .map_err(|e| DomainError::Database(e.to_string()))?
                {
                    raw_rows.push(Self::row_to_hash_map(&row)?);
                }

                // Store in cache
                let mut buf = self.memory_buffer.lock().await;
                buf.set_cached_list(&user_id, "payments", raw_rows.clone());
                raw_rows
            }
        };

        // Step 2: Convert rows to domain entities
        let mut results: Vec<Payment> = base_rows
            .iter()
            .map(|data| Self::payment_from_data(data))
            .collect::<Result<Vec<_>, _>>()?;

        // Step 3: Merge with pending operations
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
            if let BufferedOperation::Update {
                id: update_id,
                data,
                ..
            } = op
            {
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
