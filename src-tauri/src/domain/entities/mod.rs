//! Domain Entities - Academix MVP
//!
//! Pure domain models with no framework annotations or persistence concerns.

pub mod attendance;
pub mod course;
pub mod group;
pub mod payment;
pub mod student;
pub mod user;

pub use attendance::*;
pub use course::*;
pub use group::*;
pub use payment::*;
pub use student::*;
pub use user::*;
