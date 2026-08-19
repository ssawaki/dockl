use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::Notify;

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

/// Set whenever a [`is_distro_running`] check finds the distro stopped, cleared when one
/// finds it running or when the user explicitly asks to connect. Read by
/// [`refuse_if_stopped`], which every `wsl.exe`-into-the-distro spawn site consults.
///
/// A process-wide flag rather than a field on `AppState`: `ShellOutConnection`,
/// `DialStdioConnection` and `LogStreamManager` all have to consult it and none of them
/// holds (or should hold) a Tauri `State` handle. There is one WSL per machine, so there
/// is nothing per-instance to keep here anyway.
static DISTRO_STOPPED: AtomicBool = AtomicBool::new(false);

/// Signalled when a check finds the distro running again after it had been seen stopped.
/// This is what lets the `docker events` loop wait for WSL to come back without polling
/// for it — the only other way to notice would be to keep spawning `wsl -l -v` on a timer,
/// which is the very kind of unattended `wsl.exe` this gate exists to get rid of.
static DISTRO_UP: Notify = Notify::const_new();

/// Ceiling on how long [`wait_for_distro_up`] trusts the signal above. Purely a safety
/// net for a wake-up that somehow never arrives: at this cadence being stopped costs one
/// `wsl -l -v` per five minutes, against one every five seconds for the timer it replaced.
const DISTRO_UP_FALLBACK: Duration = Duration::from_secs(300);

/// Waits until the distro is seen running again. Spawns nothing while waiting.
///
/// The wake-up comes from whoever next calls [`is_distro_running`]: the window regaining
/// focus, or the user pressing "start" — i.e. from a person doing something, rather than
/// from a timer asking over and over.
pub async fn wait_for_distro_up() {
    let _ = tokio::time::timeout(DISTRO_UP_FALLBACK, DISTRO_UP.notified()).await;
}

/// Fails without spawning anything when the distro is known to be stopped.
///
/// Every `wsl.exe` call into a distro boots the whole WSL2 VM, and `wsl.exe` has no flag
/// to opt out of that (checked against its full `--help`), so refusing up front is the
/// only way not to. Automatic work — a stats poll, a redial after the relay died, a list
/// refresh off a docker event — must not boot the VM behind the user's back; only
/// [`with_connect_timeout`], i.e. a connect the user asked for, lifts this.
pub(crate) fn refuse_if_stopped() -> Result<(), AppError> {
    if DISTRO_STOPPED.load(Ordering::Relaxed) {
        return Err(AppError::DistroStopped);
    }
    Ok(())
}

/// `wsl -l -v` is answered by the WSL service on the Windows side rather than by the
/// distro, so it stays fast (measured 58–158ms) even while commands *into* the distro
/// hang. That makes it usable as a pre-flight check when the distro itself may be
/// unresponsive — which is the whole point of asking before committing to a timeout.
///
/// This is the app's only observation of whether the distro is up, so it doubles as the
/// thing that maintains [`DISTRO_STOPPED`] and wakes [`wait_for_distro_up`] — which is how
/// the app recovers on its own when the user starts WSL somewhere else entirely and then
/// comes back to the window.
pub async fn is_distro_running(distro: &str) -> bool {
    let listed = tokio::time::timeout(Duration::from_secs(5), list_distros()).await;
    let Ok(Ok(distros)) = listed else {
        // Nothing was learned — `wsl.exe` itself didn't answer. Leaving the flag alone
        // beats guessing "stopped" from a hiccup and refusing calls that would have
        // worked; the next check gets another go.
        return false;
    };
    let running = distros.iter().any(|d| d.name == distro && d.is_running);
    let was_stopped = DISTRO_STOPPED.swap(!running, Ordering::Relaxed);
    if running && was_stopped {
        // `notify_one` rather than `notify_waiters`: it leaves a permit behind when the
        // events loop isn't parked yet, so a wake-up can't fall into the gap between that
        // loop's own check and its await.
        DISTRO_UP.notify_one();
    }
    running
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
    // Only the three connect commands wrap themselves in this, and reaching one of them
    // means the user asked to connect — the one action allowed to boot a stopped distro.
    // So lift the gate the check above may just have raised, before `op` (which runs
    // inside `refuse_if_stopped`) is ever polled.
    DISTRO_STOPPED.store(false, Ordering::Relaxed);
    let result = match tokio::time::timeout(limit, op).await {
        Ok(result) => result,
        Err(_) => Err(AppError::ConnectTimeout(limit.as_secs())),
    };

    if result.is_ok() {
        // The gate was lifted by hand above, so `is_distro_running` never saw the distro
        // come up and never signalled it. Without this the `docker events` loop would stay
        // parked in `wait_for_distro_up` — connected, but with no subscription — until its
        // fallback expired. `notify_one` leaves a permit behind rather than being dropped
        // when nobody is parked yet; the cost is that a later wait can return spuriously,
        // which is harmless because its caller re-checks the real state anyway.
        DISTRO_UP.notify_one();
    } else {
        // The gate was lifted for an attempt that then failed, and nothing else would put
        // it back: a failed `setup_connect` never reaches `event_manager.start`, so the
        // `docker events` loop — the only other thing that observes the distro — isn't
        // running to notice. Left as-is, one failed connect would re-arm every automatic
        // `wsl.exe` spawn in the app for the rest of the session. Costs one `wsl -l -v`,
        // and only on the failure path.
        is_distro_running(distro).await;
    }
    result
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
    refuse_if_stopped()?;

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
    refuse_if_stopped()?;

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
