//! Register Command
//!
//! Public registration endpoint (no auth required).

use crate::application::dto::{RegisterUserRequest, RegisterUserResponse};
use crate::application::use_cases::RegisterUserUseCase;
use crate::infrastructure::repositories::MemoryBackedUserRepository;
use crate::infrastructure::turso::control_plane::ControlPlaneRepository;
use crate::infrastructure::turso::provisioning::TursoProvisioningService;
use std::sync::Arc;
use tauri::{command, State};

/// Register a new user (public endpoint - no auth required)
#[command]
pub async fn register_user(
    request: RegisterUserRequest,
    state: State<'_, super::auth::AppState>,
    control_plane: State<'_, Option<Arc<ControlPlaneRepository>>>,
    provisioning: State<'_, Option<Arc<TursoProvisioningService>>>,
) -> Result<RegisterUserResponse, String> {
    let repository = MemoryBackedUserRepository::new(
        state.connection_manager.clone(),
        state.memory_buffer.clone(),
        state.session.clone(),
    );

    // Clone the optional services from managed state
    let cp: Option<Arc<ControlPlaneRepository>> = control_plane.inner().clone();
    let prov: Option<Arc<TursoProvisioningService>> = provisioning.inner().clone();

    let use_case = RegisterUserUseCase::new(repository, cp, prov);

    use_case
        .execute(request)
        .await
        .map_err(|e| e.to_string())
}
