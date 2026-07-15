//! Application Ports - Repository Interfaces
//!
//! These traits define the interfaces (ports) for the application layer.
//! Infrastructure adapters implement these ports.

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

pub use accounting::*;
pub use attendance::*;
pub use course::*;
pub use group::*;
pub use invoice::*;
pub use payment::*;
pub use session::*;
pub use settings::*;
pub use student::*;
pub use user::*;
