//! Repository Implementations - Academix MVP
//!
//! In-memory (solo para tests) y SQLite repository implementations.

pub mod accounting;
pub mod attendance;
pub mod course;
pub mod employee;
pub mod group;
pub mod invoice;
pub mod payment;
pub mod payroll;
pub mod session;
pub mod sqlite;
pub mod student;
pub mod user;

// Re-export SQLite repositories (usar estos en producción)
pub use sqlite::*;
