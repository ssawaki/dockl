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
fn wsl_command() -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new("wsl.exe");
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// Runs `docker <args>` inside the given distro and returns captured stdout as a lossy
/// UTF-8 string (docker CLI output is UTF-8, unlike `wsl.exe -l -v`'s own UTF-16LE banner).
pub async fn run_docker(distro: &str, args: &[&str]) -> Result<String, AppError> {
    let mut full_args = vec!["-d", distro, "--", "docker"];
    full_args.extend_from_slice(args);

    let output = wsl_command()
        .args(&full_args)
        .output()
        .await
        .map_err(|e| AppError::WslUnavailable(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::CommandFailed(stderr.trim().to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
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
