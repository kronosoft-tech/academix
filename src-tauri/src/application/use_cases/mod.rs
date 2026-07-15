//! Application Use Cases - Academix MVP
//!
//! Use cases orchestrate domain logic and delegate I/O to ports.

pub mod accounting;
pub mod attendance;
pub mod auth;
pub mod course;
pub mod group;
pub mod invoice;
pub mod payment;
pub mod register;
pub mod settings;
pub mod student;
pub mod user;

pub use accounting::*;
pub use attendance::*;
pub use auth::*;
pub use course::*;
pub use group::*;
pub use invoice::*;
pub use payment::*;
pub use register::*;
pub use settings::*;
pub use student::*;
pub use user::*;
