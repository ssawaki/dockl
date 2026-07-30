use tauri::State;

use crate::compose;
use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub async fn compose_action(
    state: State<'_, AppState>,
    project: String,
    config_files: Vec<String>,
    action: String,
) -> Result<(), AppError> {
    let distro = state
        .current_distro
        .read()
        .await
        .clone()
        .ok_or(AppError::NotConfigured)?;

    compose::compose_action(&distro, &project, &config_files, &action).await?;
    Ok(())
}
