import { writable } from "svelte/store";

export type ToastStatus = "loading" | "success" | "error";

export interface Toast {
  id: string;
  status: ToastStatus;
  message: string;
}

export const toasts = writable<Toast[]>([]);

let counter = 0;

/** Shows a "loading" toast and returns its id so it can be resolved later. */
export function pushToast(message: string): string {
  const id = `toast-${++counter}`;
  toasts.update((list) => [...list, { id, status: "loading", message }]);
  return id;
}

/** Turns a loading toast into success/error, then auto-dismisses it. */
export function resolveToast(id: string, status: "success" | "error", message: string) {
  toasts.update((list) => list.map((t) => (t.id === id ? { ...t, status, message } : t)));
  const delay = status === "success" ? 3000 : 6000;
  setTimeout(() => dismissToast(id), delay);
}

export function dismissToast(id: string) {
  toasts.update((list) => list.filter((t) => t.id !== id));
}

/** For instant, synchronous actions (e.g. copy) that don't need a "loading" phase. */
export function showToast(status: "success" | "error", message: string): string {
  const id = `toast-${++counter}`;
  toasts.update((list) => [...list, { id, status, message }]);
  const delay = status === "success" ? 2000 : 6000;
  setTimeout(() => dismissToast(id), delay);
  return id;
}
