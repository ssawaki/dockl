pub mod connection;
pub mod dial_stdio;
pub mod engine_api;
pub mod events;
pub mod logs;
pub mod shell_out;
pub mod types;

pub use connection::DockerConnection;
pub use engine_api::EngineApiConnection;
// Re-exported ahead of its first use, for the same reason `ConnectionMode` itself carries
// `#[allow(dead_code)]`: the modes get wired up incrementally across milestones (see
// PLAN.md). Kept as its own statement so `DockerConnection` above still warns if it ever
// stops being used.
#[allow(unused_imports)]
pub use connection::ConnectionMode;
pub use events::DockerEventManager;
pub use logs::LogStreamManager;
pub use types::{
    ContainerActionKind, ContainerDetail, ContainerSummary, DiskUsageEntry, ImageSummary, NetworkSummary,
    VolumeSummary,
};
