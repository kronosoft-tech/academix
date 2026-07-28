//! MemoryBuffer-backed Repositories
//!
//! Writes go to MemoryBuffer (in-memory write buffer), reads check buffer cache first
//! and fallback to SQLite via database::open_connection().
//! List queries go directly to SQLite (no caching).

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

pub use accounting::MemoryBackedAccountingEntryRepository;
pub use attendance::MemoryBackedAttendanceRepository;
pub use course::MemoryBackedCourseRepository;
pub use group::MemoryBackedGroupRepository;
pub use invoice::MemoryBackedInvoiceLineRepository;
pub use invoice::MemoryBackedInvoiceRepository;
pub use payment::MemoryBackedPaymentRepository;
pub use session::MemoryBackedSessionRepository;
pub use settings::MemoryBackedSettingsRepository;
pub use student::MemoryBackedStudentRepository;
pub use user::MemoryBackedUserRepository;
