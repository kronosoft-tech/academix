//! Environment variable loader with validation and defaults.
//!
//! Provides a consistent way to load environment variables with optional defaults
//! and production-vs-development mode handling.

use std::env;

/// Environment error types
#[derive(Debug, Clone)]
pub enum EnvError {
    /// Required variable is missing
    Missing(&'static str),
    /// Variable is empty (set but blank)
    Empty(&'static str),
    /// Variable contains invalid UTF-8
    InvalidUtf8(&'static str),
}

impl std::fmt::Display for EnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvError::Missing(name) => write!(f, "missing required environment variable: {}", name),
            EnvError::Empty(name) => write!(f, "environment variable is empty: {}", name),
            EnvError::InvalidUtf8(name) => {
                write!(f, "environment variable contains invalid UTF-8: {}", name)
            }
        }
    }
}

impl std::error::Error for EnvError {}

/// Get an environment variable as String.
///
/// # Arguments
/// * `name` - Name of the environment variable
/// * `default` - Default value if variable is not set (can be None for required vars)
///
/// # Returns
/// * `Ok(String)` - The value (from env or default)
/// * `Err(EnvError)` - Variable is required but missing/empty
pub fn get_env_var(name: &'static str, default: Option<&'static str>) -> Result<String, EnvError> {
    match env::var(name) {
        Ok(value) => {
            if value.trim().is_empty() {
                if let Some(def) = default {
                    Ok(def.to_string())
                } else {
                    Err(EnvError::Empty(name))
                }
            } else {
                Ok(value)
            }
        }
        Err(std::env::VarError::NotPresent) => {
            if let Some(def) = default {
                Ok(def.to_string())
            } else {
                Err(EnvError::Missing(name))
            }
        }
        Err(std::env::VarError::NotUnicode(_)) => Err(EnvError::InvalidUtf8(name)),
    }
}

/// Check if we're running in production mode.
///
/// Production is detected by checking if APP_IDENTIFIER is set
/// to a non-default value (indicating explicit configuration).
pub fn is_production() -> bool {
    // If APP_IDENTIFIER is set and it's NOT the default, assume production
    let default_identifier = "com.luiferdev.academix";
    match env::var("APP_IDENTIFIER") {
        Ok(val) => !val.is_empty() && val != default_identifier,
        Err(_) => false,
    }
}

/// Turso configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct TursoConfig {
    pub control_plane_db_url: String,
    pub control_plane_db_token: String,
    pub turso_api_token: String,
    pub turso_org: String,
}

/// Load all Turso configuration from environment variables.
///
/// All four are required — the app cannot start without them
/// because the control plane is a Turso database.
pub fn load_turso_config() -> Result<TursoConfig, EnvError> {
    Ok(TursoConfig {
        control_plane_db_url: get_env_var("CONTROL_PLANE_DB_URL", None)?,
        control_plane_db_token: get_env_var("CONTROL_PLANE_DB_TOKEN", None)?,
        turso_api_token: get_env_var("TURSO_API_TOKEN", None)?,
        turso_org: get_env_var("TURSO_ORG", None)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_env_var_with_default() {
        env::remove_var("TEST_VAR_FOR_RUST");
        let result = get_env_var("TEST_VAR_FOR_RUST", Some("default_value"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "default_value");
    }

    #[test]
    fn test_get_env_var_missing_required() {
        env::remove_var("TEST_VAR_MISSING");
        let result = get_env_var("TEST_VAR_MISSING", None);
        assert!(matches!(result, Err(EnvError::Missing(_))));
    }

    #[test]
    fn test_get_env_var_explicit_value() {
        env::set_var("TEST_VAR_EXPLICIT", "explicit_value");
        let result = get_env_var("TEST_VAR_EXPLICIT", Some("default_value"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "explicit_value");
        env::remove_var("TEST_VAR_EXPLICIT");
    }

    #[test]
    fn test_get_env_var_empty_with_default() {
        env::set_var("TEST_VAR_EMPTY", "");
        let result = get_env_var("TEST_VAR_EMPTY", Some("fallback"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "fallback");
        env::remove_var("TEST_VAR_EMPTY");
    }

    #[test]
    fn test_is_production_default() {
        env::remove_var("APP_IDENTIFIER");
        assert!(!is_production());
    }

    #[test]
    fn test_is_production_custom() {
        env::set_var("APP_IDENTIFIER", "com.mycompany.academix");
        assert!(is_production());
        env::remove_var("APP_IDENTIFIER");
    }
}