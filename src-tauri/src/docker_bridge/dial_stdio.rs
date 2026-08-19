use std::pin::Pin;
use std::process::Stdio;
use std::sync::{Arc, Mutex as StdMutex};
use std::task::{Context, Poll};

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::client::conn::http1::SendRequest;
use hyper::{Method, Request};
use hyper_util::rt::TokioIo;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, ReadBuf};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::Mutex;

use crate::error::AppError;
use crate::wsl::wsl_command;

/// How much of the stream's opening bytes to keep for diagnostics. Enough to hold a
/// `wsl.exe` error message (UTF-16LE, so two bytes per character) without keeping the
/// whole response around.
const HEAD_CAPTURE_LIMIT: usize = 512;

/// Joins a child process's stdin and stdout into the single bidirectional stream hyper
/// needs. Reads come from the child's stdout, writes go to its stdin — which is exactly
/// what a socket looks like from hyper's point of view, and `docker system dial-stdio`
/// bridges that pair onto the Docker socket on the far side.
struct ChildPipe {
    stdin: ChildStdin,
    stdout: ChildStdout,
    /// A copy of the first bytes the child wrote, kept because `wsl.exe` reports its own
    /// failures on **stdout** — the very pipe hyper is parsing as an HTTP response.
    /// (Verified: with stderr discarded, `wsl.exe -d NoSuchDistro` still prints
    /// `WSL_E_DISTRO_NOT_FOUND`; with stdout discarded, nothing appears.) hyper can only
    /// report that as "invalid HTTP version parsed", which tells the user nothing, so the
    /// raw bytes are retained to surface the real message instead.
    head: Arc<StdMutex<Vec<u8>>>,
}

impl AsyncRead for ChildPipe {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let before = buf.filled().len();
        let polled = Pin::new(&mut self.stdout).poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = &polled {
            let fresh = &buf.filled()[before..];
            if !fresh.is_empty() {
                if let Ok(mut head) = self.head.lock() {
                    let room = HEAD_CAPTURE_LIMIT.saturating_sub(head.len());
                    if room > 0 {
                        head.extend_from_slice(&fresh[..fresh.len().min(room)]);
                    }
                }
            }
        }
        polled
    }
}

impl AsyncWrite for ChildPipe {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.stdin).poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stdin).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stdin).poll_shutdown(cx)
    }
}

/// Talks to the Docker Engine API over `wsl.exe -d <distro> --exec docker system
/// dial-stdio`, a hidden Docker CLI subcommand that bridges its own stdin/stdout onto the
/// Docker socket (it's the transport `docker -H ssh://...` and `docker context` are built
/// on).
///
/// Chosen over exposing the Engine API on a TCP port because there is no port: the only
/// thing that can reach the daemon through this is the child process Dockl spawned
/// itself, so there is nothing to authenticate, nothing left listening, and nothing to
/// undo afterwards — unlike "TCP接続", which requires writing a systemd override that
/// leaves an unauthenticated Engine API open to every process on the machine (see
/// `TcpBridgeSetupDialog`'s warning). It also needs no setup at all, and asks nothing of
/// the distro beyond the `docker` CLI this app already requires.
///
/// `dial-stdio` resolves the socket path through the Docker CLI's own context rather than
/// a hardcoded `/var/run/docker.sock`, so rootless installs (whose socket lives under
/// `$XDG_RUNTIME_DIR`) work without special handling — see PLAN.md's "Docker導入方法の差異"
/// risk.
pub struct DialStdioConnection {
    distro: String,
    /// `None` until the first request, and reset back to `None` whenever the connection
    /// breaks (WSL restarted, daemon stopped) so the next call transparently redials.
    sender: Mutex<Option<SendRequest<Full<Bytes>>>>,
    /// Whatever the last child wrote to stderr. `docker`'s own failures land there
    /// ("permission denied", "command not found"), and hyper only ever reports them to
    /// us as a closed stream — without this the user would get "connection closed" with
    /// no hint of the actual cause. Note this does *not* cover `wsl.exe`'s own errors,
    /// which go to stdout; `head` below is what catches those.
    last_stderr: Arc<Mutex<String>>,
    /// First bytes of the current relay's stdout — see `ChildPipe::head`.
    head: Arc<StdMutex<Vec<u8>>>,
}

