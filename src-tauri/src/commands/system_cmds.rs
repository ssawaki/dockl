use tauri::State;

use crate::docker_bridge::DiskUsageEntry;
use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub async fn get_disk_usage(state: State<'_, AppState>) -> Result<Vec<DiskUsageEntry>, AppError> {
    let guard = state.connection.read().await;
    let connection = guard.as_ref().ok_or(AppError::NotConfigured)?;
    connection.system_df().await
}

#[tauri::command]
pub async fn prune_build_cache(state: State<'_, AppState>, all: bool) -> Result<String, AppError> {
    let guard = state.connection.read().await;
    let connection = guard.as_ref().ok_or(AppError::NotConfigured)?;
    connection.prune_build_cache(all).await
}
