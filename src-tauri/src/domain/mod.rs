//! Domain Layer - Academix MVP
//!
//! This module contains the core business logic with zero external dependencies.
//! Following Hexagonal Architecture principles - domain is pure.

pub mod entities;
pub mod errors;
pub mod value_objects;

// Re-export commonly used types
pub use entities::*;
pub use errors::DomainError;
pub use value_objects::*;
