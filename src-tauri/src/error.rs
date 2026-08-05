use std::collections::HashMap;

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

    /// The distro accepted the call but never answered within the ceiling
    /// `wsl::with_connect_timeout` (or `EngineApiConnection::request`) allowed. Distinct
    /// from `WslUnavailable` (which means `wsl.exe` itself failed) because this is the
    /// recoverable "WSL is wedged, try again" case the UI offers a retry for, rather than
    /// a hard misconfiguration.
    #[error("WSL2 did not respond within {0}s")]
    ConnectTimeout(u64),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl AppError {
    /// Stable identifier for this kind of failure, independent of its wording.
    ///
    /// The frontend keys its translations off this rather than off the message text, so
    /// rephrasing an error here can never silently un-translate it — and a code with no
    /// translation still renders, via `message` below.
    fn code(&self) -> &'static str {
        match self {
            Self::WslUnavailable(_) => "wsl_unavailable",
            Self::NoDistroFound => "no_distro_found",
            Self::CommandFailed(_) => "command_failed",
            Self::ParseError(_) => "parse_error",
            Self::NotConfigured => "not_configured",
            Self::ConnectTimeout(_) => "connect_timeout",
            Self::Io(_) => "io",
        }
    }

    /// Values the frontend interpolates into its own translated sentence.
    ///
    /// `detail` is whatever the underlying tool said (a `wsl.exe` message, Docker's own
    /// API error) — untranslatable by definition, so it's passed through for a translated
    /// wrapper to place. Errors with nothing to interpolate return an empty map.
    fn params(&self) -> HashMap<&'static str, String> {
        match self {
            Self::WslUnavailable(detail) | Self::CommandFailed(detail) | Self::ParseError(detail) => {
                HashMap::from([("detail", detail.clone())])
            }
            Self::Io(e) => HashMap::from([("detail", e.to_string())]),
            Self::ConnectTimeout(seconds) => HashMap::from([("seconds", seconds.to_string())]),
            Self::NoDistroFound | Self::NotConfigured => HashMap::new(),
        }
    }
}

/// The wire shape every Tauri command error arrives in. `message` is the English text
/// `thiserror` produces, kept as the fallback for codes the frontend has no translation
/// for (`command_failed` and friends are raw tool output — there's nothing to translate).
#[derive(Serialize)]
struct SerializedError {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    params: HashMap<&'static str, String>,
}

// Tauri commands require errors to be Serialize so they can be sent to the frontend.
// Serialized as an object rather than a bare string so the frontend can translate known
// failures; see `src/lib/errors.ts`, which is the only thing that should read this shape.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializedError {
            code: self.code(),
            message: self.to_string(),
            params: self.params(),
        }
        .serialize(serializer)
    }
}
