use tauri::State;

use crate::docker_bridge::VolumeSummary;
use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub async fn list_volumes(state: State<'_, AppState>) -> Result<Vec<VolumeSummary>, AppError> {
    let connection = state.connection().await?;
    let mut volumes = connection.list_volumes().await?;
    // Docker promises no particular order for volumes, and in practice it differs from
    // one call to the next, so the list visibly reshuffled on every refresh. Sorted here
    // rather than inside each Connection impl so the engine-API and shell-out backends
    // can't drift into two different orders.
    volumes.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(volumes)
}

#[tauri::command]
pub async fn remove_volume(state: State<'_, AppState>, name: String) -> Result<(), AppError> {
    let connection = state.connection().await?;
    connection.remove_volume(&name).await
}

#[tauri::command]
pub async fn prune_volumes(state: State<'_, AppState>, all: bool) -> Result<String, AppError> {
    let connection = state.connection().await?;
    connection.prune_volumes(all).await
}
