//! Application Ports - Repository Interfaces
//!
//! These traits define the interfaces (ports) for the application layer.
//! Infrastructure adapters implement these ports.

pub mod attendance;
pub mod course;
pub mod group;
pub mod payment;
pub mod session;
pub mod student;
pub mod user;

pub use attendance::*;
pub use course::*;
pub use group::*;
pub use payment::*;
pub use session::*;
pub use student::*;
pub use user::*;
