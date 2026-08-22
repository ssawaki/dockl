use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::state::AppState;

/// Picks the shell to attach with when the caller doesn't name one explicitly.
///
/// Docker has no notion of a container's "default shell" to ask for — `docker exec` needs
/// a command — so this resolves the closest real equivalent at runtime: the exec user's
/// own login shell out of `/etc/passwd`, falling back to bash and then sh. Hardcoding
/// `sh` (the previous behavior) meant landing in dash/ash even in images that ship bash,
/// losing history, completion and line editing for no reason.
///
/// Details that matter:
/// - Shells are probed with `[ -x ]` and `exec`'d only once found. `exec a || exec b`
///   doesn't work as a fallback chain: a failed `exec` terminates the shell outright
///   rather than moving on to the right-hand side.
/// - `nologin`/`false` login shells are skipped. Service images routinely run as a user
///   whose passwd entry points at one, and exec'ing it would exit instantly.
/// - `getent` missing (distroless-ish images) just yields an empty first candidate, which
///   fails the `-x` test like any other miss.
const DEFAULT_SHELL_PROBE: &str = r#"for s in "$(getent passwd "$(id -u)" 2>/dev/null | cut -d: -f7)" /bin/bash /bin/sh; do case "$s" in ""|*nologin|*false) continue;; esac; [ -x "$s" ] && exec "$s"; done; exec sh"#;

/// Attaches an interactive shell inside a running container through a direct `--exec`
/// invocation of the Docker CLI resolved for the distro.
///
/// `--exec` rather than plain `--`: without it `wsl.exe` reconstructs the argv into
/// a single command line and hands it to the distro's *default* shell to parse, which
/// would mangle `DEFAULT_SHELL_PROBE`'s quoting before docker ever sees it. Measured, not
/// assumed: through plain `--` the probe returns the fallback instead of the real login
/// shell, because the outer zsh eats the `$(...)` first.
///
/// The CLI path is resolved once through the login shell before this point, preserving
/// PATH entries from shell startup without exposing any Docker argument to that shell.
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
    let docker_path = crate::wsl::docker_path(&distro).await?;

    let mut args = vec![
        "-d".to_string(),
        distro,
        "--exec".to_string(),
        docker_path,
        "exec".to_string(),
        "-it".to_string(),
        container_id,
    ];
    // An explicitly requested shell is run as-is — the probe is only for "whatever this
    // container considers normal".
    match shell {
        Some(shell) => args.push(shell),
        None => {
            args.push("sh".to_string());
            args.push("-c".to_string());
            args.push(DEFAULT_SHELL_PROBE.to_string());
        }
    }

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
pub fn pty_write(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> Result<(), AppError> {
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
