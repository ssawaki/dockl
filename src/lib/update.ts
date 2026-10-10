import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { writable, get } from "svelte/store";
import { t } from "$lib/stores/i18n";
import { showToast } from "$lib/stores/toasts";

export type UpdateStatus = "idle" | "checking" | "upToDate" | "available" | "downloading" | "error";

interface UpdateState {
  status: UpdateStatus;
  update: Update | null;
  error?: string;
}

export const updateState = writable<UpdateState>({ status: "idle", update: null });

/**
 * Checks for a new version without downloading anything. `silent` is for the once-per-launch
 * background check (see `+layout.svelte`): a failure there (e.g. no network yet at cold boot)
 * resets to idle instead of surfacing as an "error" state the Settings page would otherwise
 * show the next time it's opened, for a check the user never asked for.
 *
 * Does nothing under `tauri dev`: the endpoint is the production `latest.json`, so a dev build
 * would be offered — and would install — the production installer, a different product.
 * `VITE_UPDATE_CHECK=1` opts back in, for looking at the Settings UI.
 */
export async function checkForUpdate({
  silent = false,
}: { silent?: boolean } = {}): Promise<Update | null> {
  if (import.meta.env.DEV && !import.meta.env.VITE_UPDATE_CHECK) return null;
  updateState.set({ status: "checking", update: null });
  try {
    const update = await check();
    updateState.set(
      update ? { status: "available", update } : { status: "upToDate", update: null },
    );
    return update;
  } catch (e) {
    updateState.set(
      silent
        ? { status: "idle", update: null }
        : { status: "error", update: null, error: String(e) },
    );
    return null;
  }
}

/**
 * Downloads, installs, and relaunches into the new version. Only ever called from a Settings
 * button click, so a failure surfaces as a toast rather than through `updateState` — the
 * page's own status text has already done its job by the time this runs.
 */
export async function installUpdate(update: Update): Promise<void> {
  updateState.update((s) => ({ ...s, status: "downloading" }));
  try {
    await update.downloadAndInstall();
    await relaunch();
  } catch (e) {
    updateState.update((s) => ({ ...s, status: "available" }));
    showToast("error", get(t)("settings.update.installFailed", { error: String(e) }));
  }
}
