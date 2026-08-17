fn main() {
    // tauri_build::build() declares tauri.conf.json as a rerun trigger, but not
    // whatever that file's `version` points at — and ours points at
    // ../package.json. Without this line a version bump never invalidates the
    // build script, so the old number stays compiled into the exe's Windows
    // resource while the installer filename picks up the new one: an installer
    // called 0.1.1 whose exe reports 0.1.0.
    println!("cargo:rerun-if-changed=../package.json");
    emit_git_hash();

    tauri_build::build()
}

/// Bakes the short commit hash in, for the About dialog to name the exact build.
///
/// Both rerun triggers are needed: `.git/HEAD` changes when the branch does, and the ref
/// it points at changes on commit. Miss either and a build made after committing would go
/// on reporting the previous hash — the same staleness the `package.json` trigger above
/// exists to prevent. Empty when git isn't around to ask, e.g. a build from a tarball.
fn emit_git_hash() {
    use std::path::Path;
    use std::process::Command;

    let git_dir = Path::new("../.git");
    println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());

    if let Ok(head) = std::fs::read_to_string(git_dir.join("HEAD"))
        && let Some(reference) = head.strip_prefix("ref: ")
    {
        println!(
            "cargo:rerun-if-changed={}",
            git_dir.join(reference.trim()).display()
        );
    }

    let hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default();

    println!("cargo:rustc-env=DOCKL_GIT_HASH={hash}");
}
