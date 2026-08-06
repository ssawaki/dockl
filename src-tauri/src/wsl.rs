use std::time::Duration;

use crate::error::AppError;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroInfo {
    pub name: String,
    pub is_default: bool,
    pub is_running: bool,
    pub wsl_version: u32,
}

use serde::{Deserialize, Serialize};

/// Builds a `tokio::process::Command` for `wsl.exe`, suppressing the console window
/// that Windows would otherwise flash for a moment when spawning a console subprocess
/// from a GUI app.
pub(crate) fn wsl_command() -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new("wsl.exe");
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    // Whenever we abandon a call (a timeout below, or a dropped dial-stdio connection),
    // the `wsl.exe` behind it has to go too — otherwise a distro that stopped answering
    // accumulates one stranded process per attempt.
    cmd.kill_on_drop(true);
    cmd
}

/// Ceiling for a connection check against a distro that `wsl -l -v` reports as already
/// running. Such a distro answers in about 1.5s even on a loaded machine, so anything
/// near this means it has stopped responding rather than that it is merely slow — which
/// does happen: WSL can report `Running` while its interop layer is wedged and every
/// command into the distro hangs indefinitely.
const CONNECT_TIMEOUT_RUNNING: Duration = Duration::from_secs(10);

/// Ceiling for a distro that is stopped, where the first command has to boot the VM
/// first. That legitimately takes tens of seconds on a cold start, so this has to be
/// generous enough not to abort a boot that would have succeeded.
const CONNECT_TIMEOUT_STARTING: Duration = Duration::from_secs(60);

/// `wsl -l -v` is answered by the WSL service on the Windows side rather than by the
/// distro, so it stays fast (measured 58–158ms) even while commands *into* the distro
/// hang. That makes it usable as a pre-flight check when the distro itself may be
/// unresponsive — which is the whole point of asking before committing to a timeout.
pub async fn is_distro_running(distro: &str) -> bool {
    let listed = tokio::time::timeout(Duration::from_secs(5), list_distros()).await;
    matches!(listed, Ok(Ok(distros)) if distros.iter().any(|d| d.name == distro && d.is_running))
}

/// Runs `op`, failing with a recoverable error rather than hanging forever if the distro
/// never answers. The ceiling depends on whether the distro is already up, since "boot a
/// VM" and "answer while already booted" differ by an order of magnitude and a single
/// value would either abort legitimate cold starts or leave a wedged distro spinning.
pub async fn with_connect_timeout<T, F>(distro: &str, op: F) -> Result<T, AppError>
where
    F: std::future::Future<Output = Result<T, AppError>>,
{
    let limit = if is_distro_running(distro).await {
        CONNECT_TIMEOUT_RUNNING
    } else {
        CONNECT_TIMEOUT_STARTING
    };
    match tokio::time::timeout(limit, op).await {
        Ok(result) => result,
        Err(_) => Err(AppError::ConnectTimeout(limit.as_secs())),
    }
}

/// Runs `docker <args>` inside the given distro and returns captured stdout, decoded
/// with [`decode_process_output`].
///
/// Most of the time this output is `docker`'s own UTF-8 text. But when `wsl.exe`
/// itself fails before ever reaching `docker` — the distro name is wrong, or WSL isn't
/// running — the message on stderr is `wsl.exe`'s own, which (like its `-l -v` banner)
/// is UTF-16LE. Decoding that unconditionally as UTF-8 corrupted exactly the error text
/// a user needed to read to understand what went wrong.
pub async fn run_docker(distro: &str, args: &[&str]) -> Result<String, AppError> {
    let mut full_args = vec!["-d", distro, "--", "docker"];
    full_args.extend_from_slice(args);

    let output = wsl_command()
        .args(&full_args)
        .output()
        .await
        .map_err(|e| AppError::WslUnavailable(e.to_string()))?;

    if !output.status.success() {
        let stderr = decode_process_output(&output.stderr);
        return Err(AppError::CommandFailed(stderr.trim().to_string()));
    }

    Ok(decode_process_output(&output.stdout))
}

