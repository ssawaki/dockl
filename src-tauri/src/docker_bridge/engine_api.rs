use std::collections::HashMap;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use bollard::container::{BlkioStats, CPUStats, MemoryStats, MemoryStatsStats, NetworkStats, PidsStats};
use bollard::models;
use bytes::Bytes;
use hyper::Method;
use serde::de::DeserializeOwned;

use crate::error::AppError;

use super::connection::DockerConnection;
use super::dial_stdio::DialStdioConnection;
use super::types::{
    cpu_limit_cores, restart_policy_label, sort_port_forwards, ContainerActionKind, ContainerDetail,
    ContainerSummary, DiskUsageEntry, ImageSummary, InspectHostConfig, InspectRestartPolicy,
    MountInfo, NetworkSummary, PortForward, VolumeSummary,
};

/// How an `EngineApiConnection` reaches the Docker Engine API. Both speak the exact same
/// HTTP API and share every response-shaping path below; only the bytes' route differs.
enum Transport {
    /// "TCP接続": the loopback TCP port the user exposed themselves (see Settings' 接続
    /// section and `TcpBridgeSetupDialog`).
    ///
    /// Uses a shared `reqwest::Client` rather than `bollard::Docker` (despite this module
    /// still using `bollard::models::*` for response shapes — those are just data, reused
    /// rather than redefined). `bollard`'s own `Docker::connect_with_http` hardcodes
    /// `pool_max_idle_per_host(0)` on its internal hyper client, meaning it opens a brand
    /// new TCP connection for *every single request* with nothing kept alive to reuse;
    /// across the WSL2 boundary that per-request connection setup dominated real-world
    /// latency (confirmed: parallelizing `list_images`'s two independent calls with
    /// `bollard::Docker` roughly halved its time, exactly as expected if each was
    /// separately paying that cost). `reqwest::Client` pools and reuses connections by
    /// default, so only the first call after (re)connecting pays it.
    Tcp { http: reqwest::Client, port: u16 },
    /// "dial-stdio": HTTP over a relay child process, with no port involved at all. See
    /// `DialStdioConnection` for why this is preferred over `Tcp`.
    DialStdio(DialStdioConnection),
}

/// Connection modes that drive the Docker Engine API over HTTP, as opposed to
/// `ShellOutConnection`'s one-`wsl.exe`-process-per-call approach.
///
/// Measured on the dev machine: both transports cost ~87ms per `/containers/json` call
/// (indistinguishable — the time is Docker's own work, not the transport), against
/// ~770ms for the same data via shell-out, which pays a process spawn every time. So the
/// choice between `Tcp` and `DialStdio` is purely about safety, not speed.
pub struct EngineApiConnection {
    transport: Transport,
}

impl EngineApiConnection {
    pub fn tcp(port: u16) -> Result<Self, AppError> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| AppError::CommandFailed(e.to_string()))?;
        Ok(Self { transport: Transport::Tcp { http, port } })
    }

    pub fn dial_stdio(distro: String) -> Self {
        Self { transport: Transport::DialStdio(DialStdioConnection::new(distro)) }
    }

    /// Performs one request and returns the raw response body, turning a non-2xx response
    /// into an `AppError` that carries the `{"message": "..."}` Docker's own API errors
    /// are shaped as, rather than the raw JSON.
    ///
    /// `limit` is not optional because an unbounded call here wedges the whole app: the
    /// dial-stdio transport serializes requests behind one connection, so a call that
    /// never returns holds every later one behind it. And `wsl.exe` genuinely never
    /// returns when WSL's interop layer is stuck — measured, it ran past 60s rather than
    /// failing — so "the process will error out eventually" is not a safe assumption.
    async fn request(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        limit: Duration,
    ) -> Result<Bytes, AppError> {
        match tokio::time::timeout(limit, self.send(method, path, query)).await {
            Ok(result) => result,
            Err(_) => Err(AppError::ConnectTimeout(limit.as_secs())),
        }
    }

    async fn send(&self, method: Method, path: &str, query: &[(&str, String)]) -> Result<Bytes, AppError> {
        match &self.transport {
            Transport::DialStdio(conn) => conn.request(method, path, query).await,
            Transport::Tcp { http, port } => {
                let url = format!("http://127.0.0.1:{port}{path}");
                let resp = http
                    .request(method, url)
                    .query(query)
                    .send()
                    .await
                    .map_err(|e| AppError::CommandFailed(e.to_string()))?;
                let status = resp.status();
                let body = resp.bytes().await.map_err(|e| AppError::CommandFailed(e.to_string()))?;
                if status.is_success() {
                    return Ok(body);
                }
                let text = String::from_utf8_lossy(&body).to_string();
                let message = serde_json::from_str::<serde_json::Value>(&text)
                    .ok()
                    .and_then(|v| v.get("message")?.as_str().map(str::to_string))
                    .unwrap_or(text);
                Err(AppError::CommandFailed(format!("{status}: {message}")))
            }
        }
    }

    /// For the calls the UI issues by itself — list refreshes, inspect on selection,
    /// stats polling. These are sub-second against a healthy daemon, so anything near
    /// this ceiling means WSL has stopped answering, and the user is better served by an
    /// error they can retry than by a window that stops repainting.
    const QUICK: Duration = Duration::from_secs(10);

    /// For work that legitimately runs long and that the user explicitly asked for: a
    /// prune sweeping a large system, or a stop/remove giving the container its full
    /// SIGTERM grace period. Still bounded — just far enough out not to cut short
    /// something that was going to succeed.
    const PATIENT: Duration = Duration::from_secs(180);

    async fn get_json<T: DeserializeOwned>(&self, path: &str, query: &[(&str, String)]) -> Result<T, AppError> {
        self.get_json_within(path, query, Self::QUICK).await
    }

    async fn get_json_within<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
        limit: Duration,
    ) -> Result<T, AppError> {
        let body = self.request(Method::GET, path, query, limit).await?;
        serde_json::from_slice(&body).map_err(|e| AppError::ParseError(e.to_string()))
    }

    async fn post_json<T: DeserializeOwned>(&self, path: &str, query: &[(&str, String)]) -> Result<T, AppError> {
        let body = self.request(Method::POST, path, query, Self::PATIENT).await?;
        serde_json::from_slice(&body).map_err(|e| AppError::ParseError(e.to_string()))
    }

    async fn post_empty(&self, path: &str) -> Result<(), AppError> {
        self.request(Method::POST, path, &[], Self::PATIENT).await?;
        Ok(())
    }

    async fn delete(&self, path: &str, query: &[(&str, String)]) -> Result<(), AppError> {
        self.request(Method::DELETE, path, query, Self::PATIENT).await?;
        Ok(())
    }
}

