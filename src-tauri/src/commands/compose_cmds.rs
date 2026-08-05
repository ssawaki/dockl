use tauri::State;

use crate::compose;
use crate::error::AppError;
use crate::state::AppState;

/// Returns `docker compose`'s own combined stdout+stderr (see `wsl::run_docker_verbose`
/// — its progress output actually goes to stderr) so the frontend can show it on demand
/// instead of discarding it; up/down especially produce output worth seeing (pulling
/// images, creating networks, starting/removing each service).
#[tauri::command]
pub async fn compose_action(
    state: State<'_, AppState>,
    project: String,
    config_files: Vec<String>,
    action: String,
) -> Result<String, AppError> {
    let distro = state
        .current_distro
        .read()
        .await
        .clone()
        .ok_or(AppError::NotConfigured)?;

    compose::compose_action(&distro, &project, &config_files, &action).await
}
