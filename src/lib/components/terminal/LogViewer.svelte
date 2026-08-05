<script lang="ts">
  import { formatError } from "$lib/errors";
  import { onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { streamLogs, streamComposeLogs, stopLogStream } from "$lib/ipc/logs";
  import { XtermController } from "$lib/xterm/XtermController";
  import Icon from "$lib/components/ui/Icon.svelte";
  import TerminalSearchBar from "$lib/components/terminal/TerminalSearchBar.svelte";
  import { t } from "$lib/stores/i18n";
  import dismissCircleIcon from "@fluentui/svg-icons/icons/dismiss_circle_16_regular.svg?raw";
  import textWrapIcon from "@fluentui/svg-icons/icons/text_wrap_16_regular.svg?raw";
  import textWrapOffIcon from "@fluentui/svg-icons/icons/text_wrap_off_16_regular.svg?raw";

  // The parent wraps us in `{#key containerId}` (or `{#key project}` for the Compose
  // Logs tab), so one component instance only ever handles a single target for its
  // whole lifetime (mount → destroy), rather than being reused across switches.
  // Exactly one of `containerId` or `project` is set: `containerId` streams a single
  // container's `docker logs -f`, `project` streams every service in a Compose project
  // via `docker compose logs -f` (each line prefixed with its service name).
  let {
    containerId,
    project,
    configFiles,
    isRunning,
  }: {
    containerId?: string;
    project?: string;
    configFiles?: string[];
    isRunning: boolean;
  } = $props();

  let ended = $state(false);
  let errorMessage = $state<string | null>(null);
  let wrapEnabled = $state(true);
  let searchOpen = $state(false);
  let outerEl = $state<HTMLDivElement | undefined>();

  const controller = new XtermController();
  controller.onSearchRequested(() => (searchOpen = true));

  $effect(() => {
    controller.setWrapEnabled(wrapEnabled);
    // Turning wrapping back on drops `overflow-x`, which forces `scrollLeft` to 0 —
    // but not necessarily via a `scroll` event, so `--h-scroll` (below) could keep a
    // stale offset and mis-place the overlays the moment wrapping is turned off again.
    if (wrapEnabled) outerEl?.style.setProperty("--h-scroll", "0px");
  });

  // Everything painted "on top of" the terminal — xterm's own vertical scrollbar and our
  // search bar — is a `position: absolute` element *inside* this scroll container, so it
  // is anchored to the content, not to the scrollport. In no-wrap mode the terminal
  // buffer is deliberately far wider than the container (see XtermController's `fit()`),
  // and scrolling right slides those overlays off to the left along with the log text —
  // the vertical scrollbar visibly drifts away from the right edge instead of staying
  // pinned to it. (It only *looks* like a streaming-only bug because xterm's scrollbar
  // uses VS Code's auto-hide behaviour: it fades out 500ms after the last scroll, so on a
  // static log it is simply invisible while you drag horizontally. Nothing is touching
  // `scrollLeft` — the scroll position is fine, the overlays are just riding along.)
  //
  // Publishing the offset as a custom property lets the CSS below cancel it out with a
  // counter-`translate`, which never touches `scrollLeft` and so can't fight genuine user
  // scrolling. The counter-translate can't extend the scrollable area either: it shifts
  // the overlays right by at most `scrollLeft`, keeping their right edge at
  // `contentWidth + scrollLeft`, which stays inside the (much wider) buffer's own
  // `scrollWidth`.
  function syncScrollOffset(event: Event & { currentTarget: HTMLDivElement }) {
    const el = event.currentTarget;
    el.style.setProperty("--h-scroll", `${el.scrollLeft}px`);
  }

  let streamId: string | null = null;
  let unlistenData: UnlistenFn | null = null;
  let unlistenEnd: UnlistenFn | null = null;

  // Set in onDestroy. Without checking this, a `startStream` call still in flight when
  // the component is torn down (e.g. the user switches tabs right after it started)
  // would resolve *after* teardown and go on to adopt the stream/listeners anyway —
  // leaking the backend `docker logs -f` process forever, since nothing would be left
  // to stop it, and feeding an already-disposed terminal.
  let destroyed = false;

  async function stopCurrentStream() {
    unlistenData?.();
    unlistenEnd?.();
    unlistenData = null;
    unlistenEnd = null;
    if (streamId) {
      const id = streamId;
      streamId = null;
      try {
        await stopLogStream(id);
      } catch {
        // Best-effort: the process may have already exited on its own.
      }
    }
  }

  async function startStream() {
    // Not only for the first call: the `isRunning` false→true effect below restarts the
    // stream on its own, and a container that flaps (a restart, or two event-driven
    // refreshes in quick succession) can reach here while the previous `docker logs -f`
    // is still alive. Overwriting `streamId` and the unlisten handles without this would
    // strand that process — nothing else holds its id — and leave its listener attached,
    // so two streams would write into the same terminal.
    await stopCurrentStream();

    controller.clear();
    ended = false;
    errorMessage = null;

    let newStreamId: string;
    try {
      newStreamId = containerId
        ? await streamLogs(containerId)
        : await streamComposeLogs(project!, configFiles ?? []);
    } catch (e) {
      if (!destroyed) errorMessage = formatError(e);
      return;
    }

    if (destroyed) {
      void stopLogStream(newStreamId);
      return;
    }
    streamId = newStreamId;

    const dataUnlisten = await listen<string[]>(`logs:${newStreamId}`, (event) => {
      controller.writeLines(event.payload);
    });
    const endUnlisten = await listen(`logs:${newStreamId}:end`, () => {
      ended = true;
    });

    if (destroyed) {
      dataUnlisten();
      endUnlisten();
      void stopLogStream(newStreamId);
      return;
    }
    unlistenData = dataUnlisten;
    unlistenEnd = endUnlisten;
  }

  function mountTerminal(el: HTMLDivElement) {
    controller.mount(el);
    void startStream();
  }

  // `docker logs -f`'s process exits with the container, ending the stream (see
  // `ended` above) — nothing then restarts it on its own if the same container starts
  // back up later while this tab stays open, since `startStream` above only ever runs
  // once at mount. Watches for the false → true transition specifically (not just
  // "isRunning is true"), so this doesn't also fire on the initial mount, where
  // `startStream` above already has it covered.
  let wasRunning = false;
  let firstRun = true;
  $effect(() => {
    const running = isRunning;
    if (!firstRun && running && !wasRunning) {
      void startStream();
    }
    wasRunning = running;
    firstRun = false;
  });

  onDestroy(() => {
    destroyed = true;
    void stopCurrentStream();
    controller.dispose();
  });
</script>

<div class="log-viewer">
  <div class="toolbar">
    <button
      class="icon-btn"
      class:active={!wrapEnabled}
      title={wrapEnabled ? $t("terminal.wrapOff") : $t("terminal.wrapOn")}
      aria-label={wrapEnabled ? $t("terminal.wrapOffAriaLabel") : $t("terminal.wrapOnAriaLabel")}
      onclick={() => (wrapEnabled = !wrapEnabled)}
    >
      <Icon svg={wrapEnabled ? textWrapIcon : textWrapOffIcon} size={15} />
    </button>
  </div>
  {#if errorMessage}
    <div class="log-banner error">{errorMessage}</div>
  {:else if ended}
    <div class="log-banner">
      <Icon svg={dismissCircleIcon} size={14} />
      <span>{$t("logs.ended")}</span>
    </div>
  {/if}
  <div
    class="term-host-outer"
    class:no-wrap={!wrapEnabled}
    bind:this={outerEl}
    onscroll={syncScrollOffset}
  >
    {#if searchOpen}
      <TerminalSearchBar {controller} onClose={() => (searchOpen = false)} />
    {/if}
    <div class="term-host" use:mountTerminal></div>
  </div>
</div>

<style>
  .log-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 300px;
  }

  .toolbar {
    display: flex;
    justify-content: flex-end;
    padding: 4px 6px;
    border-bottom: 1px solid var(--dockl-border);
    flex-shrink: 0;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    /* A <button>'s UA padding (1px 6px) is subtracted from the fixed width by
       `box-sizing: border-box`, and as a flex item the icon then shrinks to the leftover
       content box rather than overflowing it — squashing it horizontally. */
    padding: 0;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--dockl-text-secondary);
    cursor: pointer;
  }

  .icon-btn:hover {
    background: var(--dockl-surface-hover);
    color: var(--dockl-text-primary);
  }

  .icon-btn.active {
    background: var(--dockl-surface-hover);
    color: var(--dockl-accent);
  }

  .log-banner {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    font-size: 12px;
    color: var(--dockl-text-secondary);
    border-bottom: 1px solid var(--dockl-border);
    flex-shrink: 0;
  }

  .log-banner.error {
    color: var(--dockl-danger);
  }

  /*
   * The padding lives on this outer element rather than on `.term-host` itself.
   * xterm's FitAddon sizes the terminal from `.term-host`'s clientHeight, which (per
   * the CSS box model) includes an element's own padding — so padding directly on the
   * element passed to `terminal.open()` makes FitAddon overestimate available rows,
   * clipping the last line. Padding here instead just shrinks the content-box that
   * `.term-host` fills, which FitAddon measures correctly.
   */
  .term-host-outer {
    position: relative;
    flex: 1;
    min-height: 0;
    padding: 12px;
  }

  /* Wrap-off intentionally makes the terminal buffer wider than the container (see
     XtermController's `fit()`) — this is what lets the user actually reach the rest of
     a long line instead of just clipping it. */
  .term-host-outer.no-wrap {
    overflow-x: auto;
  }

  /* Pins the overlays that are anchored to this scroll container's *content* back to its
     *viewport* — see `syncScrollOffset` for why they drift without this. `translate` (not
     `transform`) so it composes with, rather than clobbers, any transform these elements
     set themselves: xterm layer-hints its scrollbar `slider` with a `translate3d`, and
     only the `.scrollbar` wrapper is retargeted here, but that stays true either way.
     Scoped to `.no-wrap` because that's the only mode where this element scrolls at all. */
  .term-host-outer.no-wrap :global(.xterm-scrollable-element > .scrollbar),
  .term-host-outer.no-wrap :global(.search-bar) {
    translate: var(--h-scroll, 0px);
  }

  .term-host {
    height: 100%;
    width: 100%;
  }
</style>
