//! Repository Implementations - Academix MVP
//!
//! MemoryBuffer-backed repositories for Turso migration with SQLite fallback.

pub mod memory_backed;
pub mod sqlite; // keep for SQLite read fallback

pub use memory_backed::*;

// Explicit re-exports from sqlite for backward compatibility
// (commands and lib.rs still reference Sqlite*Repository types)
pub use sqlite::SqliteAccountingEntryRepository;
pub use sqlite::SqliteAttendanceRepository;
pub use sqlite::SqliteCourseRepository;
pub use sqlite::SqliteGroupRepository;
pub use sqlite::SqliteInvoiceRepository;
pub use sqlite::SqliteInvoiceLineRepository;
pub use sqlite::SqlitePaymentRepository;
pub use sqlite::SqliteSessionRepository;
pub use sqlite::SqliteSettingsRepository;
pub use sqlite::SqliteStudentRepository;
pub use sqlite::SqliteUserRepository;