#[cfg(test)]
mod dial_stdio_tests {
    use super::*;

    /// Both connection modes must describe the same images the same way.
    ///
    /// They read the same daemon through different shapes — `docker images` prints one
    /// row per `repository:tag`, while the Engine API returns one object per image with a
    /// `RepoTags` array — and the two used to disagree: the API path kept only the first
    /// name, so an image with several tags lost all but one, and switching modes silently
    /// changed what the list showed.
    ///
    /// `#[ignore]`d like the rest; needs a live daemon. To exercise the multi-tag case,
    /// give an image a second name first (`docker tag alpine:latest dockl-demo:v1`).
    #[tokio::test]
    #[ignore]
    async fn both_modes_report_the_same_image_names() {
        let distro = std::env::var("DOCKL_TEST_DISTRO").unwrap_or_else(|_| "Ubuntu".to_string());
        let via_api = EngineApiConnection::dial_stdio(distro.clone());
        let via_cli = super::super::shell_out::ShellOutConnection::new(distro);

        let mut api = via_api.list_images().await.expect("engine api list_images failed");
        let mut cli = via_cli.list_images().await.expect("shell-out list_images failed");
        api.sort_by(|a, b| a.id.cmp(&b.id));
        cli.sort_by(|a, b| a.id.cmp(&b.id));

        // Compared only across images both report. The two disagree about *which*
        // untagged images to list at all — `/images/json` returns dangling ones that
        // `docker images` hides by default on Docker 29 (they still show under
        // `--filter dangling=true`, so the daemon does consider them dangling). That
        // difference predates this test and is a question of which set to show, not of
        // how to describe a given image, which is what's being pinned down here.
        let cli_by_id: HashMap<_, _> = cli.iter().map(|i| (i.id.clone(), i.tags.clone())).collect();
        let mut compared = 0;
        for image in &api {
            let Some(cli_tags) = cli_by_id.get(&image.id) else { continue };
            assert_eq!(
                &image.tags, cli_tags,
                "the two connection modes disagree about the names of image {}",
                image.id,
            );
            compared += 1;
        }

        assert!(compared > 0, "no images in common — nothing was actually compared");
        assert!(
            api.iter().any(|i| i.tags.len() > 1),
            "no multi-tag image present — tag one first, or this proves nothing",
        );
    }

