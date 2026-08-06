use async_trait::async_trait;

use crate::error::AppError;
use crate::wsl;

use super::connection::DockerConnection;
use super::types::{
    fold_docker_images, ContainerActionKind, ContainerDetail, ContainerSummary, DiskUsageEntry,
    DockerDiskUsageRaw, DockerImageRaw, DockerNetworkRaw, DockerPsRaw, DockerVolumeRaw,
    ImageSummary, InspectRaw, NetworkSummary, VolumeSummary,
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
        // Removal stops the container first, so it gets the same SIGTERM + grace period a
        // plain Stop would give it, instead of the bare SIGKILL that `rm -f` on its own
        // sends. Containers that need to flush state on shutdown (databases, queues) were
        // otherwise killed mid-write purely because the user reached for the trash icon
        // rather than stop-then-delete. `docker compose down` already worked this way.
        //
        // The stop's result is deliberately discarded: an already-stopped container, or
        // one that won't stop cleanly, must still be removable. `rm -f` below then does
        // exactly what it did before, so this only ever adds a chance to exit gracefully.
        if let ContainerActionKind::Remove = action {
            let _ = wsl::run_docker(&self.distro, &["stop", id]).await;
        }

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

        let mut parsed: Vec<InspectRaw> =
            serde_json::from_str(&output).map_err(|e| AppError::ParseError(e.to_string()))?;

        let raw = parsed
            .pop()
            .ok_or_else(|| AppError::ParseError("docker inspect returned no results".into()))?;

        Ok(ContainerDetail::from(raw))
    }

    async fn list_images(&self) -> Result<Vec<ImageSummary>, AppError> {
        let output = wsl::run_docker(&self.distro, &["images", "--format", "{{json .}}"]).await?;

        let rows = output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<DockerImageRaw>(line)
                    .map_err(|e| AppError::ParseError(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // `docker images` emits one row per name, so an image with several tags would
        // otherwise be listed once per tag — see `fold_docker_images`.
        Ok(fold_docker_images(rows))
    }

    async fn remove_image(&self, id: &str) -> Result<(), AppError> {
        wsl::run_docker(&self.distro, &["rmi", id]).await?;
        Ok(())
    }

    async fn prune_images(&self, all: bool) -> Result<String, AppError> {
        // Plain `docker image prune` only removes dangling (untagged) images — not the
        // "未使用" (unused) ones this app shows as a distinct list section, which are
        // simply tagged images no container currently references. `-a` extends removal
        // to those too; the frontend lets the user opt into that scope explicitly.
        let mut args = vec!["image", "prune", "-f"];
        if all {
            args.push("-a");
        }
        wsl::run_docker(&self.distro, &args).await
    }

    async fn list_volumes(&self) -> Result<Vec<VolumeSummary>, AppError> {
        let output =
            wsl::run_docker(&self.distro, &["volume", "ls", "--format", "{{json .}}"]).await?;

        output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<DockerVolumeRaw>(line)
                    .map(VolumeSummary::from)
                    .map_err(|e| AppError::ParseError(e.to_string()))
            })
            .collect()
    }

    async fn remove_volume(&self, name: &str) -> Result<(), AppError> {
        wsl::run_docker(&self.distro, &["volume", "rm", name]).await?;
        Ok(())
    }

    async fn prune_volumes(&self, all: bool) -> Result<String, AppError> {
        // Same gotcha as `prune_images`: plain `docker volume prune` only removes
        // *anonymous* volumes, leaving named-but-unused ones (which is what this app's
        // volume list actually shows) untouched. `-a` extends removal to those too; the
        // frontend lets the user opt into that scope explicitly.
        let mut args = vec!["volume", "prune", "-f"];
        if all {
            args.push("-a");
        }
        wsl::run_docker(&self.distro, &args).await
    }

    async fn list_networks(&self) -> Result<Vec<NetworkSummary>, AppError> {
        let output =
            wsl::run_docker(&self.distro, &["network", "ls", "--format", "{{json .}}"]).await?;

        output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<DockerNetworkRaw>(line)
                    .map(NetworkSummary::from)
                    .map_err(|e| AppError::ParseError(e.to_string()))
            })
            .collect()
    }

    async fn remove_network(&self, id: &str) -> Result<(), AppError> {
        wsl::run_docker(&self.distro, &["network", "rm", id]).await?;
        Ok(())
    }

    async fn prune_networks(&self) -> Result<String, AppError> {
        wsl::run_docker(&self.distro, &["network", "prune", "-f"]).await
    }

    async fn system_df(&self) -> Result<Vec<DiskUsageEntry>, AppError> {
        let output =
            wsl::run_docker(&self.distro, &["system", "df", "--format", "{{json .}}"]).await?;

        output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<DockerDiskUsageRaw>(line)
                    .map(DiskUsageEntry::from)
                    .map_err(|e| AppError::ParseError(e.to_string()))
            })
            .collect()
    }

    async fn prune_build_cache(&self, all: bool) -> Result<String, AppError> {
        let mut args = vec!["builder", "prune", "-f"];
        if all {
            args.push("-a");
        }
        wsl::run_docker(&self.distro, &args).await
    }

    // `docker stats` in its normal continuous mode is unusable via `wsl.exe`: WSL always
    // presents a TTY to the launched Linux process regardless of how the Windows side
    // redirects stdout, so docker's live table renderer stays active and interleaves each
    // JSON line with cursor-repositioning ANSI codes — unlike `docker logs -f`, which
    // prints plain lines either way. `--no-stream` sidesteps this: it renders the table
    // exactly once (no live redraw) and exits, giving a clean single JSON line.
    async fn container_stats(&self, id: &str) -> Result<String, AppError> {
        wsl::run_docker(
            &self.distro,
            &["stats", "--no-stream", "--format", "{{json .}}", id],
        )
        .await
    }

    async fn host_cpu_count(&self) -> Result<u32, AppError> {
        // Deliberately a single `--format` template field with no delimiter between
        // multiple fields: `wsl.exe` (without `--exec`) hands the reconstructed command
        // line to the distro's default shell, so characters like `|` or `;` inside an
        // argument get reinterpreted as shell syntax instead of reaching `docker` as
        // literal text. A lone `{{.NCPU}}` has no such character, so it's safe; joining
        // several fields with a delimiter is not — see git history for the `zsh: parse
        // error` this caused.
        let output = wsl::run_docker(&self.distro, &["info", "--format", "{{.NCPU}}"]).await?;
        Ok(output.trim().parse().unwrap_or(1))
    }

    async fn container_disk_usage(&self, id: &str) -> Result<String, AppError> {
        // `docker inspect` has no equivalent field; only `docker ps -s` computes this,
        // and only for the container(s) it lists — hence the separate call and `-a`
        // (unlike `container_stats`, this works for stopped containers too).
        let filter = format!("id={id}");
        let output = wsl::run_docker(
            &self.distro,
            &[
                "ps",
                "-a",
                "-s",
                "--filter",
                &filter,
                "--format",
                "{{.Size}}",
            ],
        )
        .await?;
        Ok(output.trim().to_string())
    }
}
