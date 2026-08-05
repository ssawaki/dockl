use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

/// Returns one `docker stats --no-stream` JSON snapshot for the given container
/// (parsed on the frontend — see `src/lib/dockerStats.ts`), polled repeatedly by the UI.
/// Routed through `state.connection` like every other command so TCP/bollard-mode users
/// get this over HTTP instead of always paying a `wsl.exe` process spawn per poll.
#[tauri::command]
pub async fn get_container_stats(state: State<'_, AppState>, id: String) -> Result<String, AppError> {
    let guard = state.connection.read().await;
    let connection = guard.as_ref().ok_or(AppError::NotConfigured)?;
    connection.container_stats(&id).await
}

/// How many CPUs the Docker daemon itself sees, used as the frontend's fallback
/// "MAX %" ceiling for containers that aren't CPU-limited (`ContainerDetail::cpu_limit_cores`
/// is `None`) — an unlimited container can use every core the daemon has.
#[tauri::command]
pub async fn get_host_cpu_count(state: State<'_, AppState>) -> Result<u32, AppError> {
    let guard = state.connection.read().await;
    let connection = guard.as_ref().ok_or(AppError::NotConfigured)?;
    connection.host_cpu_count().await
}

/// Returns the container's disk usage as Docker itself formats it, e.g.
/// `"16.4kB (virtual 146MB)"` — the writable layer's own size, and in parens the
/// virtual size (that layer plus the shared image).
#[tauri::command]
pub async fn get_container_disk_usage(state: State<'_, AppState>, id: String) -> Result<String, AppError> {
    let guard = state.connection.read().await;
    let connection = guard.as_ref().ok_or(AppError::NotConfigured)?;
    connection.container_disk_usage(&id).await
}