    /// A distro that can't be reached must fail within the ceiling rather than hanging.
    ///
    /// This is not hypothetical: `wsl.exe` does not give up on its own when WSL's interop
    /// layer wedges — measured, `wsl.exe --exec docker system dial-stdio` ran past 60s
    /// without returning. Without a bound here that call never completes, and because the
    /// dial-stdio transport serializes requests behind one connection, every later call
    /// queues behind it and the whole app stops responding.
    ///
    /// Uses a nonexistent distro so it's deterministic; `#[ignore]`d like the rest since
    /// it still shells out to WSL.
    #[tokio::test]
    #[ignore]
    async fn unreachable_distro_fails_within_the_ceiling() {
        let conn = EngineApiConnection::dial_stdio("dockl-no-such-distro".to_string());
        let started = std::time::Instant::now();
        let result = conn.list_containers(true).await;
        let elapsed = started.elapsed();

        assert!(result.is_err(), "an unreachable distro should not list containers");
        assert!(
            elapsed < EngineApiConnection::QUICK + Duration::from_secs(5),
            "took {elapsed:?}, which is past the {:?} ceiling — the call is effectively unbounded",
            EngineApiConnection::QUICK,
        );
    }

    /// Drives the full `DockerConnection` surface over the dial-stdio transport — not
    /// just the raw HTTP round trip that `dial_stdio.rs`'s own test covers, but the
    /// response parsing and shaping layered on top of it, which is where a transport
    /// difference would otherwise surface as malformed rows in the UI.
    ///
    /// `#[ignore]`d for the same reason as that test: it needs a live daemon. Run with
    ///   `cargo test engine_api -- --ignored --nocapture`
    #[tokio::test]
    #[ignore]
    async fn drives_docker_connection_over_dial_stdio() {
        let distro = std::env::var("DOCKL_TEST_DISTRO").unwrap_or_else(|_| "Ubuntu".to_string());
        let conn = EngineApiConnection::dial_stdio(distro);

        let containers = conn.list_containers(true).await.expect("list_containers failed");
        // `list_images` is the interesting one: it fires two requests concurrently with
        // `try_join!`, which on this transport means both contend for the single relay.
        let images = conn.list_images().await.expect("list_images failed");
        conn.list_volumes().await.expect("list_volumes failed");
        conn.list_networks().await.expect("list_networks failed");
        conn.system_df().await.expect("system_df failed");
        assert!(conn.host_cpu_count().await.expect("host_cpu_count failed") >= 1);

        // IDs must come back in Docker's 12-char short form, matching what the shell-out
        // mode's `docker ps`/`docker images` already print.
        for c in &containers {
            assert_eq!(c.id.len(), 12, "container id not shortened: {}", c.id);
        }
        for i in &images {
            assert_eq!(i.id.len(), 12, "image id not shortened: {}", i.id);
        }

        if let Some(with_ports) = containers.iter().find(|c| !c.ports.is_empty()) {
            // The Engine API hands `NetworkSettings.Ports` back as a JSON object, which
            // deserializes into a `HashMap` — and iterating one of those yields a
            // different order per instance, so repeated inspects reshuffled the detail
            // panel's Port Forwards table under the user.
            let first = conn.inspect_container(&with_ports.id).await.expect("inspect failed");
            for attempt in 0..5 {
                let again = conn.inspect_container(&with_ports.id).await.expect("inspect failed");
                let order = |d: &ContainerDetail| {
                    d.ports
                        .iter()
                        .map(|p| format!("{}:{}/{}", p.host_ip, p.host_port, p.protocol))
                        .collect::<Vec<_>>()
                };
                assert_eq!(order(&first), order(&again), "port order changed on attempt {attempt}");
            }
        }

        if let Some(running) = containers.iter().find(|c| c.state == "running") {
            conn.inspect_container(&running.id).await.expect("inspect failed");
            let stats = conn.container_stats(&running.id).await.expect("stats failed");
            assert!(stats.contains("CPUPerc"), "stats not in docker stats shape: {stats}");
            conn.container_disk_usage(&running.id).await.expect("disk usage failed");
        }
    }
}

fn filters_query(filters: &HashMap<&str, Vec<&str>>) -> Vec<(&'static str, String)> {
    if filters.is_empty() {
        Vec::new()
    } else {
        vec![("filters", serde_json::to_string(filters).unwrap_or_default())]
    }
}

/// Docker's own short-ID convention: the first 12 characters of the full ID (which the
/// Engine API always returns in full, e.g. `sha256:<64 hex chars>` for images). Applied
/// so the TCP mode's display matches `ShellOutConnection`'s (`docker ps`/`docker images`
/// already print the 12-char short form) — the Engine API accepts either form back for
/// any operation that takes an ID, so nothing is lost by truncating for display.
fn short_id(id: &str) -> String {
    id.strip_prefix("sha256:").unwrap_or(id).chars().take(12).collect()
}

