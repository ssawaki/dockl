use async_trait::async_trait;

use crate::error::AppError;

use super::types::{ContainerActionKind, ContainerDetail, ContainerSummary};

/// Which strategy is used to talk to the Docker daemon running inside WSL2.
/// See PLAN.md "接続モード" for the rationale behind each variant.
#[allow(dead_code)] // variants are wired up incrementally across milestones, see PLAN.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionMode {
    /// No component installed in WSL2; every call shells out to `wsl.exe -- docker ...`.
    ShellOut,
    /// `dockl-bridged` proxy running in WSL2, reached over a loopback TCP port with
    /// bearer-token auth (default recommended mode).
    ManagedBridge,
    /// User already exposes the Docker Engine API over TCP themselves; we just connect.
    UserManagedTcp,
}

/// Common interface implemented by each `ConnectionMode`. Command handlers depend on
/// this trait so they don't need to know which mode is active.
#[async_trait]
pub trait DockerConnection: Send + Sync {
    async fn list_containers(&self, all: bool) -> Result<Vec<ContainerSummary>, AppError>;
    async fn container_action(
        &self,
        id: &str,
        action: ContainerActionKind,
    ) -> Result<(), AppError>;
    async fn inspect_container(&self, id: &str) -> Result<ContainerDetail, AppError>;
}
