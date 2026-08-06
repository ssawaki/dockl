use std::sync::Arc;

use bollard::{Docker, API_DEFAULT_VERSION};
use tauri::{AppHandle, State};

use crate::docker_bridge::shell_out::ShellOutConnection;
use crate::docker_bridge::{DockerConnection, EngineApiConnection};
use crate::error::AppError;
use crate::state::AppState;
use crate::wsl::{self, DistroInfo};

#[tauri::command]
pub async fn setup_list_distros() -> Result<Vec<DistroInfo>, AppError> {
    wsl::list_distros().await
}

/// Connects using the `ShellOut` mode against the given distro. The connection is
/// verified with a real `docker ps` call before being committed to app state, so a
/// distro without a reachable Docker daemon surfaces as an error immediately rather
/// than failing later on the first container-list refresh.
///
/// Also (re)starts the `docker events` subscription (`DockerEventManager`) — it's a
/// no-op past the first successful connect (see its own doc comment for why it's tied
/// to *this* command rather than something the frontend triggers separately).
#[tauri::command]
pub async fn setup_connect(
    app: AppHandle,
    state: State<'_, AppState>,
    distro: String,
) -> Result<(), AppError> {
    let connection = ShellOutConnection::new(distro.clone());
    // Bounded so a wedged distro surfaces as a retryable error instead of leaving the
    // app on its "connecting" screen forever with nothing to act on.
    wsl::with_connect_timeout(&distro, connection.list_containers(true)).await?;

    *state.connection.write().await = Some(Arc::new(connection));
    *state.current_distro.write().await = Some(distro.clone());
    state.event_manager.start(app, distro).await;
    Ok(())
}

#[tauri::command]
pub async fn setup_current_distro(state: State<'_, AppState>) -> Result<Option<String>, AppError> {
    Ok(state.current_distro.read().await.clone())
}

/// Switches the app's live `DockerConnection` (container/image/volume/network
/// list/action/prune — see the `DockerConnection` trait) over to the "user_managed_tcp"
/// mode, talking to `tcp://127.0.0.1:<port>` via `bollard` instead of shelling out to
/// `wsl.exe -- docker ...` for each call.
///
/// Compose, logs streaming, stats polling, and interactive attach/exec are untouched by
/// this — they always shell out regardless of this setting (see PLAN.md's Compose
/// section, and `pty_cmds.rs`/`logs_cmds.rs`/`stats_cmds.rs`), so `current_distro` stays
/// whatever it already was; only `state.connection` is replaced here. Verified with a
/// real `list_containers` call first, matching `setup_connect`'s own pattern, so a
/// misconfigured or unreachable endpoint surfaces immediately instead of on the next
/// container-list refresh.
#[tauri::command]
pub async fn connect_tcp_bridge(state: State<'_, AppState>, port: u16) -> Result<(), AppError> {
    let connection = EngineApiConnection::tcp(port)?;
    let distro = state
        .current_distro
        .read()
        .await
        .clone()
        .unwrap_or_default();
    wsl::with_connect_timeout(&distro, connection.list_containers(true)).await?;

    *state.connection.write().await = Some(Arc::new(connection));
    Ok(())
}

/// Switches the app's live `DockerConnection` over to the "dial_stdio" mode: the same
/// Docker Engine API as `connect_tcp_bridge`, but reached through a relay child process
/// (`docker system dial-stdio`) instead of a TCP port.
///
/// Needs no setup and opens nothing, so unlike the TCP mode there's no separate
/// "check"/"register" step to run first — this either works immediately or reports why
/// it can't. Verified with a real `list_containers` call before being committed to app
/// state, matching `setup_connect`'s pattern. `current_distro` is left alone; the same
/// carve-out applies as for `connect_tcp_bridge` (Compose, logs, stats and attach keep
/// shelling out regardless of this setting).
#[tauri::command]
pub async fn connect_dial_stdio(state: State<'_, AppState>) -> Result<(), AppError> {
    let distro = state
        .current_distro
        .read()
        .await
        .clone()
        .ok_or(AppError::NotConfigured)?;

    let connection = EngineApiConnection::dial_stdio(distro.clone());
    // Especially important here: the relay is a long-lived child process, so an
    // unresponsive distro shows up as a request that simply never completes rather than
    // as a spawn failure.
    wsl::with_connect_timeout(&distro, connection.list_containers(true)).await?;

    *state.connection.write().await = Some(Arc::new(connection));
    Ok(())
}

/// Whether `wsl -l -v` currently reports the given distro as running, so the frontend can
/// tell "connecting to a live distro" from "waiting for one to boot" — they differ by an
/// order of magnitude in how long they legitimately take, and showing one message for
/// both makes a cold start look indistinguishable from a hang.
#[tauri::command]
pub async fn setup_distro_is_running(distro: String) -> Result<bool, AppError> {
    Ok(wsl::is_distro_running(&distro).await)
}

/// Verifies that a Docker daemon is reachable at `tcp://127.0.0.1:<port>` and responds
/// to a real API ping. Used by the "user_managed_tcp" connection mode's setup flow (see
/// Settings' 接続 section) to confirm the user's own `dockerd -H tcp://...` config —
/// whether hand-written or run via the "自動登録" helper — actually worked.
///
/// This is only a probe; switching the app over is `connect_tcp_bridge` above, reached
/// from the connection-mode radio in Settings. Note that the port this checks is
/// unauthenticated (see `TcpBridgeSetupDialog`'s warning and PLAN.md's security section)
/// — a successful ping here means *anyone* on this machine can reach the Engine API too.
#[tauri::command]
pub async fn check_tcp_bridge(port: u16) -> Result<(), AppError> {
    let addr = format!("tcp://127.0.0.1:{port}");
    let docker = Docker::connect_with_http(&addr, 4, API_DEFAULT_VERSION)
        .map_err(|e| AppError::CommandFailed(e.to_string()))?;
    docker.ping().await.map_err(describe_ping_error)?;
    Ok(())
}

/// `bollard`'s own error text for a refused/unreachable TCP connection is a raw
/// hyper/hyper-util message (e.g. "Error in the hyper legacy client: client error
/// (Connect)") that means nothing to someone who just clicked a "verify" button — this
/// is by far the most common case (nothing set up yet, or the setup script failed), so
/// it gets a plain-language explanation instead; anything else falls back to bollard's
/// own message as-is.
fn describe_ping_error(err: bollard::errors::Error) -> AppError {
    if let bollard::errors::Error::HyperLegacyError { err: ref hyper_err } = err {
        if hyper_err.is_connect() {
            return AppError::CommandFailed(
                "Dockerが待ち受けていません。セットアップを実行してください。".to_string(),
            );
        }
    }
    AppError::CommandFailed(err.to_string())
}
