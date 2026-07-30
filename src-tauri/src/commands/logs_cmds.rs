use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
pub async fn stream_logs(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    tail: u32,
) -> Result<String, AppError> {
    let distro = state
        .current_distro
        .read()
        .await
        .clone()
        .ok_or(AppError::NotConfigured)?;

    state.log_streams.start(app, distro, id, tail).await
}

#[tauri::command]
pub async fn stop_log_stream(state: State<'_, AppState>, stream_id: String) -> Result<(), AppError> {
    state.log_streams.stop(&stream_id).await
}
