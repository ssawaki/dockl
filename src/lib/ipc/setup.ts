import { invoke } from "@tauri-apps/api/core";
import type { DistroInfo } from "$lib/types";

export function setupListDistros(): Promise<DistroInfo[]> {
  return invoke("setup_list_distros");
}

export function setupConnect(distro: string): Promise<void> {
  return invoke("setup_connect", { distro });
}

/**
 * Whether WSL currently reports this distro as running. Answered by the WSL service on
 * the Windows side rather than by the distro itself, so it stays fast even when commands
 * into the distro hang — which is what makes it usable to pick the right "connecting"
 * versus "starting" message before committing to a connection attempt.
 */
export function setupDistroIsRunning(distro: string): Promise<boolean> {
  return invoke("setup_distro_is_running", { distro });
}

export function setupCurrentDistro(): Promise<string | null> {
  return invoke("setup_current_distro");
}

/** Verifies a Docker daemon is reachable and responds at `tcp://127.0.0.1:<port>`. */
export function checkTcpBridge(port: number): Promise<void> {
  return invoke("check_tcp_bridge", { port });
}

/**
 * Switches the app's live connection (container/image/volume/network list/action/prune)
 * over to talking directly to `tcp://127.0.0.1:<port>` instead of shelling out to
 * `wsl.exe`. Compose/logs/stats/attach are unaffected — they always shell out regardless.
 */
export function connectTcpBridge(port: number): Promise<void> {
  return invoke("connect_tcp_bridge", { port });
}

/**
 * Switches the live connection over to talking to the Docker Engine API through a
 * `docker system dial-stdio` relay child process. Takes no port because it opens none —
 * it reuses the distro already connected via `setupConnect`.
 */
export function connectDialStdio(): Promise<void> {
  return invoke("connect_dial_stdio", {});
}
