use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSummary {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    /// Raw Docker state, e.g. "running", "exited", "paused".
    pub state: String,
    /// Human-readable status, e.g. "Up 3 hours".
    pub status: String,
    pub ports: String,
    /// `BTreeMap`, not `HashMap`, for the same reason `sort_port_forwards` exists: a
    /// `HashMap` iterates in a different order per instance, so serializing one hands the
    /// frontend a fresh order on every call and the detail panel's Labels table visibly
    /// reshuffles between refreshes. Sorting by key is also just the useful order to read
    /// them in, since Compose's own `com.docker.compose.*` labels then group together.
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerActionKind {
    Start,
    Stop,
    Restart,
    Remove,
    Pause,
    Unpause,
}

/// Raw shape of one line from `docker ps --format '{{json .}}'`.
#[derive(Debug, Deserialize)]
pub struct DockerPsRaw {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Names")]
    pub names: String,
    #[serde(rename = "Image")]
    pub image: String,
    #[serde(rename = "State")]
    pub state: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Ports")]
    pub ports: String,
    #[serde(rename = "Labels")]
    pub labels: String,
}

impl From<DockerPsRaw> for ContainerSummary {
    fn from(raw: DockerPsRaw) -> Self {
        let labels = split_docker_labels(&raw.labels)
            .into_iter()
            .filter_map(|kv| kv.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Self {
            id: raw.id,
            names: raw.names.split(',').map(|s| s.to_string()).collect(),
            image: raw.image,
            state: raw.state,
            status: raw.status,
            ports: raw.ports,
            labels,
        }
    }
}

/// Splits `docker ps`'s `Labels` field (one string, `key1=value1,key2=value2,...`) back
/// into individual `key=value` entries.
///
/// A naive `split(',')` corrupts any label whose *value* itself contains a comma —
/// notably Compose's own `com.docker.compose.project.config_files`, which lists every
/// `-f` file joined with `,`. Docker doesn't escape commas in this field at all, so
/// there's no fully unambiguous way to reverse the join; instead, a comma only starts a
/// new entry here if what follows it actually looks like `<label-key>=`, since real
/// label keys are restricted to a known character set that plain path/URL text
/// practically never matches immediately after an internal comma.
fn split_docker_labels(raw: &str) -> Vec<&str> {
    if raw.is_empty() {
        return Vec::new();
    }

    let is_key_char = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '/');
    let looks_like_key_prefix = |s: &str| match s.find('=') {
        Some(0) => false,
        Some(eq_pos) => s[..eq_pos].chars().all(is_key_char),
        None => false,
    };

    let mut entries = Vec::new();
    let mut start = 0;
    let mut search_from = 0;
    while let Some(rel) = raw[search_from..].find(',') {
        let comma = search_from + rel;
        if looks_like_key_prefix(&raw[comma + 1..]) {
            entries.push(&raw[start..comma]);
            start = comma + 1;
        }
        search_from = comma + 1;
    }
    entries.push(&raw[start..]);
    entries.into_iter().filter(|s| !s.is_empty()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image_row(id: &str, repository: &str, tag: &str) -> DockerImageRaw {
        DockerImageRaw {
            id: id.to_string(),
            repository: repository.to_string(),
            tag: tag.to_string(),
            size: "7.8MB".to_string(),
            created_since: "2 weeks ago".to_string(),
            containers: "0".to_string(),
        }
    }

    #[test]
    fn fold_docker_images_groups_every_name_onto_one_image() {
        // `docker tag` adds an alias without copying anything, so one image really can
        // answer to several names — `docker images` then prints it once per name.
        let folded = fold_docker_images(vec![
            image_row("5b10f432ef3d", "alpine", "latest"),
            image_row("4a73073bd557", "nginx", "alpine"),
            image_row("5b10f432ef3d", "dockl-demo", "v2"),
            image_row("5b10f432ef3d", "dockl-demo", "v1"),
        ]);

        assert_eq!(folded.len(), 2, "rows for the same image should collapse into one");
        assert_eq!(folded[0].id, "5b10f432ef3d");
        assert_eq!(
            folded[0].tags,
            vec!["alpine:latest", "dockl-demo:v1", "dockl-demo:v2"],
            "every name should be kept, in a stable order",
        );
        assert_eq!(folded[1].tags, vec!["nginx:alpine"]);
    }

    #[test]
    fn fold_docker_images_keeps_a_registry_port_in_the_repository() {
        // The colon in `localhost:5000` is part of the repository, not a tag separator.
        let folded = fold_docker_images(vec![image_row("abc123", "localhost:5000/app", "v3")]);
        assert_eq!(folded[0].tags, vec!["localhost:5000/app:v3"]);
    }

    #[test]
    fn fold_docker_images_leaves_a_dangling_image_nameless() {
        // The CLI renders "no name" as `<none>`; recording that verbatim would make a
        // dangling image look like it was tagged `<none>:<none>`.
        let folded = fold_docker_images(vec![image_row("deadbeef", "<none>", "<none>")]);
        assert_eq!(folded.len(), 1);
        assert!(folded[0].tags.is_empty(), "dangling image should have no names");
    }

    /// Real (anonymized-path) label string from a container brought up with
    /// `docker compose -f docker-compose.yml -f docker-compose.mysql.yml -f
    /// docker-compose.dev.yml -f docker-compose.adminer.yml up` — this is exactly the
    /// shape that broke on the naive `split(',')`, silently dropping every config file
    /// after the first.
    #[test]
    fn split_docker_labels_preserves_comma_containing_values() {
        let raw = "com.docker.compose.project.config_files=/work/proj/docker-compose.yml,/work/proj/docker-compose.mysql.yml,/work/proj/docker-compose.dev.yml,/work/proj/docker-compose.adminer.yml,com.docker.compose.project=proj,com.docker.compose.project.working_dir=/work/proj";

        let entries = split_docker_labels(raw);
        assert_eq!(
            entries,
            vec![
                "com.docker.compose.project.config_files=/work/proj/docker-compose.yml,/work/proj/docker-compose.mysql.yml,/work/proj/docker-compose.dev.yml,/work/proj/docker-compose.adminer.yml",
                "com.docker.compose.project=proj",
                "com.docker.compose.project.working_dir=/work/proj",
            ]
        );
    }

    #[test]
    fn split_docker_labels_handles_single_label() {
        assert_eq!(
            split_docker_labels("com.docker.compose.project=proj"),
            vec!["com.docker.compose.project=proj"]
        );
    }

    #[test]
    fn split_docker_labels_handles_empty_input() {
        assert!(split_docker_labels("").is_empty());
    }

    fn host_config_with_policy(policy: Option<InspectRestartPolicy>) -> InspectHostConfig {
        InspectHostConfig {
            nano_cpus: 0,
            cpu_quota: 0,
            cpu_period: 0,
            cpuset_cpus: String::new(),
            restart_policy: policy,
        }
    }

    fn policy(name: &str, max_retry: i64) -> Option<InspectRestartPolicy> {
        Some(InspectRestartPolicy {
            name: Some(name.to_string()),
            maximum_retry_count: Some(max_retry),
        })
    }

    /// The empty `Name` is what every compose-managed container here reports (no
    /// `restart:` key in the compose file) — it must not render as a blank cell.
    #[test]
    fn restart_policy_label_normalizes_unset_policy_to_no() {
        assert_eq!(restart_policy_label(&host_config_with_policy(policy("", 0))), "no");
        assert_eq!(restart_policy_label(&host_config_with_policy(None)), "no");
        assert_eq!(
            restart_policy_label(&host_config_with_policy(Some(InspectRestartPolicy {
                name: None,
                maximum_retry_count: None,
            }))),
            "no"
        );
    }

    #[test]
    fn restart_policy_label_keeps_plain_policies_verbatim() {
        assert_eq!(restart_policy_label(&host_config_with_policy(policy("no", 0))), "no");
        assert_eq!(restart_policy_label(&host_config_with_policy(policy("always", 0))), "always");
        assert_eq!(
            restart_policy_label(&host_config_with_policy(policy("unless-stopped", 0))),
            "unless-stopped"
        );
    }

    #[test]
    fn restart_policy_label_appends_retry_cap_only_for_on_failure() {
        assert_eq!(
            restart_policy_label(&host_config_with_policy(policy("on-failure", 3))),
            "on-failure:3"
        );
        // `--restart on-failure` with no cap: retry forever, so no suffix.
        assert_eq!(
            restart_policy_label(&host_config_with_policy(policy("on-failure", 0))),
            "on-failure"
        );
        // Docker itself rejects a cap on `always`, but a stray one must not leak into
        // the label as `always:5`.
        assert_eq!(restart_policy_label(&host_config_with_policy(policy("always", 5))), "always");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSummary {
    pub id: String,
    /// Every `repository:tag` name pointing at this image, sorted, and empty for a
    /// dangling (untagged) one.
    ///
    /// One image can carry any number of names — `docker tag` adds an alias without
    /// copying anything — so this is a list rather than the single repository/tag pair it
    /// used to be. `docker images` prints one *row* per name, which made the same image
    /// appear several times; the list view shows one row per image and puts the full set
    /// here for the detail panel, so no name is hidden and none is duplicated.
    pub tags: Vec<String>,
    pub size: String,
    pub created_since: String,
    /// How many containers currently reference this image, as a decimal string (Docker
    /// CLI's own format — kept as-is rather than parsed since it's display-only).
    pub containers: String,
}

/// Raw shape of one line from `docker images --format '{{json .}}'`.
#[derive(Debug, Deserialize)]
pub struct DockerImageRaw {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Repository")]
    pub repository: String,
    #[serde(rename = "Tag")]
    pub tag: String,
    #[serde(rename = "Size")]
    pub size: String,
    #[serde(rename = "CreatedSince")]
    pub created_since: String,
    #[serde(rename = "Containers")]
    pub containers: String,
}

/// Folds `docker images` output — one row per *name* — into one entry per image.
///
/// The CLI prints `myapp:latest` and `myapp:1.2.0` as two rows when both point at the
/// same image, which listed the same thing twice. Rows are grouped by ID here so the view
/// matches the Engine API path, which returns one object per image with a `RepoTags`
/// array. Everything except the name is identical across an image's rows, so the first
/// row's values are kept.
///
/// Insertion order is preserved (`docker images` already sorts newest-first) while each
/// image's own names are sorted, so the name chosen for the list is stable.
pub(crate) fn fold_docker_images(rows: Vec<DockerImageRaw>) -> Vec<ImageSummary> {
    let mut order: Vec<String> = Vec::new();
    let mut by_id: HashMap<String, ImageSummary> = HashMap::new();

    for raw in rows {
        let entry = by_id.entry(raw.id.clone()).or_insert_with(|| {
            order.push(raw.id.clone());
            ImageSummary {
                id: raw.id.clone(),
                tags: Vec::new(),
                size: raw.size.clone(),
                created_since: raw.created_since.clone(),
                containers: raw.containers.clone(),
            }
        });
        // `<none>` is how the CLI renders "no name at all", not a name in its own right —
        // recording it would make a dangling image look tagged.
        if raw.repository != "<none>" {
            entry.tags.push(format!("{}:{}", raw.repository, raw.tag));
        }
    }

    order
        .into_iter()
        .filter_map(|id| by_id.remove(&id))
        .map(|mut image| {
            image.tags.sort();
            image
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSummary {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub scope: String,
    /// The Compose project this volume belongs to, if its labels say so.
    pub compose_project: Option<String>,
}

/// Raw shape of one line from `docker volume ls --format '{{json .}}'`.
#[derive(Debug, Deserialize)]
pub struct DockerVolumeRaw {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Driver")]
    pub driver: String,
    #[serde(rename = "Mountpoint")]
    pub mountpoint: String,
    #[serde(rename = "Scope")]
    pub scope: String,
    #[serde(rename = "Labels")]
    pub labels: String,
}

impl From<DockerVolumeRaw> for VolumeSummary {
    fn from(raw: DockerVolumeRaw) -> Self {
        // Reuses the same comma-safe parser as container labels: a volume's own labels
        // can just as easily contain comma-joined values (e.g. tooling that mirrors
        // compose's config_files convention) even though today's compose labels here
        // happen not to.
        let compose_project = split_docker_labels(&raw.labels)
            .into_iter()
            .filter_map(|kv| kv.split_once('='))
            .find(|(k, _)| *k == "com.docker.compose.project")
            .map(|(_, v)| v.to_string());

        Self {
            name: raw.name,
            driver: raw.driver,
            mountpoint: raw.mountpoint,
            scope: raw.scope,
            compose_project,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSummary {
    pub id: String,
    /// "bridge"/"host"/"none" are Docker's built-in networks, present in every
    /// installation and refused by `docker network rm`/`prune` — the frontend uses this
    /// to gray them out rather than offer a delete action that will just fail.
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub internal: bool,
    /// The Compose project this network belongs to, if its labels say so.
    pub compose_project: Option<String>,
}

/// Raw shape of one line from `docker network ls --format '{{json .}}'`.
#[derive(Debug, Deserialize)]
pub struct DockerNetworkRaw {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Driver")]
    pub driver: String,
    #[serde(rename = "Scope")]
    pub scope: String,
    #[serde(rename = "Internal")]
    pub internal: String,
    #[serde(rename = "Labels")]
    pub labels: String,
}

impl From<DockerNetworkRaw> for NetworkSummary {
    fn from(raw: DockerNetworkRaw) -> Self {
        let compose_project = split_docker_labels(&raw.labels)
            .into_iter()
            .filter_map(|kv| kv.split_once('='))
            .find(|(k, _)| *k == "com.docker.compose.project")
            .map(|(_, v)| v.to_string());

        Self {
            id: raw.id,
            name: raw.name,
            driver: raw.driver,
            scope: raw.scope,
            internal: raw.internal == "true",
            compose_project,
        }
    }
}

/// Per-resource-type disk usage summary, matching `docker system df`'s own rows
/// (Images / Containers / Local Volumes / Build Cache). Each field is already
/// formatted as Docker's own human-readable string (e.g. "1.2GB", "11.45GB (72%)")
/// rather than parsed, since it's display-only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsageEntry {
    pub kind: String,
    pub total_count: String,
    pub active: String,
    pub size: String,
    pub reclaimable: String,
}

/// Raw shape of one line from `docker system df --format '{{json .}}'`.
#[derive(Debug, Deserialize)]
pub struct DockerDiskUsageRaw {
    #[serde(rename = "Type")]
    pub kind: String,
    #[serde(rename = "TotalCount")]
    pub total_count: String,
    #[serde(rename = "Active")]
    pub active: String,
    #[serde(rename = "Size")]
    pub size: String,
    #[serde(rename = "Reclaimable")]
    pub reclaimable: String,
}

impl From<DockerDiskUsageRaw> for DiskUsageEntry {
    fn from(raw: DockerDiskUsageRaw) -> Self {
        Self {
            kind: raw.kind,
            total_count: raw.total_count,
            active: raw.active,
            size: raw.size,
            reclaimable: raw.reclaimable,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForward {
    pub host_ip: String,
    pub host_port: String,
    pub container_port: String,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountInfo {
    pub mount_type: String,
    pub source: String,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerDetail {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub health: Option<String>,
    pub created: String,
    pub ip_address: Option<String>,
    pub ports: Vec<PortForward>,
    pub mounts: Vec<MountInfo>,
    /// Ordered by key — see `ContainerSummary::labels` for why this isn't a `HashMap`.
    pub labels: BTreeMap<String, String>,
    /// Number of CPU cores this container is limited to, or `None` if it isn't limited
    /// (in which case the caller falls back to however many cores the host has).
    pub cpu_limit_cores: Option<f64>,
    /// The container's `--restart` policy, already formatted the way the Docker CLI
    /// spells it (`no` / `always` / `unless-stopped` / `on-failure` / `on-failure:3`).
    pub restart_policy: String,
}

/// Raw shape of `docker inspect <id>`'s single array element. Only the fields Dockl
/// currently surfaces are modeled; everything else in the (much larger) real payload
/// is ignored by serde by default.
#[derive(Debug, Deserialize)]
pub struct InspectRaw {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Created")]
    pub created: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "State")]
    pub state: InspectState,
    #[serde(rename = "Config")]
    pub config: InspectConfig,
    #[serde(rename = "NetworkSettings")]
    pub network_settings: InspectNetworkSettings,
    #[serde(rename = "Mounts")]
    pub mounts: Vec<InspectMount>,
    #[serde(rename = "HostConfig")]
    pub host_config: InspectHostConfig,
}

/// Only the fields Dockl surfaces from `HostConfig`: the CPU-limiting ones used to
/// compute `ContainerDetail::cpu_limit_cores` (docker's `CPUPerc` stat is relative to a
/// single core, so a multi-core container can read above 100% — the frontend needs to
/// know the real ceiling to scale its chart and show a "MAX n%" figure), plus the
/// `--restart` policy.
#[derive(Debug, Deserialize)]
pub struct InspectHostConfig {
    #[serde(rename = "NanoCpus")]
    pub nano_cpus: i64,
    #[serde(rename = "CpuQuota")]
    pub cpu_quota: i64,
    #[serde(rename = "CpuPeriod")]
    pub cpu_period: i64,
    #[serde(rename = "CpusetCpus")]
    pub cpuset_cpus: String,
    /// `Option` because older Engine versions (and podman's docker-compatible API) can
    /// omit the object entirely; a missing policy means the same thing as `no`.
    #[serde(rename = "RestartPolicy")]
    pub restart_policy: Option<InspectRestartPolicy>,
}

#[derive(Debug, Deserialize)]
pub struct InspectRestartPolicy {
    /// Docker writes an empty string here for containers created without `--restart`,
    /// rather than the literal `"no"` the CLI accepts as input.
    #[serde(rename = "Name")]
    pub name: Option<String>,
    /// Only meaningful for `on-failure`; `0` there means "retry forever".
    #[serde(rename = "MaximumRetryCount")]
    pub maximum_retry_count: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct InspectState {
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Health")]
    pub health: Option<InspectHealth>,
}

#[derive(Debug, Deserialize)]
pub struct InspectHealth {
    #[serde(rename = "Status")]
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct InspectConfig {
    #[serde(rename = "Image")]
    pub image: String,
    #[serde(rename = "Labels")]
    pub labels: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct InspectNetworkSettings {
    #[serde(rename = "Ports")]
    pub ports: Option<HashMap<String, Option<Vec<InspectPortBinding>>>>,
    #[serde(rename = "Networks")]
    pub networks: HashMap<String, InspectNetworkInfo>,
}

#[derive(Debug, Deserialize)]
pub struct InspectPortBinding {
    #[serde(rename = "HostIp")]
    pub host_ip: String,
    #[serde(rename = "HostPort")]
    pub host_port: String,
}

#[derive(Debug, Deserialize)]
pub struct InspectNetworkInfo {
    #[serde(rename = "IPAddress")]
    pub ip_address: String,
}

#[derive(Debug, Deserialize)]
pub struct InspectMount {
    #[serde(rename = "Type")]
    pub mount_type: String,
    #[serde(rename = "Source")]
    pub source: String,
    #[serde(rename = "Destination")]
    pub destination: String,
}

/// Orders port forwards so the detail panel's table stops reshuffling between refreshes.
///
/// Both connection modes deserialize `NetworkSettings.Ports` from a JSON *object* into a
/// `HashMap`, and iterating one of those yields a different order per instance — so every
/// inspect returned the same bindings in a different order, and the table visibly
/// rearranged itself on each poll.
///
/// Sorted on the host port first, matching the table's leading column. The remaining
/// fields break ties rather than being decorative: one host port legitimately appears
/// once per address family (`0.0.0.0` and `::` for the same mapping), so `host_ip` is
/// needed to make the order total.
pub(crate) fn sort_port_forwards(ports: &mut [PortForward]) {
    // Ports are strings here because that's how both APIs report them; anything
    // unparseable sorts last rather than silently collapsing to 0.
    let numeric = |s: &str| s.parse::<u32>().unwrap_or(u32::MAX);
    ports.sort_by(|a, b| {
        numeric(&a.host_port)
            .cmp(&numeric(&b.host_port))
            .then_with(|| numeric(&a.container_port).cmp(&numeric(&b.container_port)))
            .then_with(|| a.protocol.cmp(&b.protocol))
            .then_with(|| a.host_ip.cmp(&b.host_ip))
    });
}

impl From<InspectRaw> for ContainerDetail {
    fn from(raw: InspectRaw) -> Self {
        let mut ports: Vec<PortForward> = raw
            .network_settings
            .ports
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(container_port_proto, bindings)| {
                let (container_port, protocol) = container_port_proto.split_once('/')?;
                let bindings = bindings?;
                Some((container_port.to_string(), protocol.to_string(), bindings))
            })
            .flat_map(|(container_port, protocol, bindings)| {
                bindings.into_iter().map(move |b| PortForward {
                    host_ip: b.host_ip.clone(),
                    host_port: b.host_port.clone(),
                    container_port: container_port.clone(),
                    protocol: protocol.clone(),
                })
            })
            .collect();
        sort_port_forwards(&mut ports);

        let mounts = raw
            .mounts
            .into_iter()
            .map(|m| MountInfo {
                mount_type: m.mount_type,
                source: m.source,
                destination: m.destination,
            })
            .collect();

        let ip_address = raw
            .network_settings
            .networks
            .values()
            .next()
            .map(|n| n.ip_address.clone())
            .filter(|ip| !ip.is_empty());

        let cpu_limit_cores = cpu_limit_cores(&raw.host_config);
        let restart_policy = restart_policy_label(&raw.host_config);

        Self {
            id: raw.id,
            name: raw.name.trim_start_matches('/').to_string(),
            image: raw.config.image,
            status: raw.state.status,
            health: raw.state.health.map(|h| h.status),
            created: raw.created,
            ip_address,
            ports,
            mounts,
            labels: raw.config.labels.unwrap_or_default(),
            cpu_limit_cores,
            restart_policy,
        }
    }
}

/// Applies the same precedence Docker itself uses when a container starts: an explicit
/// `--cpus` limit (`NanoCpus`) first, then the older quota/period pair
/// (`--cpu-quota`/`--cpu-period`), then a pinned core set (`--cpuset-cpus`). `None` means
/// the container isn't CPU-limited at all.
///
/// `pub(crate)` rather than private: `EngineApiConnection::inspect_container` reuses this
/// (building an `InspectHostConfig` from bollard's own `HostConfig`) rather than
/// duplicating the precedence logic for a second connection mode.
pub(crate) fn cpu_limit_cores(host_config: &InspectHostConfig) -> Option<f64> {
    if host_config.nano_cpus > 0 {
        return Some(host_config.nano_cpus as f64 / 1_000_000_000.0);
    }

    if host_config.cpu_quota > 0 && host_config.cpu_period > 0 {
        return Some(host_config.cpu_quota as f64 / host_config.cpu_period as f64);
    }

    count_cpuset(&host_config.cpuset_cpus).map(|count| count as f64)
}

/// Renders `HostConfig.RestartPolicy` the way the Docker CLI spells the `--restart`
/// value that produced it: `no`, `always`, `unless-stopped`, `on-failure`, or
/// `on-failure:<n>` when a retry cap is set. Inspect's empty `Name` (what a container
/// created without `--restart` reports) normalizes to `no`, since that's the flag value
/// meaning the same thing.
///
/// `pub(crate)` for the same reason as `cpu_limit_cores`: `EngineApiConnection` builds an
/// `InspectHostConfig` from bollard's own model and reuses this rather than duplicating
/// the formatting.
pub(crate) fn restart_policy_label(host_config: &InspectHostConfig) -> String {
    let Some(policy) = &host_config.restart_policy else {
        return "no".to_string();
    };

    let name = policy.name.as_deref().unwrap_or_default().trim();
    if name.is_empty() {
        return "no".to_string();
    }

    match policy.maximum_retry_count {
        // A cap only applies to `on-failure`; Docker leaves it at 0 ("retry forever")
        // for every other policy, and printing `always:0` would be nonsense.
        Some(count) if name == "on-failure" && count > 0 => format!("on-failure:{count}"),
        _ => name.to_string(),
    }
}

/// Parses a `--cpuset-cpus`-style spec (e.g. `"0-1,4"` -> 3) into how many cores it names.
fn count_cpuset(spec: &str) -> Option<u32> {
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }
    let mut count = 0u32;
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match part.split_once('-') {
            Some((start, end)) => {
                let start: i64 = start.trim().parse().ok()?;
                let end: i64 = end.trim().parse().ok()?;
                if end < start {
                    return None;
                }
                count += (end - start + 1) as u32;
            }
            None => {
                part.parse::<i64>().ok()?;
                count += 1;
            }
        }
    }
    (count > 0).then_some(count)
}
