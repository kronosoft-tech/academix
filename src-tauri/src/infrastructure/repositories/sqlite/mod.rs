//! SQLite Repository Implementations
//!
//! These modules implement the repository ports using SQLite.

pub mod accounting;
pub mod attendance;
pub mod course;
pub mod group;
pub mod invoice;
pub mod payment;
pub mod session;
pub mod settings;
pub mod student;
pub mod user;

// Re-export concrete implementations
pub use accounting::SqliteAccountingEntryRepository;
pub use attendance::SqliteAttendanceRepository;
pub use course::SqliteCourseRepository;
pub use group::SqliteGroupRepository;
pub use invoice::SqliteInvoiceLineRepository;
pub use invoice::SqliteInvoiceRepository;
pub use payment::SqlitePaymentRepository;
pub use session::SqliteSessionRepository;
pub use settings::SqliteSettingsRepository;
pub use student::SqliteStudentRepository;
pub use user::SqliteUserRepository;
