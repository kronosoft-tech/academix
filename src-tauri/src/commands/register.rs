//! Register Command
//!
//! Public registration endpoint (no auth required).

use crate::application::dto::{RegisterUserRequest, RegisterUserResponse};
use crate::application::use_cases::RegisterUserUseCase;
use crate::infrastructure::repositories::SqliteUserRepository;
use std::sync::Arc;
use tauri::{command, State};

/// Register a new user (public endpoint - no auth required)
#[command]
pub async fn register_user(
    request: RegisterUserRequest,
    pool: State<'_, Arc<crate::infrastructure::database::SqlitePool>>,
) -> Result<RegisterUserResponse, String> {
    let repository = SqliteUserRepository::new(Arc::clone(&pool));
    let use_case = RegisterUserUseCase::new(repository);

    use_case
        .execute(request)
        .map_err(|e| e.to_string())
}