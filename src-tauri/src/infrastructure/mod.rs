//! Infrastructure Layer - Academix MVP
//!
//! This module contains adapters that implement the application ports.
//! Database connections, repository implementations, and external services.

pub mod database;
pub mod password;
pub mod repositories;
pub mod turso;

pub use database::*;
pub use repositories::*;
