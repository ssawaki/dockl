use tauri::State;

use crate::docker_bridge::VolumeSummary;
use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub async fn list_volumes(state: State<'_, AppState>) -> Result<Vec<VolumeSummary>, AppError> {
    let guard = state.connection.read().await;
    let connection = guard.as_ref().ok_or(AppError::NotConfigured)?;
    connection.list_volumes().await
}

#[tauri::command]
pub async fn remove_volume(state: State<'_, AppState>, name: String) -> Result<(), AppError> {
    let guard = state.connection.read().await;
    let connection = guard.as_ref().ok_or(AppError::NotConfigured)?;
    connection.remove_volume(&name).await
}

#[tauri::command]
pub async fn prune_volumes(state: State<'_, AppState>, all: bool) -> Result<String, AppError> {
    let guard = state.connection.read().await;
    let connection = guard.as_ref().ok_or(AppError::NotConfigured)?;
    connection.prune_volumes(all).await
}
