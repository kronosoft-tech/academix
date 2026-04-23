//! SQLite Repository Implementations
//!
//! These modules implement the repository ports using SQLite.

pub mod accounting;
pub mod attendance;
pub mod course;
pub mod employee;
pub mod group;
pub mod invoice;
pub mod liability;
pub mod payment;
pub mod payroll;
pub mod session;
pub mod student;
pub mod user;

// Re-export concrete implementations
pub use accounting::SqliteAccountCategoryRepository;
pub use accounting::SqliteAccountingEntryRepository;
pub use attendance::SqliteAttendanceRepository;
pub use course::SqliteCourseRepository;
pub use employee::SqliteEmployeeRepository;
pub use group::SqliteGroupRepository;
pub use invoice::SqliteInvoiceLineRepository;
pub use invoice::SqliteInvoiceRepository;
pub use liability::SqliteLiabilityRepository;
pub use liability::SqliteEquityRepository;
pub use payment::SqlitePaymentRepository;
pub use payroll::SqlitePayrollEntryRepository;
pub use payroll::SqlitePayrollRepository;
pub use session::SqliteSessionRepository;
pub use student::SqliteStudentRepository;
pub use user::SqliteUserRepository;
