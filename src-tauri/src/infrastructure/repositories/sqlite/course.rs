//! Course SQLite Repository
//!
//! Implements CourseRepository using SQLite.

use crate::application::ports::CourseRepository;
use crate::domain::entities::course::{Course, CourseStatus};
use crate::domain::errors::DomainError;
use crate::infrastructure::database::SqlitePool;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// SQLite implementation of CourseRepository
pub struct SqliteCourseRepository {
    pool: Arc<SqlitePool>,
}

impl SqliteCourseRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
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

impl CourseRepository for SqliteCourseRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Course>, DomainError> {
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                   FROM courses WHERE id = ?";

        self.pool
            .query_row(sql, &[&id], Self::row_to_course)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn find_by_code(&self, code: &str) -> Result<Option<Course>, DomainError> {
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                   FROM courses WHERE code = ?";

        self.pool
            .query_row(sql, &[&code], Self::row_to_course)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn save(&self, course: &Course) -> Result<(), DomainError> {
        let sql = "INSERT INTO courses (id, name, description, code, credits, status, created_at, updated_at, price, duration)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        self.pool
            .execute(
                sql,
                &[
                    &course.id,
                    &course.name,
                    &course.description,
                    &course.code,
                    &course.credits.to_string(),
                    &course.status.as_str().to_string(),
                    &course.created_at.to_rfc3339(),
                    &course.updated_at.to_rfc3339(),
                    &course.price.to_string(),
                    &course.duration.to_string(),
                ],
            )
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    fn update(&self, course: &Course) -> Result<(), DomainError> {
        let sql = "UPDATE courses 
                   SET name = ?, description = ?, code = ?, credits = ?, status = ?, updated_at = ?, price = ?, duration = ?
                   WHERE id = ?";

        let affected = self
            .pool
            .execute(
                sql,
                &[
                    &course.name,
                    &course.description,
                    &course.code,
                    &course.credits.to_string(),
                    &course.status.as_str().to_string(),
                    &Utc::now().to_rfc3339(),
                    &course.price.to_string(),
                    &course.duration.to_string(),
                    &course.id,
                ],
            )
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Course", &course.id));
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        // Soft delete - just archive the course
        let sql = "UPDATE courses SET status = 'archived', updated_at = ? WHERE id = ?";

        let affected = self
            .pool
            .execute(sql, &[&Utc::now().to_rfc3339(), &id])
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        if affected == 0 {
            return Err(DomainError::not_found("Course", id));
        }
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Course>, DomainError> {
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                   FROM courses WHERE status != 'archived' ORDER BY name";

        self.pool
            .query(sql, &[], Self::row_to_course)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn find_all_archived(&self) -> Result<Vec<Course>, DomainError> {
        let sql =
            "SELECT id, name, description, code, credits, status, created_at, updated_at, price, duration 
                   FROM courses WHERE status = 'archived' ORDER BY name";

        self.pool
            .query(sql, &[], Self::row_to_course)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }

    fn restore(&self, id: &str) -> Result<(), DomainError> {
        let sql = "UPDATE courses SET status = 'draft', updated_at = ? WHERE id = ?";

        self.pool
            .execute(sql, &[&Utc::now().to_rfc3339(), &id])
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        Ok(())
    }

    fn hard_delete(&self, id: &str) -> Result<(), DomainError> {
        let sql = "DELETE FROM courses WHERE id = ?";

        self.pool
            .execute(sql, &[&id])
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        Ok(())
    }
}
