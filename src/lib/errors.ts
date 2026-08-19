import { get } from "svelte/store";
import { t, type MessageKey } from "$lib/stores/i18n";

/**
 * The shape every Tauri command error arrives in — see `src-tauri/src/error.rs`, which is
 * the only thing that produces it.
 */
interface TauriError {
  code: string;
  message: string;
  params?: Record<string, string>;
}

/**
 * Errors this app can say something more useful about than the underlying tool did.
 *
 * Deliberately an explicit list rather than "look up `errors.${code}` and hope": a code
 * with no entry here falls back to the Rust-side message, which is the right outcome for
 * `command_failed` / `parse_error` / `io`, whose text *is* the raw output of docker or
 * WSL. Wrapping those in a translated sentence would add words without adding meaning.
 */
const TRANSLATED: Record<string, MessageKey> = {
  connect_timeout: "errors.connectTimeout",
  not_configured: "errors.notConfigured",
  wsl_unavailable: "errors.wslUnavailable",
  no_distro_found: "errors.noDistroFound",
  distro_stopped: "errors.distroStopped",
};

function isTauriError(e: unknown): e is TauriError {
  return typeof e === "object" && e !== null && "code" in e && "message" in e;
}

/**
 * Turns anything caught from an `invoke` into text to show the user.
 *
 * Use this instead of `String(e)` wherever a caught error reaches the screen. Errors that
 * don't come from Rust at all (a clipboard rejection, say) pass through unchanged, so it
 * is always safe to apply.
 */
export function formatError(e: unknown): string {
  if (!isTauriError(e)) return String(e);

  const key = TRANSLATED[e.code];
  if (!key) return e.message;
  return get(t)(key, e.params ?? {});
}
