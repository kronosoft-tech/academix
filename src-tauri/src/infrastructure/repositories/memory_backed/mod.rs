//! MemoryBacked Repository Implementations
//!
//! These modules implement the repository ports using a MemoryBuffer
//! write-back cache backed by Turso cloud databases.
//! Phase 5a: Infrastructure — MemoryBuffer-backed pattern foundation.
//! Phase 5b: 6 MemoryBacked repositories — User, Session, Student, Course, Payment, Group.
//! Phase 5c: Complex repositories — AccountingEntry, Attendance, Invoice, InvoiceLine.

pub mod settings;
pub mod user;
pub mod session;
pub mod student;
pub mod course;
pub mod payment;
pub mod group;
pub mod accounting;
pub mod attendance;
pub mod invoice;
pub mod invoice_line;

pub use settings::MemoryBackedSettingsRepository;
pub use user::MemoryBackedUserRepository;
pub use session::MemoryBackedSessionRepository;
pub use student::MemoryBackedStudentRepository;
pub use course::MemoryBackedCourseRepository;
pub use payment::MemoryBackedPaymentRepository;
pub use group::MemoryBackedGroupRepository;
pub use accounting::MemoryBackedAccountingEntryRepository;
pub use attendance::MemoryBackedAttendanceRepository;
pub use invoice::MemoryBackedInvoiceRepository;
pub use invoice_line::MemoryBackedInvoiceLineRepository;
