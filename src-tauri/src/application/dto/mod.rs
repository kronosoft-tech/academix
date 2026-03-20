//! Application DTOs - Academix MVP
//!
//! Data Transfer Objects for communication between layers.

pub mod attendance;
pub mod auth;
pub mod course;
pub mod group;
pub mod payment;
pub mod student;
pub mod user;

pub use attendance::*;
pub use auth::*;
pub use course::*;
pub use group::*;
pub use payment::*;
pub use student::*;
pub use user::*;
