//! Application Ports - Repository Interfaces
//!
//! These traits define the interfaces (ports) for the application layer.
//! Infrastructure adapters implement these ports.

pub mod accounting;
pub mod attendance;
pub mod course;
pub mod employee;
pub mod equity;
pub mod group;
pub mod invoice;
pub mod liability;
pub mod payment;
pub mod payroll;
pub mod session;
pub mod student;
pub mod user;

pub use accounting::*;
pub use attendance::*;
pub use course::*;
pub use employee::*;
pub use equity::*;
pub use group::*;
pub use invoice::*;
pub use liability::*;
pub use payment::*;
pub use payroll::*;
pub use session::*;
pub use student::*;
pub use user::*;
