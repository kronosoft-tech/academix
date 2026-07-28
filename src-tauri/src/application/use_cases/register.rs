//! Registration Use Case
//!
//! Handles user registration with validation, password hashing,
//! and optional Turso database provisioning.

use crate::application::dto::{RegisterUserRequest, RegisterUserResponse};
use crate::application::errors::ApplicationError;
use crate::application::ports::UserRepository;
use crate::domain::entities::{Role, User};
use crate::domain::value_objects::{Email, Password};
use crate::infrastructure::password;
use crate::infrastructure::turso::connection_manager::run_migrations_on_db;
use crate::infrastructure::turso::control_plane::{ControlPlaneRepository, UserDbMapping};
use crate::infrastructure::turso::provisioning::{generate_db_slug, TursoProvisioningService};
use std::sync::Arc;
use uuid::Uuid;

/// Registration use case
pub struct RegisterUserUseCase<R: UserRepository> {
    user_repository: R,
    control_plane: Option<Arc<ControlPlaneRepository>>,
    provisioning: Option<Arc<TursoProvisioningService>>,
}

impl<R: UserRepository> RegisterUserUseCase<R> {
    pub fn new(
        user_repository: R,
        control_plane: Option<Arc<ControlPlaneRepository>>,
        provisioning: Option<Arc<TursoProvisioningService>>,
    ) -> Self {
        Self {
            user_repository,
            control_plane,
            provisioning,
        }
    }

    /// Register a new user with default role (not admin).
    ///
    /// If Turso services are configured, also provisions a per-user database
    /// and saves the mapping to the control plane.
    pub async fn execute(
        &self,
        request: RegisterUserRequest,
    ) -> Result<RegisterUserResponse, ApplicationError> {
        // Clone values needed before consuming request
        let email_str = request.email.clone();
        let academy_name = request.academy_name.clone();
        let name = request.name.clone();

        // Validate email format
        let email =
            Email::new(&email_str).map_err(|e| ApplicationError::Validation(e))?;

        // Validate password minimum 8 characters
        Password::validate_strength(&request.password)
            .map_err(|e| ApplicationError::Validation(e))?;

        // Check if email already exists
        if self.user_repository.exists_by_email(&email).await? {
            return Err(ApplicationError::Conflict(
                "Email already registered".to_string(),
            ));
        }

        // Hash password
        let password_hash = password::hash_password(&request.password)
            .map_err(|e| ApplicationError::Infrastructure(e.to_string()))?;

        // Generate user ID
        let user_id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string();

        // If Turso is configured, provision database and save mapping
        if let (Some(cp), Some(prov)) = (&self.control_plane, &self.provisioning) {
            let slug = generate_db_slug(&academy_name);

            let db_info = prov
                .create_database(&slug)
                .await
                .map_err(|e| {
                    ApplicationError::Infrastructure(format!(
                        "turso: failed to create database: {}",
                        e
                    ))
                })?;

            let token = prov
                .create_auth_token(&slug)
                .await
                .map_err(|e| {
                    ApplicationError::Infrastructure(format!(
                        "turso: failed to create auth token: {}",
                        e
                    ))
                })?;

            let now = chrono::Utc::now().to_rfc3339();

            // Connect to the newly created Turso DB and initialize it
            let db_url = format!("libsql://{}", db_info.hostname);
            let new_db = libsql::Builder::new_remote(db_url.clone(), token.clone())
                .build()
                .await
                .map_err(|e| {
                    ApplicationError::Infrastructure(format!(
                        "Failed to connect to new Turso DB: {}",
                        e
                    ))
                })?;

            // Run all 18 migrations on the new DB
            run_migrations_on_db(&new_db).await.map_err(|e| {
                ApplicationError::Infrastructure(format!(
                    "Failed to run migrations on new Turso DB: {}",
                    e
                ))
            })?;

            // Save user record directly to the new Turso DB
            {
                let conn = new_db.connect().map_err(|e| {
                    ApplicationError::Infrastructure(format!(
                        "Failed to connect to new DB for user insert: {}",
                        e
                    ))
                })?;

                conn.execute(
                    "INSERT INTO users (id, email, password_hash, name, role, is_active, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    libsql::params![
                        user_id.clone(),
                        email_str.clone(),
                        password_hash.clone(),
                        name.clone(),
                        "Empleado", // Default role for self-registration
                        1,          // is_active = true
                        now.clone(),
                        now.clone(),
                    ],
                )
                .await
                .map_err(|e| {
                    ApplicationError::Infrastructure(format!(
                        "Failed to save user in Turso DB: {}",
                        e
                    ))
                })?;

                println!(
                    "[TURSO] User '{}' saved to new Turso DB '{}'",
                    email_str, db_info.hostname
                );
            }

            let mapping = UserDbMapping {
                user_id: user_id.clone(),
                email: email_str.clone(),
                academy_name: academy_name.clone(),
                db_url,
                db_token: token,
                org: prov.org().to_string(),
                created_at: now,
            };

            // Save user→DB mapping to control plane
            if let Err(e) = cp.save_user_db(&mapping).await {
                eprintln!(
                    "[TURSO] Failed to save user DB mapping (non-fatal): {}",
                    e
                );
            }

            // Register the connection in ConnectionManager so login can find it
            // The ConnectionManager is not available here (it's owned by AppState),
            // but login will resolve via control plane → init_connection lazily.
        }

        // Create user with default role (empleado - not admin)
        let user = User::new(
            user_id,
            email_str,
            password_hash,
            name,
            Role::Empleado, // Default role - not admin for self-registration
        );

        // Save to local SQLite repository
        self.user_repository.save(&user).await?;

        Ok(RegisterUserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role.as_str().to_string(),
        })
    }
}
