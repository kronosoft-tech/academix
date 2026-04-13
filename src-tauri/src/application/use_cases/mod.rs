//! Application Use Cases - Academix MVP
//!
//! Use cases orchestrate domain logic and delegate I/O to ports.

pub mod accounting;
pub mod attendance;
pub mod auth;
pub mod course;
pub mod employee;
pub mod group;
pub mod invoice;
pub mod payment;
pub mod payroll;
pub mod student;
pub mod user;

pub use accounting::*;
pub use attendance::*;
pub use auth::*;
pub use course::*;
pub use employee::*;
pub use group::*;
pub use invoice::*;
pub use payment::*;
pub use payroll::*;
pub use student::*;
pub use user::*;
