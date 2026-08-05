import { formatError } from "$lib/errors";
import { get } from "svelte/store";

import { t } from "$lib/stores/i18n";
import { showToast } from "$lib/stores/toasts";

function truncate(text: string, max = 60): string {
  return text.length > max ? `${text.slice(0, max)}...` : text;
}

/**
 * Copies without announcing success, reporting only failures as a toast. Returns whether
 * it worked so the caller can show its own inline confirmation.
 *
 * This is what every on-screen copy *button* uses (via `CopyIconButton`, which flips to a
 * checkmark): the button is right where the user is looking, so a toast on top of it would
 * be redundant. `copyToClipboard` below is for copies triggered with no button to light up.
 */
export async function copySilently(value: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(value);
    return true;
  } catch (e) {
    showToast("error", get(t)("errors.copyFailed", { error: formatError(e) }));
    return false;
  }
}

/**
 * Copies and confirms with a toast. For copies with no visible button to give feedback on
 * — the terminal's Ctrl+C / right-click, and the right-click "コピー" context menu, which
 * has already closed by the time the copy lands.
 */
export async function copyToClipboard(value: string) {
  if (await copySilently(value)) {
    showToast("success", get(t)("common.copied", { value: truncate(value) }));
  }
}
