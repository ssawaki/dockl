use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use portable_pty::{native_pty_system, Child, ChildKiller, CommandBuilder, MasterPty, PtySize};
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
    // A killer split off the child rather than the child itself: the watcher task owns the
    // child for as long as it sits in `wait()`, so anything that wants to *terminate* the
    // child needs a handle that isn't behind the same lock. That's what `clone_killer` is
    // for. Holding the `Child` here instead would mean `close()` blocks on a mutex the
    // watcher only releases when the process it's waiting for is already gone.
    killer: Mutex<Box<dyn ChildKiller + Send + Sync>>,
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
            killer: Mutex::new(child.clone_killer()),
        });

        self.sessions
            .lock()
            .unwrap()
            .insert(session_id.clone(), session);

        self.spawn_reader(app, session_id.clone(), reader);
        self.spawn_child_watcher(session_id.clone(), child);

        Ok(session_id)
    }

    /// Ends the session once its process does.
    ///
    /// Needed because ConPTY doesn't close the pty's output pipe when the child exits: the
    /// pseudoconsole host owns the write end and holds it open until `ClosePseudoConsole`,
    /// which portable-pty only calls when the `MasterPty` is dropped. Left alone, the
    /// reader task below therefore never sees EOF after a shell exits — it just blocks
    /// forever, no `:exit` event is ever emitted (so the UI can't tell the session died),
    /// and the pseudoconsole plus that blocked thread stay around until the app quits.
    ///
    /// Dropping the session here is what breaks the cycle: it closes the pseudoconsole,
    /// which ends the blocking read, which lets the reader emit `:exit` and finish.
    fn spawn_child_watcher(&self, session_id: String, mut child: Box<dyn Child + Send + Sync>) {
        let sessions = self.sessions.clone();
        tokio::task::spawn_blocking(move || {
            let _ = child.wait();
            // The child being gone doesn't mean its last output has reached us: conhost
            // batches what it renders into the pipe on its own schedule, so closing the
            // pseudoconsole the instant `wait()` returns can cut off whatever it hadn't
            // flushed yet — typically the very lines someone would want to read after a
            // shell died unexpectedly. Give the reader a moment to drain first.
            std::thread::sleep(std::time::Duration::from_millis(100));
            // Taken out of the map before being dropped, so the (potentially blocking)
            // `ClosePseudoConsole` doesn't run while every other session's writes and
            // resizes are stuck behind this lock.
            let session = sessions.lock().unwrap().remove(&session_id);
            drop(session);
        });
    }

    fn spawn_reader(&self, app: AppHandle, session_id: String, mut reader: Box<dyn Read + Send>) {
        let sessions = self.sessions.clone();
        tokio::task::spawn_blocking(move || {
            let data_event = format!("pty:{session_id}:data");
            let mut buf = [0u8; 8192];
            // Carries a UTF-8 sequence that a fixed-size read split across two calls,
            // so it can be completed (or, worst case, lossily flushed) on the next one
            // instead of corrupting whichever half-character landed in this chunk.
            let mut leftover: Vec<u8> = Vec::new();
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        leftover.extend_from_slice(&buf[..n]);
                        // Pass `&str`/`Cow<str>` straight to `emit` (both are
                        // `Serialize + Clone`) rather than pre-allocating an owned
                        // `String` ourselves on top of whatever `emit` allocates
                        // internally to serialize the payload — this loop runs for
                        // every ~8KB chunk of a session that can live for a long time.
                        match std::str::from_utf8(&leftover) {
                            Ok(s) => {
                                let _ = app.emit(&data_event, s);
                                leftover.clear();
                            }
                            Err(e) => {
                                let valid_up_to = e.valid_up_to();
                                if valid_up_to > 0 {
                                    let s = std::str::from_utf8(&leftover[..valid_up_to])
                                        .expect("valid_up_to guarantees a valid prefix");
                                    let _ = app.emit(&data_event, s);
                                }
                                let remainder = leftover.split_off(valid_up_to);
                                leftover = remainder;
                                // A real UTF-8 sequence is at most 4 bytes; if we still
                                // haven't resolved it by then, it isn't a split
                                // character but genuinely invalid data — flush it
                                // lossily rather than buffering forever.
                                if leftover.len() >= 4 {
                                    let _ = app.emit(&data_event, String::from_utf8_lossy(&leftover));
                                    leftover.clear();
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            if !leftover.is_empty() {
                let _ = app.emit(&data_event, String::from_utf8_lossy(&leftover));
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
        // Bound in a `let` rather than matched inline, so the map's lock is released
        // before the kill and before this session's `MasterPty` drops at the end of the
        // block — the same reason spelled out in `spawn_child_watcher`.
        let session = self.sessions.lock().unwrap().remove(session_id);
        if let Some(session) = session {
            let _ = session.killer.lock().unwrap().kill();
        }
        Ok(())
    }
}
