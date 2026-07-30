import { showToast } from "$lib/stores/toasts";

function truncate(text: string, max = 60): string {
  return text.length > max ? `${text.slice(0, max)}...` : text;
}

/** Copies to the clipboard and shows a toast confirming success/failure. */
export async function copyToClipboard(value: string) {
  try {
    await navigator.clipboard.writeText(value);
    showToast("success", `${truncate(value)} をコピーしました`);
  } catch (e) {
    showToast("error", `コピーに失敗しました: ${String(e)}`);
  }
}
