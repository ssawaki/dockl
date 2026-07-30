use tauri::State;

use crate::docker_bridge::{ContainerActionKind, ContainerDetail, ContainerSummary};
use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub async fn list_containers(
    state: State<'_, AppState>,
    all: bool,
) -> Result<Vec<ContainerSummary>, AppError> {
    let guard = state.connection.read().await;
    let connection = guard.as_ref().ok_or(AppError::NotConfigured)?;
    connection.list_containers(all).await
}

#[tauri::command]
pub async fn container_action(
    state: State<'_, AppState>,
    id: String,
    action: ContainerActionKind,
) -> Result<(), AppError> {
    let guard = state.connection.read().await;
    let connection = guard.as_ref().ok_or(AppError::NotConfigured)?;
    connection.container_action(&id, action).await
}

#[tauri::command]
pub async fn inspect_container(
    state: State<'_, AppState>,
    id: String,
) -> Result<ContainerDetail, AppError> {
    let guard = state.connection.read().await;
    let connection = guard.as_ref().ok_or(AppError::NotConfigured)?;
    connection.inspect_container(&id).await
}
