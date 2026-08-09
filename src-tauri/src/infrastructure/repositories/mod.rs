//! Repository Implementations - Academix MVP

pub mod memory_backed;
pub mod sqlite;

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

pub use memory_backed::user::MemoryBackedUserRepository;
pub use memory_backed::session::MemoryBackedSessionRepository;
pub use memory_backed::student::MemoryBackedStudentRepository;
pub use memory_backed::course::MemoryBackedCourseRepository;
pub use memory_backed::group::MemoryBackedGroupRepository;
pub use memory_backed::payment::MemoryBackedPaymentRepository;
pub use memory_backed::attendance::MemoryBackedAttendanceRepository;
pub use memory_backed::invoice::MemoryBackedInvoiceRepository;
pub use memory_backed::invoice_line::MemoryBackedInvoiceLineRepository;
pub use memory_backed::accounting::MemoryBackedAccountingEntryRepository;
pub use memory_backed::settings::MemoryBackedSettingsRepository;