impl DialStdioConnection {
    pub fn new(distro: String) -> Self {
        Self {
            distro,
            sender: Mutex::new(None),
            last_stderr: Arc::new(Mutex::new(String::new())),
            head: Arc::new(StdMutex::new(Vec::new())),
        }
    }

    /// Turns a transport-level failure into something a user can act on.
    ///
    /// hyper's own text for these is useless on its own — a `wsl.exe` error message
    /// landing on stdout surfaces as "invalid HTTP version parsed", and a relay that died
    /// surfaces as a closed channel. Both cases have the real explanation sitting in the
    /// bytes the child actually wrote, so those win over hyper's wording whenever they
    /// don't look like an HTTP response.
    async fn explain(&self, err: impl std::fmt::Display) -> AppError {
        let head = self.head.lock().ok().map(|h| h.clone()).unwrap_or_default();
        let decoded = crate::wsl::decode_process_output(&head);
        let decoded = decoded.trim();
        if !decoded.is_empty() && !decoded.starts_with("HTTP/") {
            return AppError::WslUnavailable(decoded.to_string());
        }
        let stderr = self.last_stderr.lock().await.clone();
        if !stderr.is_empty() {
            return AppError::CommandFailed(format!("{err}: {stderr}"));
        }
        AppError::CommandFailed(err.to_string())
    }

