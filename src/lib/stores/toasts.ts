import { writable } from "svelte/store";

export type ToastStatus = "loading" | "success" | "error";

export interface Toast {
  id: string;
  status: ToastStatus;
  message: string;
  /** Command output to show in a details modal when the toast is clicked, if any. */
  output?: string;
  /** Total ms until auto-dismiss, set only when a dismiss timer is actually scheduled — lets
   *  the toast render a countdown indicator matching its real timer instead of a guessed one. */
  duration?: number;
}

export const toasts = writable<Toast[]>([]);

let counter = 0;

/** Caps how many toasts stack up at once so a burst of actions doesn't bury the newest one. */
const MAX_TOASTS = 5;

/** Tracks each toast's pending auto-dismiss timer so it can be paused (e.g. on hover) and
 *  resumed later without losing track of how much time was left. */
const timers = new Map<
  string,
  { timeoutId: ReturnType<typeof setTimeout>; remaining: number; startedAt: number }
>();

function clearTimer(id: string) {
  const timer = timers.get(id);
  if (!timer) return;
  clearTimeout(timer.timeoutId);
  timers.delete(id);
}

/**
 * Appends a toast, evicting the oldest non-loading toast if that would push the stack over
 * the cap. Loading toasts are never evicted since they track an in-flight action that still
 * needs to be resolved.
 */
function addToast(toast: Toast) {
  toasts.update((list) => {
    const next = [...list, toast];
    if (next.length <= MAX_TOASTS) return next;
    const evictIndex = next.findIndex((t) => t.status !== "loading");
    if (evictIndex === -1) return next;
    clearTimer(next[evictIndex].id);
    return next.filter((_, i) => i !== evictIndex);
  });
}

function scheduleDismiss(id: string, delay: number) {
  const timeoutId = setTimeout(() => {
    timers.delete(id);
    dismissToast(id);
  }, delay);
  timers.set(id, { timeoutId, remaining: delay, startedAt: Date.now() });
}

/** Freezes a toast's auto-dismiss countdown. No-op if the toast has none (loading/output toasts). */
export function pauseToastTimer(id: string) {
  const timer = timers.get(id);
  if (!timer) return;
  clearTimeout(timer.timeoutId);
  timer.remaining -= Date.now() - timer.startedAt;
}

/** Resumes a previously paused countdown from where it left off. */
export function resumeToastTimer(id: string) {
  const timer = timers.get(id);
  if (!timer) return;
  timer.startedAt = Date.now();
  timer.timeoutId = setTimeout(() => {
    timers.delete(id);
    dismissToast(id);
  }, timer.remaining);
}

/** Shows a "loading" toast and returns its id so it can be resolved later. */
export function pushToast(message: string): string {
  const id = `toast-${++counter}`;
  addToast({ id, status: "loading", message });
  return id;
}

/**
 * Turns a loading toast into success/error, then auto-dismisses it. `output` (e.g. a
 * compose command's own stdout/stderr) makes the toast clickable to view it in a modal,
 * and — since the user might click it to read *after* the auto-dismiss timer would have
 * fired — keeps the toast on screen until explicitly closed instead of dismissing it.
 */
export function resolveToast(
  id: string,
  status: "success" | "error",
  message: string,
  output?: string,
) {
  if (output) {
    toasts.update((list) => list.map((t) => (t.id === id ? { ...t, status, message, output } : t)));
    return;
  }
  const delay = status === "success" ? 3000 : 6000;
  toasts.update((list) =>
    list.map((t) => (t.id === id ? { ...t, status, message, duration: delay } : t)),
  );
  scheduleDismiss(id, delay);
}

export function dismissToast(id: string) {
  clearTimer(id);
  toasts.update((list) => list.filter((t) => t.id !== id));
}

/** For instant, synchronous actions (e.g. copy) that don't need a "loading" phase. */
export function showToast(status: "success" | "error", message: string): string {
  const id = `toast-${++counter}`;
  const delay = status === "success" ? 2000 : 6000;
  addToast({ id, status, message, duration: delay });
  scheduleDismiss(id, delay);
  return id;
}
