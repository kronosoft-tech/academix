//! Turso Platform API client.
//!
//! Creates and manages databases programmatically via the Turso Platform API.
//! Used by the registration flow to provision per-user databases.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Errors that can occur during Turso provisioning operations.
#[derive(Debug)]
pub enum ProvisioningError {
    /// HTTP/transport error
    Http(String),
    /// Turso API rate limit hit
    RateLimit,
    /// Database name conflict (retry with new slug)
    Conflict(String),
    /// Authentication/authorization failure
    Auth(String),
}

impl std::fmt::Display for ProvisioningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProvisioningError::Http(msg) => write!(f, "HTTP error: {}", msg),
            ProvisioningError::RateLimit => write!(f, "Turso API rate limit exceeded"),
            ProvisioningError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            ProvisioningError::Auth(msg) => write!(f, "Auth error: {}", msg),
        }
    }
}

impl std::error::Error for ProvisioningError {}

/// Information about a Turso database returned by the Platform API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub name: String,
    #[serde(rename = "hostname")]
    pub hostname: String,
}

/// Generate a unique database slug from an academy name.
///
/// Format: `academy-{normalized-name}-{4-char-suffix}`
///
/// Rules:
/// - Lowercase
/// - Replace spaces and underscores with hyphens
/// - Remove special characters
/// - Limit to 30 characters
/// - Append 4-char random suffix for uniqueness
pub fn generate_db_slug(academy_name: &str) -> String {
    let normalized: String = academy_name
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' | '-' => c,
            ' ' | '_' => '-',
            _ => '-',
        })
        .collect();

    // Remove consecutive hyphens
    let clean: String = normalized.chars().fold(String::new(), |mut acc, c| {
        if c == '-' && acc.ends_with('-') {
            // skip duplicate
        } else {
            acc.push(c);
        }
        acc
    });

    // Trim leading/trailing hyphens and limit to 30 chars
    let trimmed = clean
        .trim_matches('-')
        .chars()
        .take(30)
        .collect::<String>();

    // Add 4-char random suffix for uniqueness
    let suffix: String = Uuid::new_v4().to_string().chars().take(4).collect();

    format!("academy-{}-{}", trimmed, suffix)
}

/// Client for the Turso Platform API.
///
/// Used to create databases, generate auth tokens, and manage
/// database lifecycle programmatically.
pub struct TursoProvisioningService {
    api_token: String,
    org: String,
    client: Client,
}

impl TursoProvisioningService {
    /// Create a new provisioning service.
    ///
    /// * `api_token` — Turso Platform API token (from superadmin env)
    /// * `org` — Turso organization slug (e.g., "academix")
    pub fn new(api_token: String, org: String) -> Self {
        Self {
            api_token,
            org,
            client: Client::new(),
        }
    }

    /// Create a new database in the Turso organization.
    ///
    /// `POST /v1/organizations/{org}/databases`
    ///
    /// If the name conflicts (already exists), retries up to 2 more times
    /// with a new slug suffix.
    pub async fn create_database(&self, name: &str) -> Result<DatabaseInfo, ProvisioningError> {
        let url = format!(
            "https://api.turso.tech/v1/organizations/{}/databases",
            self.org
        );

        #[derive(Serialize)]
        struct CreateRequest {
            name: String,
        }

        // Try to create — if name exists, retry with a new slug
        let mut attempt_name = name.to_string();
        for _ in 0..3 {
            let body = CreateRequest {
                name: attempt_name.clone(),
            };

            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_token))
                .json(&body)
                .send()
                .await
                .map_err(|e| ProvisioningError::Http(e.to_string()))?;

            if response.status().is_success() {
                return response
                    .json()
                    .await
                    .map_err(|e| ProvisioningError::Http(e.to_string()));
            }

