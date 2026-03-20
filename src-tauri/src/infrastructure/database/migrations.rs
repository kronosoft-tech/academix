//! Database Migrations Module
//!
//! Manages database schema migrations for Academix MVP.

/// Migration file representation
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub sql: String,
}

impl Migration {
    /// Create a new migration from version, name and SQL content
    pub fn new(version: u32, name: impl Into<String>, sql: impl Into<String>) -> Self {
        Self {
            version,
            name: name.into(),
            sql: sql.into(),
        }
    }
}

/// Get all migrations for the database
pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration::new(
            1,
            "initial_schema",
            include_str!("../../../migrations/001_initial_schema.sql"),
        ),
        Migration::new(
            2,
            "seed_admin_user",
            include_str!("../../../migrations/002_seed_admin.sql"),
        ),
        Migration::new(
            3,
            "add_guardian_and_schedule_fields",
            include_str!("../../../migrations/003_add_guardian_and_schedule_fields.sql"),
        ),
        Migration::new(
            4,
            "add_student_enrollment_columns",
            include_str!("../../../migrations/004_add_student_enrollment_columns.sql"),
        ),
    ]
}
