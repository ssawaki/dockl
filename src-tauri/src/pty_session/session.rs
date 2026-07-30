use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::error::AppError;

struct PtySession {
    // `Box<dyn MasterPty + Send>` isn't `Sync`, and `AppState` (which holds
    // `PtySessionManager`) must be `Send + Sync` for Tauri's `State<T>` extractor.
    // Wrapping in `Mutex` sidesteps that: `Mutex<T>` is `Sync` as long as `T: Send`,
    // regardless of whether `T` itself is `Sync`.
    master: Mutex<Box<dyn MasterPty + Send>>,
    writer: Mutex<Box<dyn Write + Send>>,
    child: Mutex<Box<dyn Child + Send + Sync>>,
}

/// Manages interactive PTY sessions (`docker exec -it`, or a plain WSL shell) spawned
/// via `wsl.exe`. Unlike `LogStreamManager` (line-buffered, tokio-friendly), a PTY's
/// reader is a blocking `std::io::Read`, so it's driven from a `spawn_blocking` task
/// rather than a tokio-native async one.
#[derive(Default)]
pub struct PtySessionManager {
    sessions: Arc<Mutex<HashMap<String, Arc<PtySession>>>>,
}

impl PtySessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// `args` are the full argv to run under `wsl.exe` (e.g.
    /// `["-d", "Ubuntu", "--", "docker", "exec", "-it", "<id>", "sh"]`).
    pub fn start(
        &self,
        app: AppHandle,
        args: Vec<String>,
        cols: u16,
        rows: u16,
    ) -> Result<String, AppError> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AppError::CommandFailed(e.to_string()))?;

        let mut cmd = CommandBuilder::new("wsl.exe");
        cmd.args(&args);

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| AppError::CommandFailed(e.to_string()))?;
        // The slave end belongs to the child now; dropping our copy here (rather than
        // holding it in the session) matches portable-pty's own examples.
        drop(pair.slave);

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| AppError::CommandFailed(e.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| AppError::CommandFailed(e.to_string()))?;

        let session_id = Uuid::new_v4().to_string();
        let session = Arc::new(PtySession {
            master: Mutex::new(pair.master),
            writer: Mutex::new(writer),
            child: Mutex::new(child),
        });

        self.sessions
            .lock()
            .unwrap()
            .insert(session_id.clone(), session);

        self.spawn_reader(app, session_id.clone(), reader);

        Ok(session_id)
    }

    fn spawn_reader(&self, app: AppHandle, session_id: String, mut reader: Box<dyn Read + Send>) {
        let sessions = self.sessions.clone();
        tokio::task::spawn_blocking(move || {
            let data_event = format!("pty:{session_id}:data");
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = String::from_utf8_lossy(&buf[..n]).into_owned();
                        let _ = app.emit(&data_event, chunk);
                    }
                    Err(_) => break,
                }
            }
            sessions.lock().unwrap().remove(&session_id);
            let _ = app.emit(&format!("pty:{session_id}:exit"), ());
        });
    }

    pub fn write(&self, session_id: &str, data: &str) -> Result<(), AppError> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions.get(session_id).ok_or(AppError::NotConfigured)?;
        let result = session
            .writer
            .lock()
            .unwrap()
            .write_all(data.as_bytes())
            .map_err(|e| AppError::CommandFailed(e.to_string()));
        result
    }

    pub fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), AppError> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions.get(session_id).ok_or(AppError::NotConfigured)?;
        let result = session
            .master
            .lock()
            .unwrap()
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AppError::CommandFailed(e.to_string()));
        result
    }

    pub fn close(&self, session_id: &str) -> Result<(), AppError> {
        if let Some(session) = self.sessions.lock().unwrap().remove(session_id) {
            let _ = session.child.lock().unwrap().kill();
        }
        Ok(())
    }
}
