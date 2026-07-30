use std::sync::Arc;
use tokio::sync::RwLock;

use crate::docker_bridge::{DockerConnection, LogStreamManager};
use crate::pty_session::PtySessionManager;

/// Shared application state, managed by Tauri and injected into commands via `State<...>`.
pub struct AppState {
    pub connection: RwLock<Option<Arc<dyn DockerConnection>>>,
    pub current_distro: RwLock<Option<String>>,
    pub log_streams: LogStreamManager,
    pub pty_sessions: PtySessionManager,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connection: RwLock::new(None),
            current_distro: RwLock::new(None),
            log_streams: LogStreamManager::new(),
            pty_sessions: PtySessionManager::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
