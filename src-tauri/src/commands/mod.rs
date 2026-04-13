//! Commands Layer - Tauri Command Handlers
//!
//! This module contains the Tauri commands that bridge the frontend to the application layer.

pub mod accounting;
pub mod attendance;
pub mod auth;
pub mod base;
pub mod courses;
pub mod employees;
pub mod groups;
pub mod invoices;
pub mod payments;
pub mod payroll;
pub mod students;
pub mod users;

pub use accounting::*;
pub use attendance::*;
pub use auth::*;
pub use base::*;
pub use courses::*;
pub use employees::*;
pub use groups::*;
pub use invoices::*;
pub use payments::*;
pub use payroll::*;
pub use students::*;
pub use users::*;
