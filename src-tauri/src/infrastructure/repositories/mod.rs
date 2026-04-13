//! Repository Implementations - Academix MVP
//!
//! In-memory and SQLite repository implementations.

pub mod attendance;
pub mod course;
pub mod group;
pub mod payment;
pub mod session;
pub mod sqlite;
pub mod student;
pub mod user;

// Re-export InMemory repositories
pub use attendance::InMemoryAttendanceRepository;
pub use course::InMemoryCourseRepository;
pub use group::InMemoryGroupRepository;
pub use payment::InMemoryPaymentRepository;
pub use session::InMemorySessionRepository;
pub use student::InMemoryStudentRepository;
pub use user::InMemoryUserRepository;

// Re-export SQLite repositories
pub use sqlite::*;
