use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::AppError;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Tracks in-flight `docker logs -f` child processes (regardless of `ConnectionMode`,
/// this always shells out via `wsl.exe`, same rationale as `compose::compose_action`)
/// so a stream can be cancelled later and so we notice when the underlying container
/// stops producing output.
#[derive(Default)]
pub struct LogStreamManager {
    streams: Arc<Mutex<HashMap<String, Child>>>,
}

impl LogStreamManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn start(
        &self,
        app: AppHandle,
        distro: String,
        docker_args: Vec<String>,
    ) -> Result<String, AppError> {
        crate::wsl::refuse_if_stopped()?;

        let stream_id = Uuid::new_v4().to_string();

        let mut cmd = tokio::process::Command::new("wsl.exe");
        #[cfg(windows)]
        {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        cmd.arg("-d")
            .arg(&distro)
            .arg("--")
            .arg("docker")
            .args(&docker_args);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        cmd.kill_on_drop(true);

        let mut child = cmd
            .spawn()
            .map_err(|e| AppError::WslUnavailable(e.to_string()))?;

        let stdout = child.stdout.take().expect("stdout was piped");
        let stderr = child.stderr.take().expect("stderr was piped");

        let data_event = format!("logs:{stream_id}");
        spawn_line_forwarder(app.clone(), data_event.clone(), stdout);
        spawn_line_forwarder(app.clone(), data_event, stderr);

        self.streams.lock().await.insert(stream_id.clone(), child);
        self.spawn_exit_watcher(app, stream_id.clone());

        Ok(stream_id)
    }

    /// Polls (rather than `.wait()`s directly) so this doesn't fight `stop()` for
    /// exclusive access to the `Child` — both just take the map's mutex briefly.
    fn spawn_exit_watcher(&self, app: AppHandle, stream_id: String) {
        let streams = self.streams.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;
                let mut guard = streams.lock().await;
                let Some(child) = guard.get_mut(&stream_id) else {
                    break; // already removed via stop()
                };
                match child.try_wait() {
                    Ok(Some(_status)) => {
                        guard.remove(&stream_id);
                        drop(guard);
                        let _ = app.emit(&format!("logs:{stream_id}:end"), ());
                        break;
                    }
                    Ok(None) => continue,
                    Err(_) => break,
                }
            }
        });
    }

    pub async fn stop(&self, stream_id: &str) -> Result<(), AppError> {
        if let Some(mut child) = self.streams.lock().await.remove(stream_id) {
            let _ = child.kill().await;
        }
        Ok(())
    }
}

/// How long a batch waits for more lines to arrive before it's flushed, once the first
/// line in it shows up. Bounds latency for a chatty container (a burst gets coalesced
/// into one emit instead of one per line) without making a single log line feel delayed.
const BATCH_WINDOW: Duration = Duration::from_millis(15);
/// Caps a single batch/emit regardless of how long lines keep arriving within the
/// window, so an extremely chatty container can't grow one emit unboundedly.
const MAX_BATCH_LINES: usize = 500;

fn spawn_line_forwarder<R>(app: AppHandle, event: String, reader: R)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    // Split into a reader task (pushes lines into a channel as soon as each is available)
    // and a batching task (drains the channel, coalescing whatever arrived within
    // `BATCH_WINDOW` into one `app.emit` instead of one per line) — a chatty container
    // (e.g. a dev server logging every request) would otherwise cost one IPC
    // serialize/dispatch cycle per line.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if tx.send(line).is_err() {
                break; // batching task ended (e.g. app shutting down)
            }
        }
    });

    tokio::spawn(async move {
        loop {
            let Some(first) = rx.recv().await else { break };
            let mut batch = vec![first];
            let deadline = tokio::time::Instant::now() + BATCH_WINDOW;
            while batch.len() < MAX_BATCH_LINES {
                let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
                if remaining.is_zero() {
                    break;
                }
                match tokio::time::timeout(remaining, rx.recv()).await {
                    Ok(Some(line)) => batch.push(line),
                    _ => break,
                }
            }
            let _ = app.emit(&event, batch);
        }
    });
}
