use async_trait::async_trait;

use crate::error::AppError;

use super::types::{
    ContainerActionKind, ContainerDetail, ContainerSummary, DiskUsageEntry, ImageSummary,
    NetworkSummary, VolumeSummary,
};

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
    async fn container_action(&self, id: &str, action: ContainerActionKind)
        -> Result<(), AppError>;
    async fn inspect_container(&self, id: &str) -> Result<ContainerDetail, AppError>;
    async fn list_images(&self) -> Result<Vec<ImageSummary>, AppError>;
    async fn remove_image(&self, id: &str) -> Result<(), AppError>;
    /// Removes unused images not referenced by any container. `all: false` limits this
    /// to dangling (untagged) images, matching plain `docker image prune`; `all: true`
    /// extends it to unused-but-tagged images too (`-a`). Returns docker's own summary
    /// text (e.g. "Total reclaimed space: 1.2GB") to show the user as-is.
    async fn prune_images(&self, all: bool) -> Result<String, AppError>;
    async fn list_volumes(&self) -> Result<Vec<VolumeSummary>, AppError>;
    async fn remove_volume(&self, name: &str) -> Result<(), AppError>;
    /// Removes unused volumes not referenced by any container. `all: false` limits this
    /// to anonymous volumes, matching plain `docker volume prune`; `all: true` extends
    /// it to unused named volumes too (`-a`). Returns docker's own summary text (e.g.
    /// "Total reclaimed space: 1.2GB") to show the user as-is.
    async fn prune_volumes(&self, all: bool) -> Result<String, AppError>;
    async fn list_networks(&self) -> Result<Vec<NetworkSummary>, AppError>;
    async fn remove_network(&self, id: &str) -> Result<(), AppError>;
    /// Removes networks not used by any container, returning docker's own summary text
    /// as-is (unlike image/volume prune, there's no anonymous/named-style scope split).
    async fn prune_networks(&self) -> Result<String, AppError>;
    /// The same per-resource-type disk usage summary `docker system df` shows (Images /
    /// Containers / Local Volumes / Build Cache), for the storage overview page.
    async fn system_df(&self) -> Result<Vec<DiskUsageEntry>, AppError>;
    /// Removes build cache not associated with any image. `all: false` matches plain
    /// `docker builder prune` (dangling cache only); `all: true` extends it to cache
    /// Docker would otherwise keep around to speed up future builds (`-a`).
    async fn prune_build_cache(&self, all: bool) -> Result<String, AppError>;
    /// One `docker stats --no-stream --format '{{json .}}'`-shaped JSON line (see
    /// `src/lib/dockerStats.ts`'s `RawDockerStats`) — kept as this exact wire format so
    /// both connection modes share one frontend parser regardless of how each computes it.
    async fn container_stats(&self, id: &str) -> Result<String, AppError>;
    /// How many CPUs the Docker daemon itself sees (`docker info`'s `NCPU`), used as the
    /// frontend's fallback "MAX %" ceiling for containers with no `--cpus` limit.
    async fn host_cpu_count(&self) -> Result<u32, AppError>;
    /// The container's disk usage formatted exactly like `docker ps -s`'s `Size` column,
    /// e.g. `"16.4kB (virtual 146MB)"` (see `src/lib/dockerStats.ts`'s `parseDiskUsage`).
    async fn container_disk_usage(&self, id: &str) -> Result<String, AppError>;
}
