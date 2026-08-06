<script lang="ts">
  import { formatError } from "$lib/errors";
  import { onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { startAttachSession, ptyWrite, ptyResize, ptyClose } from "$lib/ipc/pty";
  import { XtermController } from "$lib/xterm/XtermController";
  import Icon from "$lib/components/ui/Icon.svelte";
  import TerminalSearchBar from "$lib/components/terminal/TerminalSearchBar.svelte";
  import { t } from "$lib/stores/i18n";
  import dismissCircleIcon from "@fluentui/svg-icons/icons/dismiss_circle_16_regular.svg?raw";

  // The parent wraps us in `{#key containerId}`, so one component instance only ever
  // handles a single containerId for its whole lifetime (mount → destroy), rather than
  // being reused across container switches. That's what makes the plain "start once on
  // mount" shape below correct/sufficient.
  //
  // `hidden` is how the panel switches away from the Terminal tab without losing the
  // session — the same arrangement WslShellDialog uses. The component stays mounted and
  // only its root gets `display: none`, so the pty, the scrollback and both event
  // listeners survive. Unmounting instead (which is what the tab chain used to do) killed
  // the session outright, dropping whatever the user was in the middle of.
  let { containerId, hidden = false }: { containerId: string; hidden?: boolean } = $props();

  let ended = $state(false);
  let errorMessage = $state<string | null>(null);
  let searchOpen = $state(false);

  const controller = new XtermController({ interactive: true });
  controller.onSearchRequested(() => (searchOpen = true));

  let sessionId: string | null = null;
  let unlistenData: UnlistenFn | null = null;
  let unlistenExit: UnlistenFn | null = null;

  // Set in onDestroy — see LogViewer.svelte's `destroyed` for why this is needed: a
  // `startSession` call still in flight at teardown must not adopt its session
  // afterwards, or it leaks an interactive `wsl.exe`/PTY child process that nothing can
  // close from the UI anymore.
  let destroyed = false;

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
    controller.clear();
    ended = false;
    errorMessage = null;

    let newSessionId: string;
    try {
      newSessionId = await startAttachSession(id, controller.cols, controller.rows);
    } catch (e) {
      if (!destroyed) errorMessage = formatError(e);
      return;
    }

    if (destroyed) {
      void ptyClose(newSessionId);
      return;
    }
    sessionId = newSessionId;

    const dataUnlisten = await listen<string>(`pty:${newSessionId}:data`, (event) => {
      controller.write(event.payload);
    });
    const exitUnlisten = await listen(`pty:${newSessionId}:exit`, () => {
      ended = true;
    });

    if (destroyed) {
      dataUnlisten();
      exitUnlisten();
      void ptyClose(newSessionId);
      return;
    }
    unlistenData = dataUnlisten;
    unlistenExit = exitUnlisten;
  }

  function mountTerminal(el: HTMLDivElement) {
    controller.onData((data) => {
      if (sessionId) void ptyWrite(sessionId, data);
    });

    controller.onResize(({ cols, rows }) => {
      if (sessionId) void ptyResize(sessionId, cols, rows);
    });

    // Waits for the initial fit so the PTY (and full-screen apps like vim that read its
    // size once at startup) gets the real panel dimensions instead of xterm's 80x24
    // default — starting at the wrong size, with nothing to correct it until the next
    // resize, is what made vim render so badly broken.
    void controller.mount(el).then(() => {
      terminalMounted = true;
      return startSession(containerId);
    });
  }

  let terminalMounted = $state(false);

  // Runs when the terminal first appears and again every time it's un-hidden.
  //
  // A hidden element has no layout box, so `fit()` deliberately bails out (see
  // XtermController.fit — measuring one anyway is actively destructive). That means any
  // window or panel resize that happened while another tab was showing never reached the
  // terminal or the pty, and has to be picked up on the way back in.
  //
  // No `focus()` here, unlike WslShellDialog: that one is a dialog the user often opens by
  // keyboard shortcut, where landing anywhere else is useless. A tab is switched to by
  // clicking or arrowing onto the tab itself, and yanking focus into the terminal from
  // under that is not what was asked for.
  $effect(() => {
    if (hidden || !terminalMounted) return;
    // Same reason `mount()` defers its first fit: `display` has only just changed, and
    // measuring in this tick can still read the pre-layout size.
    requestAnimationFrame(() => controller.fit());
  });

  onDestroy(() => {
    destroyed = true;
    void stopCurrentSession();
    controller.dispose();
  });
</script>

<div class="terminal-session" class:hidden>
  {#if errorMessage}
    <div class="term-banner error">{errorMessage}</div>
  {:else if ended}
    <div class="term-banner">
      <Icon svg={dismissCircleIcon} size={14} />
      <span>{$t("terminal.sessionEnded")}</span>
    </div>
  {/if}
  <div class="term-host-outer">
    {#if searchOpen}
      <TerminalSearchBar {controller} onClose={() => (searchOpen = false)} />
    {/if}
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

  /* `display: none` rather than the parent unmounting us — see the `hidden` prop. This
     also takes the terminal out of the tab order while it's away, so a hidden session
     can't be typed into by accident. */
  .terminal-session.hidden {
    display: none;
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
    position: relative;
    flex: 1;
    min-height: 0;
    padding: 12px;
  }

  .term-host {
    height: 100%;
    width: 100%;
  }
</style>
