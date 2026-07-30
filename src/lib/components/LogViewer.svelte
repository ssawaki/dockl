<script lang="ts">
  import { onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { streamLogs, stopLogStream } from "$lib/ipc/logs";
  import { XtermController } from "$lib/xterm/XtermController";
  import Icon from "$lib/components/Icon.svelte";
  import dismissCircleIcon from "@fluentui/svg-icons/icons/dismiss_circle_16_regular.svg?raw";

  let { containerId }: { containerId: string } = $props();

  let termEl: HTMLDivElement | undefined = $state();
  let ended = $state(false);
  let errorMessage = $state<string | null>(null);

  const controller = new XtermController();

  let streamId: string | null = null;
  let unlistenData: UnlistenFn | null = null;
  let unlistenEnd: UnlistenFn | null = null;

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

  async function startStream(id: string) {
    await stopCurrentStream();
    controller.clear();
    ended = false;
    errorMessage = null;

    try {
      const newStreamId = await streamLogs(id);
      streamId = newStreamId;
      unlistenData = await listen<string>(`logs:${newStreamId}`, (event) => {
        controller.writeLine(event.payload);
      });
      unlistenEnd = await listen(`logs:${newStreamId}:end`, () => {
        ended = true;
      });
    } catch (e) {
      errorMessage = String(e);
    }
  }

  $effect(() => {
    if (termEl && containerId) {
      void startStream(containerId);
    }
  });

  function mountTerminal(el: HTMLDivElement) {
    termEl = el;
    controller.mount(el);
  }

  onDestroy(() => {
    void stopCurrentStream();
    controller.dispose();
  });
</script>

<div class="log-viewer">
  {#if errorMessage}
    <div class="log-banner error">{errorMessage}</div>
  {:else if ended}
    <div class="log-banner">
      <Icon svg={dismissCircleIcon} size={14} />
      <span>ログ出力が終了しました（コンテナが停止しました）</span>
    </div>
  {/if}
  <div class="term-host-outer">
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
    flex: 1;
    min-height: 0;
    padding: 12px;
  }

  .term-host {
    height: 100%;
    width: 100%;
  }
</style>