/// Matches `docker system df`'s own decimal (1000-based) size formatting closely enough
/// for display — see `src/lib/dockerStats.ts`'s `parseSize`, which the frontend already
/// uses to parse exactly this convention (kB/MB/GB, not KiB/MiB/GiB).
fn format_bytes(bytes: i64) -> String {
    const UNITS: [&str; 5] = ["B", "kB", "MB", "GB", "TB"];
    let bytes = bytes.max(0) as f64;
    if bytes < 1000.0 {
        return format!("{bytes:.0}B");
    }
    let exp = ((bytes.ln() / 1000f64.ln()).floor() as usize).min(UNITS.len() - 1);
    let value = bytes / 1000f64.powi(exp as i32);
    format!("{value:.2}{}", UNITS[exp])
}

/// Approximates Go's `units.HumanDuration` (what `docker images`'s `CreatedSince` column
/// uses) — the raw Engine API only gives a Unix timestamp, not this string, so it's
/// computed here to match `ShellOutConnection`'s existing display convention.
fn format_relative_time(unix_secs: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let elapsed = (now - unix_secs).max(0);

    let (value, unit) = if elapsed < 60 {
        (elapsed.max(1), "second")
    } else if elapsed < 3600 {
        (elapsed / 60, "minute")
    } else if elapsed < 86400 {
        (elapsed / 3600, "hour")
    } else if elapsed < 604_800 {
        (elapsed / 86400, "day")
    } else if elapsed < 2_592_000 {
        (elapsed / 604_800, "week")
    } else if elapsed < 31_536_000 {
        (elapsed / 2_592_000, "month")
    } else {
        (elapsed / 31_536_000, "year")
    };
    format!("{value} {unit}{} ago", if value == 1 { "" } else { "s" })
}

/// Same convention as `format_bytes` but binary (1024-based) — matches go-units'
/// `BytesSize`, which is what `docker stats`'s MEM USAGE column itself uses (unlike its
/// NET I/O / BLOCK I/O columns, which are decimal — see `format_bytes`'s own doc comment).
fn format_bytes_binary(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let bytes = bytes as f64;
    if bytes < 1024.0 {
        return format!("{bytes:.0}B");
    }
    let exp = ((bytes.ln() / 1024f64.ln()).floor() as usize).min(UNITS.len() - 1);
    let value = bytes / 1024f64.powi(exp as i32);
    format!("{value:.2}{}", UNITS[exp])
}

/// Minimal mirror of the `/containers/{id}/stats` response, reusing `bollard::container`'s
/// field-level types (not its top-level `Stats`, whose `read`/`preread` timestamps require
/// enabling bollard's `chrono`/`time` feature — fields this doesn't need anyway).
#[derive(serde::Deserialize)]
struct RawStats {
    cpu_stats: CPUStats,
    precpu_stats: CPUStats,
    memory_stats: MemoryStats,
    networks: Option<HashMap<String, NetworkStats>>,
    blkio_stats: BlkioStats,
    pids_stats: PidsStats,
}

/// The exact shape `docker stats --format '{{json .}}'` prints, so `container_stats` can
/// serialize into it and both connection modes share one frontend parser
/// (`src/lib/dockerStats.ts`'s `RawDockerStats`) regardless of how each computes it.
#[derive(serde::Serialize)]
struct FormattedStats {
    #[serde(rename = "CPUPerc")]
    cpu_perc: String,
    #[serde(rename = "MemUsage")]
    mem_usage: String,
    #[serde(rename = "MemPerc")]
    mem_perc: String,
    #[serde(rename = "NetIO")]
    net_io: String,
    #[serde(rename = "BlockIO")]
    block_io: String,
    #[serde(rename = "PIDs")]
    pids: String,
}

/// Mirrors the Docker CLI's own `calculateCPUPercentUnix` (moby's
/// cli/command/container/stats_helpers.go), so TCP-mode CPU% matches what `docker stats`
/// itself would show rather than some other, differently-normalized figure.
fn cpu_percent(cpu: &CPUStats, precpu: &CPUStats) -> f64 {
    let cpu_delta = cpu.cpu_usage.total_usage as f64 - precpu.cpu_usage.total_usage as f64;
    let system_delta = cpu.system_cpu_usage.unwrap_or(0) as f64 - precpu.system_cpu_usage.unwrap_or(0) as f64;
    let online_cpus = cpu
        .online_cpus
        .filter(|&n| n > 0)
        .unwrap_or_else(|| cpu.cpu_usage.percpu_usage.as_ref().map(|v| v.len() as u64).unwrap_or(1));
    if system_delta > 0.0 && cpu_delta > 0.0 {
        (cpu_delta / system_delta) * online_cpus as f64 * 100.0
    } else {
        0.0
    }
}

