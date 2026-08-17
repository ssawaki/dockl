use tauri::State;

use crate::docker_bridge::DiskUsageEntry;
use crate::error::AppError;
use crate::state::AppState;

/// The commit this binary was built from, or `None` when it was built outside a checkout
/// (see `emit_git_hash` in build.rs).
#[tauri::command]
pub fn app_commit_hash() -> Option<String> {
    let hash = env!("DOCKL_GIT_HASH");
    (!hash.is_empty()).then(|| hash.to_owned())
}

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
