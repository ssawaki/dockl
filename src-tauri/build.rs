fn main() {
    // tauri_build::build() declares tauri.conf.json as a rerun trigger, but not
    // whatever that file's `version` points at — and ours points at
    // ../package.json. Without this line a version bump never invalidates the
    // build script, so the old number stays compiled into the exe's Windows
    // resource while the installer filename picks up the new one: an installer
    // called 0.1.1 whose exe reports 0.1.0.
    println!("cargo:rerun-if-changed=../package.json");

    tauri_build::build()
}
