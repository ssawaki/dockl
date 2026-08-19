use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// The event this app actually cares about, normalized from `docker events`'s raw JSON
/// (`{"Type":"container","Action":"start","Actor":{"ID":"...",...},...}`) down to just
/// enough for a listener to decide "does my current view need a refresh?" — nothing
/// here parses `Actor.Attributes`; no current listener needs more than kind/action/id.
#[derive(Debug, Clone, Serialize)]
pub struct DockerEvent {
    pub kind: String,
    pub action: String,
    pub id: String,
}

#[derive(Debug, Deserialize)]
struct RawEvent {
    #[serde(rename = "Type")]
    kind: String,
    #[serde(rename = "Action")]
    action: String,
    #[serde(rename = "Actor")]
    actor: RawActor,
}

#[derive(Debug, Deserialize)]
struct RawActor {
    #[serde(rename = "ID")]
    id: String,
}

/// Whether any listener in this app cares about this event. `docker events` emits a lot
/// that's irrelevant to "does a list need refreshing": `exec_create`/`exec_start`/
/// `exec_die` fire constantly from healthchecks, and volume `mount`/`unmount` fire on
/// every container start/stop without the volume *list* itself changing. Confirmed
/// against real `docker events --format '{{json .}}'` output (restarting one compose
/// service produced 13 lines, most of them exactly this noise) rather than guessed.
fn is_relevant(kind: &str, action: &str) -> bool {
    match kind {
        "container" => matches!(
            action,
            "start"
                | "stop"
                | "die"
                | "create"
                | "destroy"
                | "pause"
                | "unpause"
                | "rename"
                | "restart"
        ),
        "image" => true,
        "volume" | "network" => matches!(action, "create" | "destroy"),
        _ => false,
    }
}

/// Emitted once each time the distro is found stopped, so the frontend can show its
/// "WSL2 is not running" screen instead of letting every later action fail one by one.
/// The way back is not an event: the window regaining focus re-checks on its own.
const DISTRO_STOPPED_EVENT: &str = "wsl:distro-stopped";

/// Keeps a single `docker events --format '{{json .}}'` process running for the app's
/// whole connected lifetime, forwarding relevant events as the `docker:event` Tauri
/// event. Always shells out via `wsl.exe`, regardless of `ConnectionMode` — same
/// rationale as `compose::compose_action`/`LogStreamManager`: this is one long-lived
/// subscription for the app's entire session, not a repeated per-call request, so the
/// "avoid a `wsl.exe` spawn per call" motivation behind `EngineApiConnection` doesn't
/// apply — a single idle subprocess for the app's lifetime is negligible by comparison.
pub struct DockerEventManager {
    started: Mutex<bool>,
}

impl DockerEventManager {
    pub fn new() -> Self {
        Self {
            started: Mutex::new(false),
        }
    }

    /// Starts the background subscription if it isn't already running. Safe to call
    /// more than once — only the first call (per process lifetime) does anything, so
    /// callers don't need to track whether they've already started it.
    pub async fn start(&self, app: AppHandle, distro: String) {
        let mut started = self.started.lock().await;
        if *started {
            return;
        }
        *started = true;
        drop(started);

        tokio::spawn(async move {
            let mut backoff = Duration::from_secs(1);
            loop {
                // Checked before every spawn, not just the first: this loop never gives
                // up, and each `wsl.exe` into a stopped distro boots the whole WSL2 VM —
                // so without this a `wsl --shutdown` while Dockl is open was undone within
                // seconds, over and over. `wsl -l -v` is answered by the Windows-side
                // service, so asking costs nothing and starts nothing.
                if !crate::wsl::is_distro_running(&distro).await {
                    let _ = app.emit(DISTRO_STOPPED_EVENT, ());
                    // Parks until something *else* observes the distro running again,
                    // rather than asking on a timer: a stopped distro can stay stopped for
                    // days, and polling it forever is what this whole change is avoiding.
                    crate::wsl::wait_for_distro_up().await;
                    continue;
                }
                if run_once(&app, &distro).await {
                    // Exited after having connected successfully at least once — the
                    // daemon/distro is presumably fine, so retry promptly rather than
                    // applying the backoff meant for a distro/daemon that isn't up yet.
                    backoff = Duration::from_secs(1);
                } else {
                    backoff = (backoff * 2).min(Duration::from_secs(30));
                }
                tokio::time::sleep(backoff).await;
            }
        });
    }
}

impl Default for DockerEventManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Runs one `docker events` process until it exits (it shouldn't, on its own, while the
/// daemon is healthy — normal exits happen from `wsl.exe`/distro/daemon hiccups). Returns
/// whether it ever received at least one line, so the retry loop can distinguish "was
/// briefly connected, then dropped" (retry fast) from "never connected at all" (back off).
async fn run_once(app: &AppHandle, distro: &str) -> bool {
    let mut cmd = tokio::process::Command::new("wsl.exe");
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.arg("-d")
        .arg(distro)
        .arg("--")
        .arg("docker")
        .arg("events")
        .arg("--format")
        .arg("{{json .}}");
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::null());
    cmd.kill_on_drop(true);

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(_) => return false,
    };
    let Some(stdout) = child.stdout.take() else {
        return false;
    };

    let mut received_any = false;
    let mut lines = BufReader::new(stdout).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        received_any = true;
        if let Ok(raw) = serde_json::from_str::<RawEvent>(&line) {
            if is_relevant(&raw.kind, &raw.action) {
                let _ = app.emit(
                    "docker:event",
                    DockerEvent {
                        kind: raw.kind,
                        action: raw.action,
                        id: raw.actor.id,
                    },
                );
            }
        }
    }

    let _ = child.kill().await;
    received_any
}
