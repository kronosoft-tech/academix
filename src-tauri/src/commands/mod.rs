//! Commands Layer - Tauri Command Handlers
//!
//! This module contains the Tauri commands that bridge the frontend to the application layer.

pub mod accounting;
pub mod admin;
pub mod attendance;
pub mod auth;
pub mod base;
pub mod courses;
pub mod groups;
pub mod invoices;
pub mod payments;
pub mod register;
pub mod settings;
pub mod students;
pub mod updater;
pub mod users;

pub use accounting::*;
pub use admin::*;
pub use attendance::*;
pub use auth::*;
pub use base::*;
pub use courses::*;
pub use groups::*;
pub use invoices::*;
pub use payments::*;
pub use register::*;
pub use settings::*;
pub use students::*;
pub use updater::*;
pub use users::*;
