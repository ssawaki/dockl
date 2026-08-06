/**
 * Runs `refresh` when F5 is pressed, for as long as the calling component is alive.
 * Call once at the top of a route's `<script>`, the same way `refreshOnDockerEvents` is.
 *
 * This is what keeps a manual refresh reachable without a mouse: the header's refresh
 * button is deliberately outside the tab order (see PageHeader).
 *
 * `preventDefault` is the important part. The webview treats F5 as "reload the page",
 * which in a Tauri app throws away everything the SPA is holding — the connection state
 * the root layout established, the current selection, scroll position — to re-fetch one
 * list. Modified F5 (Ctrl+F5 and friends) is left alone rather than silently reinterpreted.
 */
export function refreshOnF5(refresh: () => void) {
  $effect(() => {
    function onKeydown(e: KeyboardEvent) {
      if (e.key !== "F5" || e.ctrlKey || e.altKey || e.shiftKey || e.metaKey) return;
      e.preventDefault();
      refresh();
    }

    window.addEventListener("keydown", onKeydown);
    return () => window.removeEventListener("keydown", onKeydown);
  });
}
