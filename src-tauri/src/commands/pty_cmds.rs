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

/// Attaches an interactive shell inside a running container, via
/// `wsl.exe -d <distro> --exec docker exec -it <id> <shell>`.
///
/// `--exec` rather than plain `--`: without it `wsl.exe` reconstructs the argv into
/// a single command line and hands it to the distro's *default* shell to parse, which
/// would mangle `DEFAULT_SHELL_PROBE`'s quoting before docker ever sees it. Measured, not
/// assumed: through plain `--` the probe returns the fallback instead of the real login
/// shell, because the outer zsh eats the `$(...)` first.
///
/// The tradeoff is that `--exec` skips the login shell, so `docker` must be on WSL's
/// default PATH (`/usr/bin/docker` for the apt-installed engine this app targets — see
/// PLAN.md). Anything reachable *only* via a PATH entry added by the user's shell rc —
/// notably Docker Desktop's `/mnt/c/Program Files/Docker/...` CLI — would not be found
/// here even though `wsl::run_docker`'s plain `--` finds it. That's why this asymmetry is
/// confined to this one call: `run_docker` passes only metacharacter-free arguments, so
/// it has nothing to gain from `--exec` and keeps the broader PATH.
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

    let mut args = vec![
        "-d".to_string(),
        distro,
        "--exec".to_string(),
        "docker".to_string(),
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
