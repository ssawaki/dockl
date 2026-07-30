use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::state::AppState;

/// Attaches an interactive shell inside a running container, via
/// `wsl.exe -d <distro> -- docker exec -it <id> <shell>`.
#[tauri::command]
pub async fn start_attach_session(
    app: AppHandle,
    state: State<'_, AppState>,
    container_id: String,
    shell: Option<String>,
    cols: u16,
    rows: u16,
) -> Result<String, AppError> {
    let distro = state
        .current_distro
        .read()
        .await
        .clone()
        .ok_or(AppError::NotConfigured)?;

    let shell = shell.unwrap_or_else(|| "sh".to_string());
    let args = vec![
        "-d".to_string(),
        distro,
        "--".to_string(),
        "docker".to_string(),
        "exec".to_string(),
        "-it".to_string(),
        container_id,
        shell,
    ];

    state.pty_sessions.start(app, args, cols, rows)
}

/// Opens a plain interactive shell into the connected WSL2 distro, independent of any
/// container — the "open a WSL shell" quick action.
#[tauri::command]
pub async fn start_wsl_shell_session(
    app: AppHandle,
    state: State<'_, AppState>,
    cols: u16,
    rows: u16,
) -> Result<String, AppError> {
    let distro = state
        .current_distro
        .read()
        .await
        .clone()
        .ok_or(AppError::NotConfigured)?;

    let args = vec!["-d".to_string(), distro];
    state.pty_sessions.start(app, args, cols, rows)
}

#[tauri::command]
pub fn pty_write(state: State<'_, AppState>, session_id: String, data: String) -> Result<(), AppError> {
    state.pty_sessions.write(&session_id, &data)
}

#[tauri::command]
pub fn pty_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), AppError> {
    state.pty_sessions.resize(&session_id, cols, rows)
}

#[tauri::command]
pub fn pty_close(state: State<'_, AppState>, session_id: String) -> Result<(), AppError> {
    state.pty_sessions.close(&session_id)
}
