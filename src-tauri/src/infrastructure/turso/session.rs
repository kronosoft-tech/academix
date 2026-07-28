//! Current authenticated user session.
//!
//! Holds the current authenticated user's identity.
//! Set after login, cleared on logout.
//! Each command resolves the session token and sets user_id before accessing data.

/// Holds the current authenticated user's identity.
///
/// Set after login, cleared on logout.
/// Each command resolves the session token and sets user_id before accessing data.
#[derive(Debug, Clone, Default)]
pub struct CurrentSession {
    pub user_id: Option<String>,
}
