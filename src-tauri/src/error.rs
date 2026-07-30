use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("WSL not available: {0}")]
    WslUnavailable(String),

    #[allow(dead_code)] // reserved for an "auto-pick a distro" fallback, not wired up yet
    #[error("no WSL2 distro with Docker was found")]
    NoDistroFound,

    #[error("command failed: {0}")]
    CommandFailed(String),

    #[error("failed to parse command output: {0}")]
    ParseError(String),

    #[error("docker connection is not configured")]
    NotConfigured,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

// Tauri commands require errors to be Serialize so they can be sent to the frontend.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