    /// Spawns a fresh relay process and completes an HTTP/1.1 handshake over its stdio.
    async fn dial(&self) -> Result<SendRequest<Full<Bytes>>, AppError> {
        // Redialing is automatic (any request finding no live relay does it), so without
        // this a single background refresh would boot a distro the user had stopped.
        crate::wsl::refuse_if_stopped()?;

        let mut child = wsl_command()
            .args([
                "-d",
                &self.distro,
                "--exec",
                "docker",
                "system",
                "dial-stdio",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // Ties the relay's lifetime to this connection: dropping the `Child` (which
            // happens when the driver task below ends) kills `wsl.exe`, and killing
            // `wsl.exe` takes the Linux-side `docker system dial-stdio` with it, so a
            // crashed or reconnecting Dockl never accumulates orphans inside the distro.
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| AppError::WslUnavailable(e.to_string()))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppError::CommandFailed("no stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::CommandFailed("no stdout".into()))?;
        let stderr = child.stderr.take();

        // Reset per dial: the buffer describes the relay currently on the other end, and
        // a stale message from a previous one would be actively misleading.
        if let Ok(mut head) = self.head.lock() {
            head.clear();
        }
        let pipe = ChildPipe {
            stdin,
            stdout,
            head: self.head.clone(),
        };
        let (sender, conn) = hyper::client::conn::http1::handshake(TokioIo::new(pipe))
            .await
            .map_err(|e| AppError::CommandFailed(e.to_string()))?;

        let stderr_slot = self.last_stderr.clone();
        tokio::spawn(async move {
            // Owned here rather than by `DialStdioConnection` so that the child is dropped
            // (and so killed) exactly when the connection it belongs to ends.
            let _child = child;
            if let Some(mut stderr) = stderr {
                let mut buf = String::new();
                tokio::select! {
                    _ = stderr.read_to_string(&mut buf) => {
                        if !buf.trim().is_empty() {
                            *stderr_slot.lock().await = buf.trim().to_string();
                        }
                    }
                    _ = conn => {}
                }
            } else {
                let _ = conn.await;
            }
        });

        Ok(sender)
    }

    /// Sends one request, redialing once if the existing relay turned out to be dead —
    /// which is the normal state of affairs after `wsl --shutdown`, a daemon restart, or
    /// the distro being stopped and started again.
    ///
    /// One retry rather than a loop: a second failure means the daemon or the distro is
    /// genuinely unreachable, and retrying further would only delay the error the user
    /// needs to see. Only a lost transport is retried; a Docker-level error (a 404 for a
    /// container that doesn't exist, say) would fail identically on a fresh relay.
    pub async fn request(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<Bytes, AppError> {
        match self.attempt(method.clone(), path, query, false).await {
            Attempt::Ok(body) => Ok(body),
            Attempt::Failed(e) => Err(e),
            Attempt::TransportLost(_) => match self.attempt(method, path, query, true).await {
                Attempt::Ok(body) => Ok(body),
                Attempt::Failed(e) | Attempt::TransportLost(e) => Err(e),
            },
        }
    }

    async fn attempt(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        force_redial: bool,
    ) -> Attempt {
        match self.try_request(method, path, query, force_redial).await {
            Ok(body) => Attempt::Ok(body),
            Err((e, transport_lost)) => {
                if transport_lost {
                    Attempt::TransportLost(e)
                } else {
                    Attempt::Failed(e)
                }
            }
        }
    }

    /// The `bool` in the error is "the relay is gone", telling `request` whether a retry
    /// could plausibly succeed. Returned alongside the error rather than checked by
    /// re-reading `self.sender` afterwards: that read would need the same lock this holds,
    /// and taking it from a `match` guard makes whether it deadlocks depend on exactly
    /// when the guard's temporary is dropped.
    async fn try_request(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        force_redial: bool,
    ) -> Result<Bytes, (AppError, bool)> {
        let mut guard = self.sender.lock().await;
        if force_redial || guard.is_none() {
            // A relay that can't even be spawned is itself a lost transport, but there's
            // no point retrying a spawn that just failed.
            *guard = Some(self.dial().await.map_err(|e| (e, false))?);
        }
        let sender = guard.as_mut().expect("just dialed");

        let uri = if query.is_empty() {
            path.to_string()
        } else {
            let qs = serde_urlencoded::to_string(query)
                .map_err(|e| (AppError::ParseError(e.to_string()), false))?;
            format!("{path}?{qs}")
        };

        let req = Request::builder()
            .method(method)
            .uri(&uri)
            // hyper/1.1 requires a Host header. There's no real host on this transport, so
            // the value is arbitrary — Docker ignores it — but it has to be present and
            // well-formed for the request to be accepted at all.
            .header(hyper::header::HOST, "docker")
            .body(Full::new(Bytes::new()))
            .map_err(|e| (AppError::CommandFailed(e.to_string()), false))?;

        let resp = match sender.send_request(req).await {
            Ok(resp) => resp,
            Err(e) => {
                // The relay is gone; clear it so the retry (or the next call) dials a new
                // one instead of reusing a stream that will never answer.
                *guard = None;
                return Err((self.explain(e).await, true));
            }
        };

        let status = resp.status();
        let body = match resp.into_body().collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(e) => return Err((self.explain(e).await, true)),
        };

        if status.is_success() {
            return Ok(body);
        }

        // Docker's own API errors are `{"message": "..."}`; surface just that rather than
        // the raw JSON, matching what the TCP transport does with the same responses.
        let text = String::from_utf8_lossy(&body).to_string();
        let message = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v.get("message")?.as_str().map(str::to_string))
            .unwrap_or(text);
        // A Docker-level error, not a transport one — the relay is fine and answering.
        Err((
            AppError::CommandFailed(format!("{status}: {message}")),
            false,
        ))
    }
}

/// Outcome of a single send, distinguishing "the relay died" (worth one redial) from
/// "Docker said no" (redialing would produce the same answer).
enum Attempt {
    Ok(Bytes),
    Failed(AppError),
    TransportLost(AppError),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exercises the transport against a real daemon: relay spawn, HTTP/1.1 handshake,
    /// connection reuse across calls, query encoding, and Docker's own error shape.
    ///
    /// `#[ignore]`d because it needs a running WSL2 distro with Docker, unlike the pure
    /// unit tests elsewhere in this crate. Run it explicitly with:
    ///   `cargo test dial_stdio -- --ignored --nocapture`
    /// and override the distro with `DOCKL_TEST_DISTRO` if it isn't named "Ubuntu".
    #[tokio::test]
    #[ignore]
    async fn round_trips_against_live_daemon() {
        let distro = std::env::var("DOCKL_TEST_DISTRO").unwrap_or_else(|_| "Ubuntu".to_string());
        let conn = DialStdioConnection::new(distro);

        let ping = conn
            .request(Method::GET, "/_ping", &[])
            .await
            .expect("ping failed");
        assert_eq!(&ping[..], b"OK", "unexpected /_ping body");

        // A second call must reuse the relay opened by the first rather than redialing.
        let version = conn
            .request(Method::GET, "/version", &[])
            .await
            .expect("version failed");
        assert!(version.starts_with(b"{"), "/version was not JSON");

        // Query parameters have to survive urlencoding: `filters` is a JSON blob, which
        // is the most fragile case the app actually sends.
        let filters = r#"{"dangling":["true"]}"#.to_string();
        conn.request(Method::GET, "/images/json", &[("filters", filters)])
            .await
            .expect("filtered image list failed");

        // Docker's own 404 body is `{"message": "..."}`; the transport should surface
        // that text rather than the raw JSON.
        let err = conn
            .request(Method::GET, "/containers/dockl-no-such-container/json", &[])
            .await
            .expect_err("expected a 404");
        let text = err.to_string();
        assert!(text.contains("404"), "error lost its status: {text}");
        assert!(
            !text.contains("{\"message\""),
            "error body was not unwrapped: {text}"
        );
    }

