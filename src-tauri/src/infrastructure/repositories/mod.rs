//! Repository Implementations - Academix MVP
//!
//! SQLite repository implementations for production persistence.
//! In-memory implementations removed - use only sqlite/* for all data persistence.

// Production SQLite implementations ONLY
pub mod sqlite;

// Keep balance of ports/domain interfaces but don't load in-memory impls
// The in-memory repos were only for early testing and are replaced by sqlite/ implementations

// Re-export SQLite repositories for all modules
pub use sqlite::*;
