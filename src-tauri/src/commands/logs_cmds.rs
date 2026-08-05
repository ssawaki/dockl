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

    let docker_args = vec!["logs".into(), "-f".into(), "--tail".into(), tail.to_string(), id];
    state.log_streams.start(app, distro, docker_args).await
}

#[tauri::command]
pub async fn stop_log_stream(state: State<'_, AppState>, stream_id: String) -> Result<(), AppError> {
    state.log_streams.stop(&stream_id).await
}

/// Like `stream_logs`, but for an entire Compose project: `docker compose logs -f`
/// interleaves every service's output into one stream, each line prefixed with its
/// service name, so the frontend doesn't need to juggle one stream per container.
#[tauri::command]
pub async fn stream_compose_logs(
    app: AppHandle,
    state: State<'_, AppState>,
    project: String,
    config_files: Vec<String>,
    tail: u32,
) -> Result<String, AppError> {
    let distro = state
        .current_distro
        .read()
        .await
        .clone()
        .ok_or(AppError::NotConfigured)?;

    // `--ansi always`: compose only colors each service's line prefix when it thinks
    // it's writing to a TTY, which stdout never is here (it's piped to us) — without
    // this, every line would come through as plain, uncolored text.
    let mut docker_args = vec![
        "compose".into(),
        "-p".into(),
        project,
        "--ansi".into(),
        "always".into(),
    ];
    for file in config_files {
        docker_args.push("-f".into());
        docker_args.push(file);
    }
    docker_args.push("logs".into());
    docker_args.push("-f".into());
    docker_args.push("--tail".into());
    docker_args.push(tail.to_string());

    state.log_streams.start(app, distro, docker_args).await
}
