//! Infrastructure Layer - Academix MVP
//!
//! This module contains adapters that implement the application ports.
//! Database connections, repository implementations, and external services.

pub mod local_db;
pub mod password;
pub mod repositories;
pub mod turso;
pub mod updater;

pub use repositories::*;
