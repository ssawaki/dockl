/**
 * Keeps Tab/Shift+Tab cycling within a modal dialog instead of escaping to whatever's
 * behind it, and restores focus to whatever had it before the dialog opened once it
 * closes (Cancel/Confirm/Escape/backdrop click all tear down the same node). Attach to
 * the dialog's content element: `<div class="dialog" use:trapFocus>`.
 */
const FOCUSABLE_SELECTOR = [
  "a[href]",
  "button:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  "fluent-button:not([disabled])",
  "fluent-switch:not([disabled])",
  "fluent-radio:not([disabled])",
  "fluent-checkbox:not([disabled])",
  '[tabindex]:not([tabindex="-1"])',
].join(", ");

export function trapFocus(node: HTMLElement) {
  // Captured synchronously at attach time, before this dialog's own auto-focus (e.g.
  // ConfirmDialog's queueMicrotask'd confirm-button focus) has a chance to run.
  const previouslyFocused = document.activeElement as HTMLElement | null;

  function focusableEls(): HTMLElement[] {
    return Array.from(node.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)).filter(
      (el) => el.offsetParent !== null,
    );
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key !== "Tab") return;
    const els = focusableEls();
    if (els.length === 0) return;

    const first = els[0];
    const last = els[els.length - 1];
    const active = document.activeElement as HTMLElement | null;

    // Also fires if focus somehow isn't inside the dialog at all (e.g. it opened
    // without anything explicitly focused yet) — Tab should still land inside it.
    if (e.shiftKey) {
      if (active === first || !active || !node.contains(active)) {
        e.preventDefault();
        last.focus();
      }
    } else if (active === last || !active || !node.contains(active)) {
      e.preventDefault();
      first.focus();
    }
  }

  node.addEventListener("keydown", onKeydown);

  return {
    destroy() {
      node.removeEventListener("keydown", onKeydown);
      // Restore focus to wherever it was before this dialog opened — otherwise closing
      // it leaves focus on <body> (or wherever the browser defaults to once the
      // focused element is torn down), rather than back on the button that opened it.
      if (previouslyFocused && document.contains(previouslyFocused)) {
        previouslyFocused.focus();
      }
    },
  };
}
