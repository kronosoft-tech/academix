//! SQLite Repository Implementations
//!
//! These modules implement the repository ports using SQLite.

pub mod attendance;
pub mod course;
pub mod group;
pub mod payment;
pub mod student;
pub mod user;

// Re-export concrete implementations
pub use attendance::SqliteAttendanceRepository;
pub use course::SqliteCourseRepository;
pub use group::SqliteGroupRepository;
pub use payment::SqlitePaymentRepository;
pub use student::SqliteStudentRepository;
pub use user::SqliteUserRepository;
