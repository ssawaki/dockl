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
        container_id: String,
        tail: u32,
    ) -> Result<String, AppError> {
        let stream_id = Uuid::new_v4().to_string();

        let mut cmd = tokio::process::Command::new("wsl.exe");
        #[cfg(windows)]
        {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        cmd.args([
            "-d",
            &distro,
            "--",
            "docker",
            "logs",
            "-f",
            "--tail",
            &tail.to_string(),
            &container_id,
        ]);
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

fn spawn_line_forwarder<R>(app: AppHandle, event: String, reader: R)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    let _ = app.emit(&event, line);
                }
                _ => break,
            }
        }
    });
}