            if response.status() == reqwest::StatusCode::CONFLICT {
                // Name taken, generate new slug
                let suffix: String =
                    Uuid::new_v4().to_string().chars().take(4).collect();
                attempt_name = format!("{}-{}", &name[..name.len().min(25)], suffix);
                continue;
            }

            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(ProvisioningError::RateLimit);
            }

            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(ProvisioningError::Http(format!(
                "Status {}: {}",
                status, body_text
            )));
        }

        Err(ProvisioningError::Conflict(
            "Could not create database: all name attempts conflicted".into(),
        ))
    }

    /// Generate an auth token for a database.
    ///
    /// `POST /v1/organizations/{org}/databases/{name}/auth/tokens`
    ///
    /// Returns the JWT token string.
    pub async fn create_auth_token(&self, db_name: &str) -> Result<String, ProvisioningError> {
        let url = format!(
            "https://api.turso.tech/v1/organizations/{}/databases/{}/auth/tokens",
            self.org, db_name
        );

        #[derive(Serialize)]
        struct TokenRequest {
            permission: String,
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            #[serde(rename = "jwt")]
            jwt: String,
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&TokenRequest {
                permission: "full-access".into(),
            })
            .send()
            .await
            .map_err(|e| ProvisioningError::Http(e.to_string()))?;

        if response.status().is_success() {
            let token_resp: TokenResponse = response
                .json()
                .await
                .map_err(|e| ProvisioningError::Http(e.to_string()))?;
            Ok(token_resp.jwt)
        } else {
            let status = response.status();
            Err(ProvisioningError::Auth(format!(
                "Token creation failed: {}",
                status
            )))
        }
    }

    /// List all databases in the organization (for superadmin).
    ///
    /// `GET /v1/organizations/{org}/databases`
    pub async fn list_databases(&self) -> Result<Vec<DatabaseInfo>, ProvisioningError> {
        let url = format!(
            "https://api.turso.tech/v1/organizations/{}/databases",
            self.org
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await
            .map_err(|e| ProvisioningError::Http(e.to_string()))?;

        #[derive(Deserialize)]
        struct ListResponse {
            databases: Vec<DatabaseInfo>,
        }

        let list: ListResponse = response
            .json()
            .await
            .map_err(|e| ProvisioningError::Http(e.to_string()))?;

        Ok(list.databases)
    }

    /// Delete a database (when user deletes account).
    ///
    /// `DELETE /v1/organizations/{org}/databases/{name}`
    pub async fn delete_database(&self, db_name: &str) -> Result<(), ProvisioningError> {
        let url = format!(
            "https://api.turso.tech/v1/organizations/{}/databases/{}",
            self.org, db_name
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await
            .map_err(|e| ProvisioningError::Http(e.to_string()))?;

        if response.status().is_success() || response.status() == reqwest::StatusCode::NOT_FOUND {
            Ok(())
        } else {
            let status = response.status();
            Err(ProvisioningError::Http(format!(
                "Delete failed: {}",
                status
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_db_slug_lowercases() {
        let slug = generate_db_slug("Music School");
        assert!(slug.starts_with("academy-music-school-"));
        assert_eq!(slug.chars().filter(|&c| c == '-').count(), 3); // academy-{name}-{suffix}
    }

    #[test]
    fn test_generate_db_slug_replaces_spaces() {
        let slug = generate_db_slug("My Academy Name");
        assert!(slug.starts_with("academy-my-academy-name-"));
    }

    #[test]
    fn test_generate_db_slug_removes_special_chars() {
        let slug = generate_db_slug("Hello@World!#2024");
        assert!(slug.starts_with("academy-hello-world-2024-"));
    }

    #[test]
    fn test_generate_db_slug_trims_long_names() {
        let long_name = "A very long academy name that should be truncated significantly";
        let slug = generate_db_slug(long_name);
        // academy- prefix (8) + 30 chars + - (1) + 4 chars suffix = ~43
        assert!(slug.len() <= "academy-".len() + 30 + 1 + 4);
        assert!(slug.starts_with("academy-"));
    }

    #[test]
    fn test_generate_db_slug_handles_underscores() {
        let slug = generate_db_slug("music_school");
        assert!(slug.starts_with("academy-music-school-"));
    }

    #[test]
    fn test_generate_db_slug_unique_suffix() {
        let slug1 = generate_db_slug("Test");
        let slug2 = generate_db_slug("Test");
        // Suffixes should be different (random)
        assert_ne!(slug1, slug2);
    }
}
