/**
 * Builds the `keydown` handler that runs `refresh` on F5, for `<svelte:window>`:
 *
 * ```svelte
 * <svelte:window onkeydown={f5RefreshHandler(refresh)} />
 * ```
 *
 * This is what keeps a manual refresh reachable without a mouse: the header's refresh
 * button is deliberately outside the tab order (see PageHeader).
 *
 * A handler factory rather than an `$effect` that calls `addEventListener` itself —
 * `<svelte:window>` is how Svelte wants global listeners registered, and it's what the
 * rest of the app already uses. It also means the listener's lifetime is the component's,
 * with nothing to remember to tear down.
 *
 * `preventDefault` is the important part. The webview treats F5 as "reload the page",
 * which in a Tauri app throws away everything the SPA is holding — the connection state
 * the root layout established, the current selection, scroll position — to re-fetch one
 * list. Modified F5 (Ctrl+F5 and friends) is left alone rather than silently reinterpreted.
 */
export function f5RefreshHandler(refresh: () => void) {
  return (e: KeyboardEvent) => {
    if (e.key !== "F5" || e.ctrlKey || e.altKey || e.shiftKey || e.metaKey) return;
    e.preventDefault();
    refresh();
  };
}
