//! Application Layer - Academix MVP
//!
//! This module contains use cases and ports (interfaces) for the application.
//! Following Hexagonal Architecture - application orchestrates domain logic.

pub mod dto;
pub mod errors;
pub mod ports;
pub mod use_cases;

// Re-export commonly used types
pub use errors::ApplicationError;
