use tauri::State;

use crate::docker_bridge::NetworkSummary;
use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub async fn list_networks(state: State<'_, AppState>) -> Result<Vec<NetworkSummary>, AppError> {
    let guard = state.connection.read().await;
    let connection = guard.as_ref().ok_or(AppError::NotConfigured)?;
    let mut networks = connection.list_networks().await?;
    // Same as volumes: no ordering guarantee from Docker, so the list reshuffled on every
    // refresh. Sorted here to keep the two backends consistent with each other.
    networks.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(networks)
}

#[tauri::command]
pub async fn remove_network(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let guard = state.connection.read().await;
    let connection = guard.as_ref().ok_or(AppError::NotConfigured)?;
    connection.remove_network(&id).await
}

#[tauri::command]
pub async fn prune_networks(state: State<'_, AppState>) -> Result<String, AppError> {
    let guard = state.connection.read().await;
    let connection = guard.as_ref().ok_or(AppError::NotConfigured)?;
    connection.prune_networks().await
}
