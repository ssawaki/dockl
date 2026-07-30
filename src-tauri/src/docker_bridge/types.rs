use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub labels: HashMap<String, String>,
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
        let labels = raw
            .labels
            .split(',')
            .filter(|s| !s.is_empty())
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
    pub labels: HashMap<String, String>,
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
    pub labels: Option<HashMap<String, String>>,
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

impl From<InspectRaw> for ContainerDetail {
    fn from(raw: InspectRaw) -> Self {
        let ports = raw
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
        }
    }
}
