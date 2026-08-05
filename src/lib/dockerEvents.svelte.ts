import { listen } from "@tauri-apps/api/event";

/** Mirrors the Rust side's `DockerEvent` (`docker_bridge::events`). */
export interface DockerEvent {
  kind: "container" | "image" | "volume" | "network";
  action: string;
  id: string;
}

/**
 * Calls `onChange` (debounced) whenever a `docker:event` of one of `kinds` arrives —
 * the backend's own `docker events` subscription, already filtered down to events that
 * plausibly change a list (see `docker_bridge::events::is_relevant`).
 *
 * Debounced because a single Compose `up`/`down` fires a dozen+ related events within a
 * couple of seconds (confirmed against real `docker events` output), which would
 * otherwise mean a dozen+ redundant refreshes for one user action.
 *
 * Returns an unlisten function — call it (e.g. from `onDestroy`) to stop watching.
 */
export async function watchDockerEvents(
  kinds: DockerEvent["kind"][],
  onChange: () => void,
  debounceMs = 400,
): Promise<() => void> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const unlisten = await listen<DockerEvent>("docker:event", (event) => {
    if (!kinds.includes(event.payload.kind)) return;
    clearTimeout(timer);
    timer = setTimeout(onChange, debounceMs);
  });
  return () => {
    clearTimeout(timer);
    unlisten();
  };
}

/**
 * Runs `refresh()` once the app's connected, then keeps it in sync via `watchDockerEvents`
 * — the "wait for connection, refresh, subscribe, unsubscribe on destroy" block every
 * list route (containers/images/volumes/networks/storage) used to hand-roll separately.
 * Call once at the top of a route's `<script>`, passing e.g. `() => $connection.status
 * === "connected"` — a getter rather than the boolean itself, since the `$connection`
 * auto-subscription sigil this needs to read only works inside a `.svelte` file's own
 * compiled scope, not here; calling the getter from inside this module's `$effect` still
 * tracks the dependency correctly (Svelte's reactivity follows the read, not the file).
 *
 * Also closes a race the hand-rolled version had: `watchDockerEvents`'s `await listen(...)`
 * doesn't resolve synchronously, so if the component were destroyed first, the listener
 * it had *just* registered would never learn to unsubscribe — nothing kept the resulting
 * `unwatch` function around to call, since it only exists once that `.then()` runs. This
 * tracks that with a `cancelled` flag scoped to each effect run, so a listener that
 * finishes registering after teardown gets stopped immediately instead of leaking for
 * the rest of the app's lifetime.
 */
export function refreshOnDockerEvents(
  isConnected: () => boolean,
  kinds: DockerEvent["kind"][],
  refresh: () => void,
) {
  $effect(() => {
    if (!isConnected()) return;

    let cancelled = false;
    let unwatch: (() => void) | undefined;

    refresh();
    void watchDockerEvents(kinds, refresh).then((stop) => {
      if (cancelled) {
        stop();
      } else {
        unwatch = stop;
      }
    });

    return () => {
      cancelled = true;
      unwatch?.();
    };
  });
}