/// Mirrors `calculateMemUsageUnixNoCache`: subtracts the cgroup's reclaimable page cache
/// from the raw usage figure, matching what `docker stats`'s MEM USAGE actually shows —
/// raw `usage` alone would make an idle container with a large page cache look far
/// heavier than it is.
fn mem_used_bytes(mem: &MemoryStats) -> u64 {
    let usage = mem.usage.unwrap_or(0);
    let cache = match &mem.stats {
        Some(MemoryStatsStats::V1(v1)) => v1.total_inactive_file,
        Some(MemoryStatsStats::V2(v2)) => v2.inactive_file,
        None => 0,
    };
    if cache < usage {
        usage - cache
    } else {
        usage
    }
}

/// Sums rx/tx bytes across every network interface, matching `docker stats`'s NET I/O
/// (which likewise reports one combined figure, not a per-interface breakdown).
fn network_io(networks: &Option<HashMap<String, NetworkStats>>) -> (u64, u64) {
    let Some(networks) = networks else { return (0, 0) };
    networks.values().fold((0, 0), |(rx, tx), n| (rx + n.rx_bytes, tx + n.tx_bytes))
}

/// Sums read/write bytes from the recursive blkio entries, matching `docker stats`'s
/// BLOCK I/O (`op` is Docker's own capitalization, e.g. `"Read"`/`"Write"` on Linux).
fn block_io(blkio: &BlkioStats) -> (u64, u64) {
    let Some(entries) = &blkio.io_service_bytes_recursive else { return (0, 0) };
    entries.iter().fold((0, 0), |(read, write), e| match e.op.to_lowercase().as_str() {
        "read" => (read + e.value, write),
        "write" => (read, write + e.value),
        _ => (read, write),
    })
}