/// Like [`run_docker`], but returns stdout *and* stderr combined on success too, not
/// just on failure. `docker compose`'s own progress (pulling images, creating networks,
/// starting/removing each service) is written to stderr even on success — `run_docker`
/// alone would silently discard exactly the output a user clicking a compose toast to
/// "see what happened" wants to read.
pub async fn run_docker_verbose(distro: &str, args: &[&str]) -> Result<String, AppError> {
    let mut full_args = vec!["-d", distro, "--", "docker"];
    full_args.extend_from_slice(args);

    let output = wsl_command()
        .args(&full_args)
        .output()
        .await
        .map_err(|e| AppError::WslUnavailable(e.to_string()))?;

    let combined = [
        decode_process_output(&output.stdout),
        decode_process_output(&output.stderr),
    ]
    .into_iter()
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join("\n");

    if !output.status.success() {
        return Err(AppError::CommandFailed(if combined.is_empty() {
            "unknown error".to_string()
        } else {
            combined
        }));
    }

    Ok(combined)
}

/// `wsl.exe` writes its own text (banners, its own error messages) as UTF-16LE, while a
/// command it runs inside the distro (e.g. `docker`) writes plain UTF-8 to its
/// redirected stdout/stderr. We can't know in advance which produced a given buffer, so
/// this guesses — in two steps, because neither check alone is reliable:
///
/// 1. Strict UTF-8 validation. Genuine UTF-8 (including Japanese/CJK text, which is
///    what this app's own locale mostly produces) always validates. UTF-16LE text
///    *containing any non-ASCII character* almost never does, because its interleaved
///    high bytes don't form valid UTF-8 continuation sequences — so a validation
///    failure is a strong UTF-16LE signal on its own.
/// 2. But plain ASCII is exactly the case step 1 can't catch: every byte of
///    UTF-16LE-encoded ASCII (each character followed by a `0x00` filler byte) is
///    *also* independently valid as its own single-byte UTF-8 code point, so it passes
///    strict validation despite being the wrong encoding. We only need the blunter
///    "lots of 0x00 bytes" heuristic to catch this remaining case.
pub(crate) fn decode_process_output(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    if std::str::from_utf8(bytes).is_err() {
        return decode_utf16le(bytes);
    }

    let zero_count = bytes.iter().filter(|&&b| b == 0).count();
    let looks_like_padded_ascii_utf16le = zero_count * 3 >= bytes.len();
    if looks_like_padded_ascii_utf16le {
        decode_utf16le(bytes)
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

/// Lists installed WSL distros via `wsl.exe -l -v`. The output of this specific flag is
/// UTF-16LE (with quirky spacing), unlike most other `wsl.exe` invocations, so it needs
/// its own decoding path.
pub async fn list_distros() -> Result<Vec<DistroInfo>, AppError> {
    let output = wsl_command()
        .args(["-l", "-v"])
        .output()
        .await
        .map_err(|e| AppError::WslUnavailable(e.to_string()))?;

    if !output.status.success() {
        let stderr = decode_utf16le(&output.stderr);
        return Err(AppError::WslUnavailable(stderr));
    }

    let text = decode_utf16le(&output.stdout);
    Ok(parse_distro_list(&text))
}

fn decode_utf16le(bytes: &[u8]) -> String {
    let u16s: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .collect();
    String::from_utf16_lossy(&u16s)
        .trim_start_matches('\u{feff}')
        .to_string()
}

fn parse_distro_list(text: &str) -> Vec<DistroInfo> {
    text.lines()
        .skip(1) // header row: "  NAME    STATE    VERSION"
        .filter_map(|line| {
            let line = line.trim_end();
            if line.trim().is_empty() {
                return None;
            }
            let is_default = line.starts_with('*');
            let rest = line.trim_start_matches('*').trim();
            let fields: Vec<&str> = rest.split_whitespace().collect();
            if fields.len() < 3 {
                return None;
            }
            let version: u32 = fields[fields.len() - 1].parse().unwrap_or(0);
            let state = fields[fields.len() - 2].to_string();
            let name = fields[..fields.len() - 2].join(" ");
            Some(DistroInfo {
                name,
                is_default,
                is_running: state.eq_ignore_ascii_case("running"),
                wsl_version: version,
            })
        })
        .collect()
}
