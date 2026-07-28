//! Student SQLite Repository
//!
//! Implements StudentRepository using SQLite.

use async_trait::async_trait;
use crate::application::ports::StudentRepository;
use crate::domain::entities::Student;
use crate::domain::errors::DomainError;
use crate::infrastructure::local_db;
use chrono::{DateTime, Utc};

/// SQLite implementation of StudentRepository
#[derive(Clone)]
pub struct SqliteStudentRepository;

impl SqliteStudentRepository {
    pub fn new() -> Self {
        Self
    }

    fn row_to_student(row: &libsql::Row) -> Result<Student, DomainError> {
        let birth_date_str: Option<String> = row.get(8).map_err(|e| DomainError::Database(e.to_string()))?;
        let guardian_name: Option<String> = row.get(9).map_err(|e| DomainError::Database(e.to_string()))?;
        let guardian_document: Option<String> = row.get(10).map_err(|e| DomainError::Database(e.to_string()))?;
        let guardian_phone: Option<String> = row.get(11).map_err(|e| DomainError::Database(e.to_string()))?;
        let active: i32 = row.get(12).map_err(|e| DomainError::Database(e.to_string()))?;
        let created_str: String = row.get(13).map_err(|e| DomainError::Database(e.to_string()))?;
        let updated_str: String = row.get(14).map_err(|e| DomainError::Database(e.to_string()))?;
        let course_id: Option<String> = row.get(15).map_err(|e| DomainError::Database(e.to_string()))?;
        let group_id: Option<String> = row.get(16).map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(Student {
            id: row.get(0).map_err(|e| DomainError::Database(e.to_string()))?,
            user_id: row.get(1).map_err(|e| DomainError::Database(e.to_string()))?,
            first_name: row.get(2).map_err(|e| DomainError::Database(e.to_string()))?,
            last_name: row.get(3).map_err(|e| DomainError::Database(e.to_string()))?,
            document_type: row.get(4).map_err(|e| DomainError::Database(e.to_string()))?,
            document_number: row.get(5).map_err(|e| DomainError::Database(e.to_string()))?,
            email: row.get(6).map_err(|e| DomainError::Database(e.to_string()))?,
            phone: row.get(7).map_err(|e| DomainError::Database(e.to_string()))?,
            address: row.get::<Option<String>>(17).map_err(|e| DomainError::Database(e.to_string()))?,
            birth_date: birth_date_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
                    .or_else(|| {
                        chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                            .ok()
                            .and_then(|d| d.and_hms_opt(0, 0, 0))
                            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                    })
            }),
            guardian_name,
            guardian_document,
            guardian_phone,
            course_id,
            group_id,
            active: active != 0,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

impl Default for SqliteStudentRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StudentRepository for SqliteStudentRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Student>, DomainError> {
        let sql = "SELECT id, user_id, first_name, last_name, document_type, document_number,
                          email, phone, birth_date, guardian_name, guardian_document, guardian_phone,
                          active, created_at, updated_at, course_id, group_id, address
                   FROM students WHERE id = ?1";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![id.to_string()]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => Ok(Some(Self::row_to_student(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_user_id(&self, user_id: &str) -> Result<Option<Student>, DomainError> {
        let sql = "SELECT id, user_id, first_name, last_name, document_type, document_number,
                          email, phone, birth_date, guardian_name, guardian_document, guardian_phone,
                          active, created_at, updated_at, course_id, group_id, address
                   FROM students WHERE user_id = ?1";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![user_id.to_string()]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        match rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            Some(row) => Ok(Some(Self::row_to_student(&row)?)),
            None => Ok(None),
        }
    }

    async fn save(&self, student: &Student) -> Result<(), DomainError> {
        let sql = "INSERT INTO students (id, user_id, first_name, last_name, document_type, 
                                      document_number, email, phone, birth_date, 
                                      guardian_name, guardian_document, guardian_phone,
                                      active, created_at, updated_at, course_id, group_id, address)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)";

        let birth_date = student.birth_date.map(|dt| dt.to_rfc3339());
        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        conn.execute(
            sql,
            libsql::params![
                student.id.clone(),
                student.user_id.clone(),
                student.first_name.clone(),
                student.last_name.clone(),
                student.document_type.clone(),
                student.document_number.clone(),
                student.email.clone(),
                student.phone.clone(),
                birth_date,
                student.guardian_name.clone(),
                student.guardian_document.clone(),
                student.guardian_phone.clone(),
                if student.active { 1 } else { 0 },
                student.created_at.to_rfc3339(),
                student.updated_at.to_rfc3339(),
                student.course_id.clone(),
                student.group_id.clone(),
                student.address.clone(),
            ],
        )
        .await
        .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, student: &Student) -> Result<(), DomainError> {
        let sql = "UPDATE students 
                   SET first_name = ?1, last_name = ?2, email = ?3, phone = ?4, address = ?5, 
                       birth_date = ?6, guardian_name = ?7, guardian_document = ?8, guardian_phone = ?9,
                       active = ?10, updated_at = ?11, course_id = ?12, group_id = ?13
                   WHERE id = ?14";

        let birth_date = student.birth_date.map(|dt| dt.to_rfc3339());
        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(
                sql,
                libsql::params![
                    student.first_name.clone(),
                    student.last_name.clone(),
                    student.email.clone(),
                    student.phone.clone(),
                    student.address.clone(),
                    birth_date,
                    student.guardian_name.clone(),
                    student.guardian_document.clone(),
                    student.guardian_phone.clone(),
                    if student.active { 1 } else { 0 },
                    Utc::now().to_rfc3339(),
                    student.course_id.clone(),
                    student.group_id.clone(),
                    student.id.clone(),
                ],
            )
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Student", &student.id));
        }
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "UPDATE students SET active = 0, updated_at = ?1 WHERE id = ?2";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let affected = conn
            .execute(sql, libsql::params![Utc::now().to_rfc3339(), id.to_string()])
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Student", id));
        }
        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Student>, DomainError> {
        let sql = "SELECT id, user_id, first_name, last_name, document_type, document_number,
                          email, phone, birth_date, guardian_name, guardian_document, guardian_phone,
                          active, created_at, updated_at, course_id, group_id, address
                   FROM students WHERE active = 1 ORDER BY last_name, first_name";

        let conn = local_db::get_db().connect().map_err(|e| DomainError::Database(e.to_string()))?;
        let mut rows = conn.query(sql, libsql::params![]).await.map_err(|e| DomainError::Database(e.to_string()))?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| DomainError::Database(e.to_string()))? {
            results.push(Self::row_to_student(&row)?);
        }
        Ok(results)
    }
}
