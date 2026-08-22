use tauri::State;

use crate::docker_bridge::{ContainerActionKind, ContainerDetail, ContainerSummary};
use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub async fn list_containers(
    state: State<'_, AppState>,
    all: bool,
) -> Result<Vec<ContainerSummary>, AppError> {
    let connection = state.connection().await?;
    connection.list_containers(all).await
}

#[tauri::command]
pub async fn container_action(
    state: State<'_, AppState>,
    id: String,
    action: ContainerActionKind,
) -> Result<(), AppError> {
    let connection = state.connection().await?;
    connection.container_action(&id, action).await
}

#[tauri::command]
pub async fn inspect_container(
    state: State<'_, AppState>,
    id: String,
) -> Result<ContainerDetail, AppError> {
    let connection = state.connection().await?;
    connection.inspect_container(&id).await
}