fn format_ports(ports: &[models::Port]) -> String {
    ports
        .iter()
        .map(|p| {
            let proto = p.typ.map(|t| t.to_string()).unwrap_or_default();
            match (p.ip.as_deref(), p.public_port) {
                (Some(ip), Some(public)) => format!("{ip}:{public}->{}/{proto}", p.private_port),
                (None, Some(public)) => format!("{public}->{}/{proto}", p.private_port),
                _ => format!("{}/{proto}", p.private_port),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn container_summary_from_bollard(c: models::ContainerSummary) -> ContainerSummary {
    ContainerSummary {
        id: short_id(c.id.as_deref().unwrap_or_default()),
        names: c
            .names
            .unwrap_or_default()
            .into_iter()
            .map(|n| n.trim_start_matches('/').to_string())
            .collect(),
        image: c.image.unwrap_or_default(),
        state: c.state.unwrap_or_default(),
        status: c.status.unwrap_or_default(),
        ports: format_ports(&c.ports.unwrap_or_default()),
        // Collected into the summary's `BTreeMap`: the API client hands these over as a
        // `HashMap`, whose per-instance iteration order is exactly what this avoids.
        labels: c.labels.unwrap_or_default().into_iter().collect(),
    }
}

#[async_trait]
impl DockerConnection for EngineApiConnection {
    async fn list_containers(&self, all: bool) -> Result<Vec<ContainerSummary>, AppError> {
        let query = [("all", all.to_string())];
        let raw: Vec<models::ContainerSummary> = self.get_json("/containers/json", &query).await?;
        Ok(raw.into_iter().map(container_summary_from_bollard).collect())
    }

    async fn container_action(&self, id: &str, action: ContainerActionKind) -> Result<(), AppError> {
        match action {
            ContainerActionKind::Start => self.post_empty(&format!("/containers/{id}/start")).await,
            ContainerActionKind::Stop => self.post_empty(&format!("/containers/{id}/stop")).await,
            ContainerActionKind::Restart => self.post_empty(&format!("/containers/{id}/restart")).await,
            ContainerActionKind::Remove => {
                // Stop first so the container gets the same SIGTERM + grace period a plain
                // Stop would give it, rather than the bare SIGKILL `force` alone sends —
                // see the matching comment in shell_out.rs. The result is discarded on
                // purpose: an already-stopped container (which answers 304 here) or one
                // that won't stop cleanly must still be removable, and `force` below then
                // behaves exactly as it did before.
                let _ = self.post_empty(&format!("/containers/{id}/stop")).await;
                self.delete(&format!("/containers/{id}"), &[("force", "true".to_string())]).await
            }
            ContainerActionKind::Pause => self.post_empty(&format!("/containers/{id}/pause")).await,
            ContainerActionKind::Unpause => self.post_empty(&format!("/containers/{id}/unpause")).await,
        }
    }

    async fn inspect_container(&self, id: &str) -> Result<ContainerDetail, AppError> {
        let raw: models::ContainerInspectResponse = self.get_json(&format!("/containers/{id}/json"), &[]).await?;

        let state = raw.state.unwrap_or_default();
        let config = raw.config.unwrap_or_default();
        let network_settings = raw.network_settings.unwrap_or_default();
        let host_config = raw.host_config.unwrap_or_default();

        let mut ports: Vec<PortForward> = network_settings
            .ports
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(container_port_proto, bindings)| {
                let (container_port, protocol) = container_port_proto.split_once('/')?;
                let bindings = bindings?;
                Some((container_port.to_string(), protocol.to_string(), bindings))
            })
            .flat_map(|(container_port, protocol, bindings)| {
                bindings.into_iter().filter_map(move |b| {
                    Some(PortForward {
                        host_ip: b.host_ip?,
                        host_port: b.host_port?,
                        container_port: container_port.clone(),
                        protocol: protocol.clone(),
                    })
                })
            })
            .collect();
        sort_port_forwards(&mut ports);

        let mounts = raw
            .mounts
            .unwrap_or_default()
            .into_iter()
            .map(|m| MountInfo {
                mount_type: m.typ.map(|t| t.to_string()).unwrap_or_default(),
                source: m.source.unwrap_or_default(),
                destination: m.destination.unwrap_or_default(),
            })
            .collect();

        let ip_address = network_settings
            .networks
            .unwrap_or_default()
            .into_values()
            .next()
            .and_then(|n| n.ip_address)
            .filter(|ip| !ip.is_empty());

        let inspect_host_config = InspectHostConfig {
            nano_cpus: host_config.nano_cpus.unwrap_or(0),
            cpu_quota: host_config.cpu_quota.unwrap_or(0),
            cpu_period: host_config.cpu_period.unwrap_or(0),
            cpuset_cpus: host_config.cpuset_cpus.unwrap_or_default(),
            restart_policy: host_config.restart_policy.map(|p| InspectRestartPolicy {
                // bollard models the name as an enum whose `EMPTY` variant `Display`s as
                // "", which `restart_policy_label` already normalizes to "no".
                name: p.name.map(|n| n.to_string()),
                maximum_retry_count: p.maximum_retry_count,
            }),
        };

        Ok(ContainerDetail {
            id: raw.id.unwrap_or_default(),
            name: raw.name.unwrap_or_default().trim_start_matches('/').to_string(),
            image: config.image.unwrap_or_default(),
            status: state.status.map(|s| s.to_string()).unwrap_or_default(),
            health: state.health.and_then(|h| h.status).map(|s| s.to_string()),
            created: raw.created.unwrap_or_default(),
            ip_address,
            ports,
            mounts,
            labels: config.labels.unwrap_or_default().into_iter().collect(),
            cpu_limit_cores: cpu_limit_cores(&inspect_host_config),
            restart_policy: restart_policy_label(&inspect_host_config),
        })
    }

    async fn list_images(&self) -> Result<Vec<ImageSummary>, AppError> {
        // Plain `list_images` doesn't return a usable per-image container count (the
        // Engine API sets it to `-1`, "not calculated", outside `/system/df`), so it's
        // cross-referenced here against the container list instead, matching what
        // `docker images`'s own `Containers` column shows. Still run concurrently (see
        // this struct's doc comment for why that matters) even though `reqwest`'s
        // pooling makes a second sequential call much cheaper than it was with bollard's
        // own client — no reason to pay two round trips serially when they're independent.
        let all_containers_query = [("all", "true".to_string())];
        let (images, containers): (Vec<models::ImageSummary>, Vec<models::ContainerSummary>) = tokio::try_join!(
            self.get_json("/images/json", &[]),
            self.get_json("/containers/json", &all_containers_query),
        )?;

        let mut counts: HashMap<String, u32> = HashMap::new();
        for c in &containers {
            if let Some(image_id) = &c.image_id {
                *counts.entry(image_id.clone()).or_insert(0) += 1;
            }
        }

        Ok(images
            .into_iter()
            .map(|img| {
                // Every name, not just the first: `RepoTags` holds one entry per
                // `repository:tag` pointing at this image, and taking only `.first()`
                // hid the rest from the UI entirely. Sorted so the name the list shows
                // doesn't change between refreshes.
                let mut tags = img.repo_tags.clone();
                tags.sort();
                let count = counts.get(&img.id).copied().unwrap_or(0);
                ImageSummary {
                    id: short_id(&img.id),
                    tags,
                    size: format_bytes(img.size),
                    created_since: format_relative_time(img.created),
                    containers: count.to_string(),
                }
            })
            .collect())
    }

    async fn remove_image(&self, id: &str) -> Result<(), AppError> {
        self.delete(&format!("/images/{id}"), &[]).await
    }

    async fn prune_images(&self, all: bool) -> Result<String, AppError> {
        // Mirrors `ShellOutConnection::prune_images`'s scope: `dangling=true` (the
        // default) only removes untagged images; `-a`'s equivalent is `dangling=false`,
        // widening it to unused-but-tagged images too.
        let dangling = (!all).to_string();
        let filters = HashMap::from([("dangling", vec![dangling.as_str()])]);
        let resp: models::ImagePruneResponse = self.post_json("/images/prune", &filters_query(&filters)).await?;
        Ok(format!(
            "Total reclaimed space: {}",
            format_bytes(resp.space_reclaimed.unwrap_or(0))
        ))
    }

    async fn list_volumes(&self) -> Result<Vec<VolumeSummary>, AppError> {
        let resp: models::VolumeListResponse = self.get_json("/volumes", &[]).await?;

        Ok(resp
            .volumes
            .unwrap_or_default()
            .into_iter()
            .map(|v| VolumeSummary {
                name: v.name,
                driver: v.driver,
                mountpoint: v.mountpoint,
                scope: v.scope.map(|s| s.to_string()).unwrap_or_default(),
                compose_project: v.labels.get("com.docker.compose.project").cloned(),
            })
            .collect())
    }

    async fn remove_volume(&self, name: &str) -> Result<(), AppError> {
        self.delete(&format!("/volumes/{name}"), &[]).await
    }

    async fn prune_volumes(&self, all: bool) -> Result<String, AppError> {
        // `all` isn't in the Engine API's documented `filters` list for this endpoint in
        // every version, but it is accepted by modern Docker — it's the same one
        // `docker volume prune -a` sends, widening the default anonymous-only scope to
        // unused-but-named volumes too.
        let filters: HashMap<&str, Vec<&str>> =
            if all { HashMap::from([("all", vec!["true"])]) } else { HashMap::new() };
        let resp: models::VolumePruneResponse = self.post_json("/volumes/prune", &filters_query(&filters)).await?;
        Ok(format!(
            "Total reclaimed space: {}",
            format_bytes(resp.space_reclaimed.unwrap_or(0))
        ))
    }

    async fn list_networks(&self) -> Result<Vec<NetworkSummary>, AppError> {
        let networks: Vec<models::Network> = self.get_json("/networks", &[]).await?;

        Ok(networks
            .into_iter()
            .map(|n| NetworkSummary {
                id: short_id(&n.id.unwrap_or_default()),
                name: n.name.unwrap_or_default(),
                driver: n.driver.unwrap_or_default(),
                scope: n.scope.unwrap_or_default(),
                internal: n.internal.unwrap_or(false),
                compose_project: n
                    .labels
                    .unwrap_or_default()
                    .get("com.docker.compose.project")
                    .cloned(),
            })
            .collect())
    }

    async fn remove_network(&self, id: &str) -> Result<(), AppError> {
        self.delete(&format!("/networks/{id}"), &[]).await
    }

    async fn prune_networks(&self) -> Result<String, AppError> {
        let resp: models::NetworkPruneResponse = self.post_json("/networks/prune", &[]).await?;
        Ok(format!(
            "Deleted Networks: {}",
            resp.networks_deleted.unwrap_or_default().len()
        ))
    }

    async fn system_df(&self) -> Result<Vec<DiskUsageEntry>, AppError> {
        // `PATIENT` rather than `QUICK`: the daemon walks every image layer and volume to
        // answer this, which is slow enough on a busy system to trip a ten-second ceiling.
        let df: models::SystemDataUsageResponse =
            self.get_json_within("/system/df", &[], Self::PATIENT).await?;

        let images = df.images.unwrap_or_default();
        let images_entry = DiskUsageEntry {
            kind: "Images".to_string(),
            total_count: images.len().to_string(),
            active: images.iter().filter(|i| i.containers > 0).count().to_string(),
            size: format_bytes(images.iter().map(|i| i.size).sum()),
            reclaimable: format_bytes(images.iter().filter(|i| i.containers == 0).map(|i| i.size).sum()),
        };

        let containers = df.containers.unwrap_or_default();
        let is_running = |c: &&models::ContainerSummary| c.state.as_deref() == Some("running");
        let containers_entry = DiskUsageEntry {
            kind: "Containers".to_string(),
            total_count: containers.len().to_string(),
            active: containers.iter().filter(is_running).count().to_string(),
            size: format_bytes(containers.iter().map(|c| c.size_rw.unwrap_or(0)).sum()),
            reclaimable: format_bytes(
                containers
                    .iter()
                    .filter(|c| !is_running(c))
                    .map(|c| c.size_rw.unwrap_or(0))
                    .sum(),
            ),
        };

        let volumes = df.volumes.unwrap_or_default();
        let vol_size = |v: &models::Volume| v.usage_data.as_ref().map(|u| u.size).unwrap_or(0);
        let vol_refs = |v: &models::Volume| v.usage_data.as_ref().map(|u| u.ref_count).unwrap_or(0);
        let volumes_entry = DiskUsageEntry {
            kind: "Local Volumes".to_string(),
            total_count: volumes.len().to_string(),
            active: volumes.iter().filter(|v| vol_refs(v) > 0).count().to_string(),
            size: format_bytes(volumes.iter().map(vol_size).sum()),
            reclaimable: format_bytes(volumes.iter().filter(|v| vol_refs(v) == 0).map(vol_size).sum()),
        };

        let build_cache = df.build_cache.unwrap_or_default();
        let build_cache_entry = DiskUsageEntry {
            kind: "Build Cache".to_string(),
            total_count: build_cache.len().to_string(),
            active: build_cache.iter().filter(|b| b.in_use.unwrap_or(false)).count().to_string(),
            size: format_bytes(build_cache.iter().map(|b| b.size.unwrap_or(0)).sum()),
            reclaimable: format_bytes(
                build_cache
                    .iter()
                    .filter(|b| !b.in_use.unwrap_or(false))
                    .map(|b| b.size.unwrap_or(0))
                    .sum(),
            ),
        };

        Ok(vec![images_entry, containers_entry, volumes_entry, build_cache_entry])
    }

    /// Build cache pruning has no `bollard` wrapper at all — it isn't part of
    /// `container.rs`/`image.rs`/`system.rs`. The Engine API does expose it directly
    /// though (`POST /build/prune`, the same one `docker builder prune` calls when not
    /// using a BuildKit builder), so this is just another call through the same shared,
    /// pooled `http` client as everything else here.
    async fn prune_build_cache(&self, all: bool) -> Result<String, AppError> {
        let resp: models::BuildPruneResponse =
            self.post_json("/build/prune", &[("all", all.to_string())]).await?;
        Ok(format!(
            "Total reclaimed space: {}",
            format_bytes(resp.space_reclaimed.unwrap_or(0))
        ))
    }

    // `stream=false` without `one-shot` matches `docker stats --no-stream`'s own
    // behavior: the daemon waits briefly between its `precpu_stats`/`cpu_stats` samples
    // so the CPU% delta is meaningful, then returns both in one response — the same
    // ~1s latency `docker stats --no-stream` already has today via shell-out, just
    // without spawning a `wsl.exe` process to get it.
    async fn container_stats(&self, id: &str) -> Result<String, AppError> {
        let query = [("stream", "false".to_string())];
        let raw: RawStats = self.get_json(&format!("/containers/{id}/stats"), &query).await?;

        let cpu_perc = cpu_percent(&raw.cpu_stats, &raw.precpu_stats);
        let mem_used = mem_used_bytes(&raw.memory_stats);
        let mem_limit = raw.memory_stats.limit.unwrap_or(0);
        let mem_perc = if mem_limit > 0 { mem_used as f64 / mem_limit as f64 * 100.0 } else { 0.0 };
        let (rx, tx) = network_io(&raw.networks);
        let (block_read, block_write) = block_io(&raw.blkio_stats);

        let formatted = FormattedStats {
            cpu_perc: format!("{cpu_perc:.2}%"),
            mem_usage: format!("{} / {}", format_bytes_binary(mem_used), format_bytes_binary(mem_limit)),
            mem_perc: format!("{mem_perc:.2}%"),
            net_io: format!("{} / {}", format_bytes(rx as i64), format_bytes(tx as i64)),
            block_io: format!("{} / {}", format_bytes(block_read as i64), format_bytes(block_write as i64)),
            pids: raw.pids_stats.current.unwrap_or(0).to_string(),
        };
        serde_json::to_string(&formatted).map_err(|e| AppError::ParseError(e.to_string()))
    }

    async fn host_cpu_count(&self) -> Result<u32, AppError> {
        let info: models::SystemInfo = self.get_json("/info", &[]).await?;
        Ok(info.ncpu.filter(|&n| n > 0).unwrap_or(1) as u32)
    }

    async fn container_disk_usage(&self, id: &str) -> Result<String, AppError> {
        let filters = HashMap::from([("id", vec![id])]);
        let mut query = filters_query(&filters);
        query.push(("all", "true".to_string()));
        query.push(("size", "true".to_string()));
        // `size=true` makes the daemon compute the container's writable-layer size, which
        // is markedly slower than a plain list — same reason `system_df` waits longer.
        let containers: Vec<models::ContainerSummary> =
            self.get_json_within("/containers/json", &query, Self::PATIENT).await?;
        let c = containers
            .into_iter()
            .next()
            .ok_or_else(|| AppError::CommandFailed("container not found".to_string()))?;
        Ok(format!(
            "{} (virtual {})",
            format_bytes(c.size_rw.unwrap_or(0)),
            format_bytes(c.size_root_fs.unwrap_or(0))
        ))
    }
}
