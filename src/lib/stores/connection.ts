import { writable } from "svelte/store";

/**
 * `starting` is split out from `connecting` because the two differ by an order of
 * magnitude in how long they legitimately take: reaching a distro that's already up is
 * near-instant, while booting a stopped one takes tens of seconds. Shown under one label
 * they'd be indistinguishable, and a cold start would look identical to a hang.
 *
 * `failed` is the timeout case — WSL never answered. It's separate from `disconnected`
 * (nothing set up yet, which sends the user to /setup) because it's retryable in place.
 *
 * `stopped` means the saved distro isn't running and we deliberately haven't touched it:
 * any command into a stopped distro boots the whole VM, which is not something launching
 * a GUI should do behind the user's back. It waits for an explicit start instead.
 */
export type ConnectionStatus =
  "connecting" | "starting" | "connected" | "disconnected" | "failed" | "stopped";

export interface ConnectionState {
  status: ConnectionStatus;
  distro: string | null;
  /** Only set when `status` is `failed`: the error to show alongside the retry button. */
  error?: string;
}

/**
 * App-wide WSL2/Docker connection state, checked once at startup by the root layout
 * (see +layout.svelte) rather than separately by every route — each route previously
 * re-ran its own `docker ps`-based connection check on every navigation.
 */
export const connection = writable<ConnectionState>({ status: "connecting", distro: null });