    /// `wsl.exe` reports its own failures on stdout, so they arrive on the very stream
    /// hyper is parsing as an HTTP response and it can only call them "invalid HTTP
    /// version parsed". This checks the real message is surfaced instead.
    ///
    /// Deterministic — it only needs `wsl.exe` to exist and to reject a distro name that
    /// doesn't — but `#[ignore]`d with the rest since it still shells out to WSL.
    #[tokio::test]
    #[ignore]
    async fn reports_wsl_error_rather_than_a_parse_failure() {
        let conn = DialStdioConnection::new("dockl-no-such-distro".to_string());
        let err = conn
            .request(Method::GET, "/_ping", &[])
            .await
            .expect_err("a nonexistent distro should not connect");
        let text = err.to_string();

        assert!(
            !text.contains("invalid HTTP version"),
            "hyper's parse error leaked through instead of WSL's own message: {text}",
        );
        // `wsl.exe` localizes this text, so match on the stable error code it always
        // appends rather than on any particular language's wording.
        assert!(
            text.contains("WSL_E_DISTRO_NOT_FOUND"),
            "expected WSL's own explanation, got: {text}",
        );
    }

    /// Kills the relay out from under a live connection and checks the next request
    /// transparently dials a new one — the situation every `wsl --shutdown`, daemon
    /// restart, or stopped distro produces.
    ///
    /// `#[ignore]`d like the others, and additionally note that the `pkill` below hits
    /// *every* `dial-stdio` process in the distro, including any belonging to a running
    /// Dockl. That's harmless (it would simply redial too, which is the very thing under
    /// test) but it does mean this isn't a test to run casually against a busy machine.
    #[tokio::test]
    #[ignore]
    async fn redials_after_the_relay_dies() {
        let distro = std::env::var("DOCKL_TEST_DISTRO").unwrap_or_else(|_| "Ubuntu".to_string());
        let conn = DialStdioConnection::new(distro.clone());

        conn.request(Method::GET, "/_ping", &[])
            .await
            .expect("initial ping failed");

        let status = wsl_command()
            .args([
                "-d",
                &distro,
                "--exec",
                "pkill",
                "-f",
                "docker system dial-stdio",
            ])
            .status()
            .await
            .expect("could not run pkill");
        // A non-zero exit means either "matched nothing" or "wsl.exe never reached the
        // distro", and the exit code alone can't tell them apart — the latter is common
        // on a WSL that intermittently stops responding. Either way no relay was killed,
        // so there's nothing for the redial below to prove.
        assert!(
            status.success(),
            "could not kill the relay — either none was running, or WSL did not respond. \
             Test is inconclusive rather than failed; re-run once WSL is answering.",
        );

        let ping = conn
            .request(Method::GET, "/_ping", &[])
            .await
            .expect("request after the relay died should have redialed");
        assert_eq!(&ping[..], b"OK", "unexpected body from the redialed relay");
    }
}
