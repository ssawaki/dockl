use std::sync::Arc;
use tokio::sync::RwLock;

use crate::docker_bridge::{DockerConnection, DockerEventManager, LogStreamManager};
use crate::error::AppError;
use crate::pty_session::PtySessionManager;

/// Shared application state, managed by Tauri and injected into commands via `State<...>`.
pub struct AppState {
    docker: RwLock<Option<DockerTarget>>,
    pub log_streams: LogStreamManager,
    pub pty_sessions: PtySessionManager,
    pub event_manager: DockerEventManager,
}

struct DockerTarget {
    connection: Arc<dyn DockerConnection>,
    distro: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            docker: RwLock::new(None),
            log_streams: LogStreamManager::new(),
            pty_sessions: PtySessionManager::new(),
            event_manager: DockerEventManager::new(),
        }
    }

    pub async fn connection(&self) -> Result<Arc<dyn DockerConnection>, AppError> {
        self.docker
            .read()
            .await
            .as_ref()
            .map(|target| target.connection.clone())
            .ok_or(AppError::NotConfigured)
    }

    pub async fn distro(&self) -> Result<String, AppError> {
        self.docker
            .read()
            .await
            .as_ref()
            .map(|target| target.distro.clone())
            .ok_or(AppError::NotConfigured)
    }

    pub async fn current_distro(&self) -> Option<String> {
        self.docker
            .read()
            .await
            .as_ref()
            .map(|target| target.distro.clone())
    }

    pub async fn set_target(&self, connection: Arc<dyn DockerConnection>, distro: String) {
        *self.docker.write().await = Some(DockerTarget { connection, distro });
    }

    pub async fn replace_connection(
        &self,
        connection: Arc<dyn DockerConnection>,
    ) -> Result<(), AppError> {
        let mut target = self.docker.write().await;
        let target = target.as_mut().ok_or(AppError::NotConfigured)?;
        target.connection = connection;
        Ok(())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
