//! Course SQLite Repository
//!
//! Implements CourseRepository using SQLite.

use crate::application::ports::CourseRepository;
use crate::domain::entities::course::{Course, CourseStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::database;
use chrono::{DateTime, Utc};

/// SQLite implementation of CourseRepository
#[derive(Clone)]
pub struct SqliteCourseRepository;

impl SqliteCourseRepository {
    pub fn new() -> Self {
        Self
    }

    fn row_to_course(row: &rusqlite::Row<'_>) -> rusqlite::Result<Course> {
        let status_str: String = row.get(5)?;
        let created_str: String = row.get(6)?;
        let updated_str: String = row.get(7)?;
        let price: f64 = row.get(8).unwrap_or(200000.0);
        let duration: i32 = row.get::<_, Option<i32>>(9)?.unwrap_or(0);

        Ok(Course {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            code: row.get(3)?,
            credits: row.get(4)?,
            price,
            duration,
            status: CourseStatus::from_str(&status_str).unwrap_or(CourseStatus::Draft),
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

impl Default for SqliteCourseRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl CourseRepository for SqliteCourseRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Course>, DomainError> {
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                   FROM courses WHERE id = ?";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [id], Self::row_to_course) {
            Ok(course) => Ok(Some(course)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn find_by_code(&self, code: &str) -> Result<Option<Course>, DomainError> {
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                   FROM courses WHERE code = ?";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        match conn.query_row(sql, [code], Self::row_to_course) {
            Ok(course) => Ok(Some(course)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DomainError::Validation(e.to_string())),
        }
    }

    fn save(&self, course: &Course) -> Result<(), DomainError> {
        eprintln!("[COURSE REPO] Saving course: id={}, name={}, status={}", course.id, course.name, course.status.as_str());
        let sql = "INSERT INTO courses (id, name, description, code, credits, status, created_at, updated_at, price, duration)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        conn.execute(
            sql,
            rusqlite::params![
                course.id,
                course.name,
                course.description,
                course.code,
                course.credits,
                course.status.as_str(),
                course.created_at.to_rfc3339(),
                course.updated_at.to_rfc3339(),
                course.price,
                course.duration,
            ],
        )
        .map_err(|e| {
            eprintln!("[COURSE REPO ERROR] Failed to insert: {}", e);
            DomainError::Validation(e.to_string())
        })?;

        eprintln!("[COURSE REPO] Course saved successfully");
        Ok(())
    }

    fn update(&self, course: &Course) -> Result<(), DomainError> {
        let sql = "UPDATE courses 
                   SET name = ?, description = ?, code = ?, credits = ?, status = ?, updated_at = ?, price = ?, duration = ?
                   WHERE id = ?";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let affected = conn
            .execute(
                sql,
                rusqlite::params![
                    course.name,
                    course.description,
                    course.code,
                    course.credits,
                    course.status.as_str(),
                    Utc::now().to_rfc3339(),
                    course.price,
                    course.duration,
                    course.id,
                ],
            )
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Course", &course.id));
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "UPDATE courses SET status = 'archived', updated_at = ? WHERE id = ?";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let affected = conn
            .execute(sql, rusqlite::params![Utc::now().to_rfc3339(), id])
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Course", id));
        }
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Course>, DomainError> {
        eprintln!("[COURSE REPO] Finding all courses (status != 'archived')");
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                   FROM courses WHERE status != 'archived' ORDER BY name";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map([], Self::row_to_course)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        let result = collected.map_err(|e| DomainError::Validation(e.to_string()))?;
        eprintln!("[COURSE REPO] Found {} courses", result.len());
        Ok(result)
    }

    fn find_all_archived(&self) -> Result<Vec<Course>, DomainError> {
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                   FROM courses WHERE status = 'archived' ORDER BY name";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        let mut stmt = conn.prepare(sql).map_err(|e| DomainError::Validation(e.to_string()))?;
        let rows = stmt
            .query_map([], Self::row_to_course)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        let collected: Result<Vec<_>, _> = rows.collect();
        collected.map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn restore(&self, id: &str) -> Result<(), DomainError> {
        let sql = "UPDATE courses SET status = 'draft', updated_at = ? WHERE id = ?";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        conn.execute(sql, rusqlite::params![Utc::now().to_rfc3339(), id])
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        Ok(())
    }

    fn hard_delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "DELETE FROM courses WHERE id = ?";

        let conn = database::open_connection().map_err(|e| DomainError::Database(e))?;
        conn.execute(sql, rusqlite::params![id])
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        Ok(())
    }
}
