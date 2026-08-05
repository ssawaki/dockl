import { load } from "@tauri-apps/plugin-store";
import { setupConnect, connectTcpBridge, connectDialStdio } from "$lib/ipc/setup";
import { TCP_BRIDGE_PORT } from "$lib/tcpBridge";

const STORE_PATH = "settings.json";

/**
 * `dial_stdio` is the recommended mode: same Engine API as `user_managed_tcp` at
 * indistinguishable speed, but with no port opened and no setup to undo. `shell_out`
 * remains the safe default and the fallback whenever another mode fails to reconnect.
 */
export type ConnectionMode = "shell_out" | "dial_stdio" | "user_managed_tcp";

/**
 * Reconnects using whichever distro was persisted from a previous successful setup.
 * Returns false (rather than throwing) when there is nothing persisted yet, or the
 * persisted distro is no longer reachable — callers should send the user to `/setup`
 * in that case.
 *
 * `setupConnect` (ShellOut) always runs first regardless of the saved connection mode:
 * `current_distro` — which Compose/logs/stats/attach all depend on directly, unrelated
 * to `DockerConnection` — only gets set as a side effect of that call, and `dial_stdio`
 * needs it to know which distro to spawn its relay in. If the saved mode is one of the
 * Engine API modes, `state.connection` is then switched over on top of that; a failure
 * here is swallowed rather than surfaced, leaving the already-working ShellOut connection
 * in place; the user can re-select the mode from Settings.
 */
export async function ensureConnected(): Promise<boolean> {
  const store = await load(STORE_PATH, { autoSave: true });
  const distro = await store.get<string>("connectedDistro");
  if (!distro) return false;

  try {
    await setupConnect(distro);
  } catch {
    return false;
  }

  const mode = await store.get<ConnectionMode>("connectionMode");
  if (mode === "dial_stdio" || mode === "user_managed_tcp") {
    try {
      await (mode === "dial_stdio" ? connectDialStdio() : connectTcpBridge(TCP_BRIDGE_PORT));
    } catch (e) {
      console.error(`Failed to reconnect via ${mode}, staying on shell_out:`, e);
    }
  }

  return true;
}

/**
 * Records the distro a just-completed setup connected to, and settles which mode to
 * start on.
 *
 * `dial_stdio` is preferred and so is tried here rather than simply assumed: it rests on
 * `docker system dial-stdio`, an undocumented subcommand that isn't guaranteed to exist
 * in every docker CLI build (a podman shim, say), and a first launch that lands on a
 * broken mode is a bad first impression. Probing costs one relay spawn, once.
 *
 * `setupConnect` must already have succeeded — that call is what sets the
 * `current_distro` the relay gets spawned in. Returns the mode actually settled on.
 */
export async function persistConnectedDistro(distro: string): Promise<ConnectionMode> {
  const store = await load(STORE_PATH, { autoSave: true });
  await store.set("connectedDistro", distro);

  let mode: ConnectionMode = "shell_out";
  try {
    await connectDialStdio();
    mode = "dial_stdio";
  } catch (e) {
    console.error("dial_stdio unavailable during setup, starting on shell_out:", e);
  }
  await store.set("connectionMode", mode satisfies ConnectionMode);
  return mode;
}

/** The distro a previous setup connected to, or null if setup hasn't been completed. */
export async function getConnectedDistro(): Promise<string | null> {
  const store = await load(STORE_PATH, { autoSave: true });
  return (await store.get<string>("connectedDistro")) ?? null;
}

export async function getConnectionMode(): Promise<ConnectionMode> {
  const store = await load(STORE_PATH, { autoSave: true });
  return (await store.get<ConnectionMode>("connectionMode")) ?? "shell_out";
}

export async function persistConnectionMode(mode: ConnectionMode): Promise<void> {
  const store = await load(STORE_PATH, { autoSave: true });
  await store.set("connectionMode", mode);
}
