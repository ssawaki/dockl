import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { writable, get } from "svelte/store";
import { t, type MessageKey } from "$lib/stores/i18n";
import { pushToast, resolveToast } from "$lib/stores/toasts";
import { devCheckOptions } from "$lib/update.dev";

export type UpdateStatus = "idle" | "checking" | "upToDate" | "available" | "downloading" | "error";

interface UpdateState {
  status: UpdateStatus;
  update: Update | null;
  error?: string;
}

export const updateState = writable<UpdateState>({ status: "idle", update: null });

/**
 * tauri-plugin-updater's errors reach us only as English text, so these match on that text.
 * A wording change in the plugin stops one of these matching and nothing worse: the caller's
 * fallback is shown instead, with the original text still alongside it.
 */
const KNOWN_ERRORS: [RegExp, MessageKey][] = [
  [/Could not fetch a valid release JSON/, "settings.update.releaseNotFound"],
  [/^The signature |minisign data/, "settings.update.signatureFailed"],
  [/in the response `platforms`|fallback platforms/, "settings.update.platformNotFound"],
  [/error sending request|dns error|timed out/, "settings.update.connectFailed"],
];

export function updateErrorKey(raw: string, fallback: MessageKey): MessageKey {
  return KNOWN_ERRORS.find(([pattern]) => pattern.test(raw))?.[1] ?? fallback;
}

/**
 * Checks for a new version without downloading anything. `silent` is for the once-per-launch
 * background check (see `+layout.svelte`): a failure there (e.g. no network yet at cold boot)
 * resets to idle instead of surfacing as an "error" state the Settings page would otherwise
 * show the next time it's opened, for a check the user never asked for.
 */
export async function checkForUpdate({
  silent = false,
}: { silent?: boolean } = {}): Promise<Update | null> {
  updateState.set({ status: "checking", update: null });
  try {
    const update = await check(import.meta.env.DEV ? devCheckOptions() : undefined);
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
    const raw = String(e);
    // A loading toast resolved in the same tick, because that is the only way to attach
    // `output` — which is what makes the original text readable and copyable from the toast.
    const id = pushToast("");
    resolveToast(id, "error", get(t)(updateErrorKey(raw, "settings.update.installFailed")), raw);
  }
}
