//! Student SQLite Repository
//!
//! Implements StudentRepository using SQLite.

use crate::application::ports::StudentRepository;
use crate::domain::entities::Student;
use crate::domain::errors::DomainError;
use crate::infrastructure::database;
use chrono::{DateTime, Utc};

/// SQLite implementation of StudentRepository
#[derive(Clone)]
pub struct SqliteStudentRepository;

impl SqliteStudentRepository {
    pub fn new() -> Self {
        Self
    }

    fn row_to_student(row: &rusqlite::Row<'_>) -> rusqlite::Result<Student> {
        let birth_date_str: Option<String> = row.get(8)?;
        let guardian_name: Option<String> = row.get(9)?;
        let guardian_document: Option<String> = row.get(10)?;
        let guardian_phone: Option<String> = row.get(11)?;
        let active: i32 = row.get(12)?;
        let created_str: String = row.get(13)?;
        let updated_str: String = row.get(14)?;
        let course_id: Option<String> = row.get(15)?;
        let group_id: Option<String> = row.get(16)?;

        Ok(Student {
            id: row.get(0)?,
            user_id: row.get(1)?,
            first_name: row.get(2)?,
            last_name: row.get(3)?,
            document_type: row.get(4)?,
            document_number: row.get(5)?,
            email: row.get(6)?,
            phone: row.get(7)?,
            address: row.get::<_, Option<String>>(17)?,
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

impl StudentRepository for SqliteStudentRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Student>, DomainError> {
        let sql = "SELECT id, user_id, first_name, last_name, document_type, document_number,
                          email, phone, birth_date, guardian_name, guardian_document, guardian_phone,
                          active, created_at, updated_at, course_id, group_id, address
                   FROM students WHERE id = ?";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [id], Self::row_to_student) {
            Ok(student) => Ok(Some(student)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn find_by_user_id(&self, user_id: &str) -> Result<Option<Student>, DomainError> {
        let sql = "SELECT id, user_id, first_name, last_name, document_type, document_number,
                          email, phone, birth_date, guardian_name, guardian_document, guardian_phone,
                          active, created_at, updated_at, course_id, group_id, address
                   FROM students WHERE user_id = ?";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [user_id], Self::row_to_student) {
            Ok(student) => Ok(Some(student)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn save(&self, student: &Student) -> Result<(), DomainError> {
        let sql = "INSERT INTO students (id, user_id, first_name, last_name, document_type, 
                                      document_number, email, phone, birth_date, 
                                      guardian_name, guardian_document, guardian_phone,
                                      active, created_at, updated_at, course_id, group_id, address)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        let birth_date = student.birth_date.map(|dt| dt.to_rfc3339());
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        conn.execute(
            sql,
            rusqlite::params![
                student.id,
                student.user_id,
                student.first_name,
                student.last_name,
                student.document_type,
                student.document_number,
                student.email,
                student.phone,
                birth_date,
                student.guardian_name,
                student.guardian_document,
                student.guardian_phone,
                if student.active { 1 } else { 0 },
                student.created_at.to_rfc3339(),
                student.updated_at.to_rfc3339(),
                student.course_id,
                student.group_id,
                student.address,
            ],
        )
        .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    fn update(&self, student: &Student) -> Result<(), DomainError> {
        let sql = "UPDATE students 
                   SET first_name = ?, last_name = ?, email = ?, phone = ?, address = ?, 
                       birth_date = ?, guardian_name = ?, guardian_document = ?, guardian_phone = ?,
                       active = ?, updated_at = ?, course_id = ?, group_id = ?
                   WHERE id = ?";

        let birth_date = student.birth_date.map(|dt| dt.to_rfc3339());
        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let affected = conn
            .execute(
                sql,
                rusqlite::params![
                    student.first_name,
                    student.last_name,
                    student.email,
                    student.phone,
                    student.address,
                    birth_date,
                    student.guardian_name,
                    student.guardian_document,
                    student.guardian_phone,
                    if student.active { 1 } else { 0 },
                    Utc::now().to_rfc3339(),
                    student.course_id,
                    student.group_id,
                    student.id,
                ],
            )
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Student", &student.id));
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "UPDATE students SET active = 0, updated_at = ? WHERE id = ?";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let affected = conn
            .execute(sql, rusqlite::params![Utc::now().to_rfc3339(), id])
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Student", id));
        }
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Student>, DomainError> {
        let sql = "SELECT id, user_id, first_name, last_name, document_type, document_number,
                          email, phone, birth_date, guardian_name, guardian_document, guardian_phone,
                          active, created_at, updated_at, course_id, group_id, address
                   FROM students WHERE active = 1 ORDER BY last_name, first_name";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map([], Self::row_to_student)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }
}
