//! Repository Implementations - Academix MVP
//!
//! In-memory and SQLite repository implementations.

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

// Re-export InMemory repositories
pub use accounting::InMemoryAccountCategoryRepository;
pub use accounting::InMemoryAccountingEntryRepository;
pub use attendance::InMemoryAttendanceRepository;
pub use course::InMemoryCourseRepository;
pub use employee::InMemoryEmployeeRepository;
pub use group::InMemoryGroupRepository;
pub use invoice::InMemoryInvoiceLineRepository;
pub use invoice::InMemoryInvoiceRepository;
pub use payment::InMemoryPaymentRepository;
pub use payroll::InMemoryPayrollEntryRepository;
pub use payroll::InMemoryPayrollRepository;
pub use session::InMemorySessionRepository;
pub use student::InMemoryStudentRepository;
pub use user::InMemoryUserRepository;

// Re-export SQLite repositories
pub use sqlite::*;
