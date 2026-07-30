import { load } from "@tauri-apps/plugin-store";
import { setupConnect } from "$lib/ipc/setup";

const STORE_PATH = "settings.json";

/**
 * Reconnects using whichever distro was persisted from a previous successful setup.
 * Returns false (rather than throwing) when there is nothing persisted yet, or the
 * persisted distro is no longer reachable — callers should send the user to `/setup`
 * in that case.
 */
export async function ensureConnected(): Promise<boolean> {
  const store = await load(STORE_PATH, { autoSave: true });
  const distro = await store.get<string>("connectedDistro");
  if (!distro) return false;

  try {
    await setupConnect(distro);
    return true;
  } catch {
    return false;
  }
}

export async function persistConnectedDistro(distro: string): Promise<void> {
  const store = await load(STORE_PATH, { autoSave: true });
  await store.set("connectedDistro", distro);
  await store.set("connectionMode", "shell_out");
}
