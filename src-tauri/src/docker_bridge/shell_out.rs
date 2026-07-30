use async_trait::async_trait;

use crate::error::AppError;
use crate::wsl;

use super::connection::DockerConnection;
use super::types::{
    ContainerActionKind, ContainerDetail, ContainerSummary, DockerPsRaw, InspectRaw,
};

/// "ブリッジなし" connection mode: every call shells out to `wsl.exe -- docker ...`.
/// No component is installed inside WSL2 and no port is opened. Simplest and safest
/// setup, at the cost of per-call process-spawn overhead versus `ManagedBridge`.
pub struct ShellOutConnection {
    distro: String,
}

impl ShellOutConnection {
    pub fn new(distro: impl Into<String>) -> Self {
        Self {
            distro: distro.into(),
        }
    }
}

#[async_trait]
impl DockerConnection for ShellOutConnection {
    async fn list_containers(&self, all: bool) -> Result<Vec<ContainerSummary>, AppError> {
        let mut args = vec!["ps", "--format", "{{json .}}"];
        if all {
            args.insert(1, "-a");
        }

        let output = wsl::run_docker(&self.distro, &args).await?;

        output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<DockerPsRaw>(line)
                    .map(ContainerSummary::from)
                    .map_err(|e| AppError::ParseError(e.to_string()))
            })
            .collect()
    }

    async fn container_action(
        &self,
        id: &str,
        action: ContainerActionKind,
    ) -> Result<(), AppError> {
        let args: Vec<&str> = match action {
            ContainerActionKind::Start => vec!["start", id],
            ContainerActionKind::Stop => vec!["stop", id],
            ContainerActionKind::Restart => vec!["restart", id],
            ContainerActionKind::Remove => vec!["rm", "-f", id],
            ContainerActionKind::Pause => vec!["pause", id],
            ContainerActionKind::Unpause => vec!["unpause", id],
        };

        wsl::run_docker(&self.distro, &args).await?;
        Ok(())
    }

    async fn inspect_container(&self, id: &str) -> Result<ContainerDetail, AppError> {
        let output = wsl::run_docker(&self.distro, &["inspect", id]).await?;

        let mut parsed: Vec<InspectRaw> = serde_json::from_str(&output)
            .map_err(|e| AppError::ParseError(e.to_string()))?;

        let raw = parsed
            .pop()
            .ok_or_else(|| AppError::ParseError("docker inspect returned no results".into()))?;

        Ok(ContainerDetail::from(raw))
    }
}
