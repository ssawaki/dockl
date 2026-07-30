use std::sync::Arc;

use tauri::State;

use crate::docker_bridge::shell_out::ShellOutConnection;
use crate::docker_bridge::DockerConnection;
use crate::error::AppError;
use crate::state::AppState;
use crate::wsl::{self, DistroInfo};

#[tauri::command]
pub async fn setup_list_distros() -> Result<Vec<DistroInfo>, AppError> {
    wsl::list_distros().await
}

/// Connects using the `ShellOut` mode against the given distro. The connection is
/// verified with a real `docker ps` call before being committed to app state, so a
/// distro without a reachable Docker daemon surfaces as an error immediately rather
/// than failing later on the first container-list refresh.
#[tauri::command]
pub async fn setup_connect(state: State<'_, AppState>, distro: String) -> Result<(), AppError> {
    let connection = ShellOutConnection::new(distro.clone());
    connection.list_containers(true).await?;

    *state.connection.write().await = Some(Arc::new(connection));
    *state.current_distro.write().await = Some(distro);
    Ok(())
}

#[tauri::command]
pub async fn setup_current_distro(state: State<'_, AppState>) -> Result<Option<String>, AppError> {
    Ok(state.current_distro.read().await.clone())
}
