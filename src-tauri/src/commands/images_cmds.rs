use tauri::State;

use crate::docker_bridge::ImageSummary;
use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub async fn list_images(state: State<'_, AppState>) -> Result<Vec<ImageSummary>, AppError> {
    let connection = state.connection().await?;
    connection.list_images().await
}

#[tauri::command]
pub async fn remove_image(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let connection = state.connection().await?;
    connection.remove_image(&id).await
}

#[tauri::command]
pub async fn prune_images(state: State<'_, AppState>, all: bool) -> Result<String, AppError> {
    let connection = state.connection().await?;
    connection.prune_images(all).await
}
