<script lang="ts">
  import { onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { startAttachSession, ptyWrite, ptyResize, ptyClose } from "$lib/ipc/pty";
  import { XtermController } from "$lib/xterm/XtermController";
  import Icon from "$lib/components/Icon.svelte";
  import dismissCircleIcon from "@fluentui/svg-icons/icons/dismiss_circle_16_regular.svg?raw";

  let { containerId }: { containerId: string } = $props();

  let ended = $state(false);
  let errorMessage = $state<string | null>(null);

  const controller = new XtermController({ interactive: true });

  let sessionId: string | null = null;
  let unlistenData: UnlistenFn | null = null;
  let unlistenExit: UnlistenFn | null = null;
  let disposeOnData: (() => void) | null = null;
  let disposeOnResize: (() => void) | null = null;

  async function stopCurrentSession() {
    unlistenData?.();
    unlistenExit?.();
    unlistenData = null;
    unlistenExit = null;
    if (sessionId) {
      const id = sessionId;
      sessionId = null;
      try {
        await ptyClose(id);
      } catch {
        // Best-effort: the process may have already exited on its own.
      }
    }
  }

  async function startSession(id: string) {
    await stopCurrentSession();
    controller.clear();
    ended = false;
    errorMessage = null;

    try {
      const newSessionId = await startAttachSession(id, controller.cols, controller.rows);
      sessionId = newSessionId;
      unlistenData = await listen<string>(`pty:${newSessionId}:data`, (event) => {
        controller.write(event.payload);
      });
      unlistenExit = await listen(`pty:${newSessionId}:exit`, () => {
        ended = true;
      });
    } catch (e) {
      errorMessage = String(e);
    }
  }

  function mountTerminal(el: HTMLDivElement) {
    controller.mount(el);

    disposeOnData = controller.onData((data) => {
      if (sessionId) void ptyWrite(sessionId, data);
    }).dispose;

    disposeOnResize = controller.onResize(({ cols, rows }) => {
      if (sessionId) void ptyResize(sessionId, cols, rows);
    }).dispose;

    void startSession(containerId);
  }

  onDestroy(() => {
    disposeOnData?.();
    disposeOnResize?.();
    void stopCurrentSession();
    controller.dispose();
  });
</script>

<div class="terminal-session">
  {#if errorMessage}
    <div class="term-banner error">{errorMessage}</div>
  {:else if ended}
    <div class="term-banner">
      <Icon svg={dismissCircleIcon} size={14} />
      <span>セッションが終了しました</span>
    </div>
  {/if}
  <div class="term-host-outer">
    <div class="term-host" use:mountTerminal></div>
  </div>
</div>

<style>
  .terminal-session {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 300px;
  }

  .term-banner {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    font-size: 12px;
    color: var(--dockl-text-secondary);
    border-bottom: 1px solid var(--dockl-border);
    flex-shrink: 0;
  }

  .term-banner.error {
    color: var(--dockl-danger);
  }

  /* See LogViewer.svelte for why the padding lives on the outer element rather than
     directly on `.term-host` (the one xterm's FitAddon measures). */
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
