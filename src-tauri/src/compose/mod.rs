use crate::error::AppError;
use crate::wsl;

/// Runs `docker compose -p <project> -f <file1> -f <file2> ... <up -d|stop|down>` against
/// the given distro. Compose isn't part of the Docker Engine API, so unlike container
/// actions this always shells out regardless of the active `ConnectionMode` (see
/// PLAN.md's Compose section).
///
/// Mirrors the per-container start/stop/restart/remove actions: "up" starts (creating
/// services if needed), "stop" stops without removing anything, "restart" restarts the
/// project's containers in place, and "down" removes the project's containers and
/// network (like `rm -f` does for a single container).
pub async fn compose_action(
    distro: &str,
    project: &str,
    config_files: &[String],
    action: &str,
) -> Result<String, AppError> {
    let mut args: Vec<String> = vec!["compose".into(), "-p".into(), project.into()];
    for file in config_files {
        args.push("-f".into());
        args.push(file.clone());
    }

    match action {
        "up" => {
            args.push("up".into());
            args.push("-d".into());
        }
        "stop" => args.push("stop".into()),
        "restart" => args.push("restart".into()),
        "down" => args.push("down".into()),
        other => {
            return Err(AppError::CommandFailed(format!(
                "unknown compose action: {other}"
            )))
        }
    }

    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    wsl::run_docker_verbose(distro, &arg_refs).await
}
