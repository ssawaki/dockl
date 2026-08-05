<script lang="ts">
  import { formatError } from "$lib/errors";
  import { onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { trapFocus } from "$lib/actions/trapFocus";
  import { startWslShellSession, ptyWrite, ptyResize, ptyClose } from "$lib/ipc/pty";
  import { XtermController } from "$lib/xterm/XtermController";
  import Icon from "$lib/components/ui/Icon.svelte";
  import TerminalSearchBar from "$lib/components/terminal/TerminalSearchBar.svelte";
  import { t } from "$lib/stores/i18n";
  import dismissIcon from "@fluentui/svg-icons/icons/dismiss_20_regular.svg?raw";
  import dismissCircleIcon from "@fluentui/svg-icons/icons/dismiss_circle_16_regular.svg?raw";

  // The caller renders us behind an `{#if}`, so one component instance only ever covers
  // a single shell session for its whole lifetime (mount -> destroy), same as
  // TerminalSession.svelte's `{#key containerId}` guarantee.
  //
  // `startSession` lets a caller run something other than a bare interactive shell as the
  // pty's own child process, rather than opening a shell and typing a command into it
  // afterwards. That "type it in after" approach races the shell becoming ready to read,
  // which is how a `sudo` prompt ends up swallowing input meant for the shell — spawning
  // the target directly has no such gap. Defaults to the plain "open a WSL shell"
  // behavior; no caller currently overrides it (the TCP setup script that did was
  // removed, see TcpEndpointDialog).
  // `hidden` is how the caller "closes" the dialog without losing the session: the
  // component stays mounted and only its root gets `display: none`, so the pty, the
  // terminal's scrollback and both event listeners all survive until the caller really
  // tears us down. `onEnded` fires when the pty's own process exits — at that point
  // there's nothing left worth preserving, which is the caller's cue to drop us.
  // Callers that leave `onEnded` unset keep the dialog up with the "session ended" banner
  // instead (TcpBridgeSetupDialog needs that: its script's final output is the result).
  let {
    onClose,
    title,
    hidden = false,
    onEnded,
    startSession = startWslShellSession,
  }: {
    onClose: () => void;
    title?: string;
    hidden?: boolean;
    onEnded?: () => void;
    startSession?: (cols: number, rows: number) => Promise<string>;
  } = $props();

  let displayTitle = $derived(title ?? $t("wslShell.title"));

  let ended = $state(false);
  let errorMessage = $state<string | null>(null);
  let searchOpen = $state(false);

  const controller = new XtermController({ interactive: true });
  controller.onSearchRequested(() => (searchOpen = true));

  let sessionId: string | null = null;
  let unlistenData: UnlistenFn | null = null;
  let unlistenExit: UnlistenFn | null = null;

  // See TerminalSession.svelte's `destroyed` for why this is needed: a `startSession`
  // call still in flight when the dialog closes must not adopt its session afterwards.
  let destroyed = false;

  async function stopSession() {
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

  async function connectSession() {
    let newSessionId: string;
    try {
      newSessionId = await startSession(controller.cols, controller.rows);
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
      // The process is gone and Rust has already dropped the session, but the terminal
      // stays focused and keeps accepting keystrokes — each of which would otherwise fire
      // a `pty_write` at an id the backend no longer knows, and get back an error. Same
      // for the `pty_resize` that the banner appearing below triggers via the resize
      // observer. Dropping the id here makes both paths quietly no-op.
      sessionId = null;
      onEnded?.();
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

    // See TerminalSession.svelte's mountTerminal: waiting for the initial fit avoids
    // starting the PTY at xterm's 80x24 default, which is what broke full-screen apps
    // like vim.
    void controller.mount(el).then(() => {
      terminalMounted = true;
      return connectSession();
    });
  }

  let terminalMounted = $state(false);

  // Runs when the terminal first appears and again every time it's un-hidden.
  //
  // `fit()` because a hidden element has no layout box for FitAddon to measure: it bails
  // out, so any window resize that happened while we were away never reached the terminal
  // (or the pty) and has to be picked up on the way back in.
  //
  // `focus()` because `trapFocus` only constrains Tab, it doesn't focus anything itself —
  // without this the dialog appears with focus still on <body> and the first keystrokes go
  // nowhere. Matters most for the Ctrl+` shortcut (see Titlebar.svelte), where the user
  // never touches the mouse; the terminal is the only thing here worth focusing anyway.
  $effect(() => {
    if (hidden || !terminalMounted) return;
    // Same reason `mount()` defers its first fit: `display` has only just changed, and
    // measuring in this tick can still read the pre-layout size.
    requestAnimationFrame(() => {
      controller.fit();
      controller.focus();
    });
  });

  function handleKeydown(e: KeyboardEvent) {
    // This listener stays live while hidden — the component is still mounted.
    if (hidden) return;
    if (e.key === "Escape") onClose();
  }

  onDestroy(() => {
    destroyed = true;
    void stopSession();
    controller.dispose();
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" class:hidden onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="dialog dockl-surface"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    use:trapFocus
    onclick={(e) => e.stopPropagation()}
  >
    <div class="dialog-header">
      <span class="dialog-title">{displayTitle}</span>
      <button
        class="icon-btn"
        title={$t("common.close")}
        aria-label={$t("common.close")}
        onclick={onClose}
      >
        <Icon svg={dismissIcon} size={16} />
      </button>
    </div>
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
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 3000;
  }

  /* `display: none` rather than `{#if}` in the caller — see the `hidden` prop's comment.
     It also takes the whole subtree out of the accessibility tree and out of Tab order,
     so a hidden session can't be reached by keyboard either. */
  .backdrop.hidden {
    display: none;
  }

  .dialog {
    background: var(--dockl-menu-bg);
    width: min(880px, 90vw);
    height: min(560px, 80vh);
    display: flex;
    flex-direction: column;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-bottom: 1px solid var(--dockl-border);
    flex-shrink: 0;
  }

  .dialog-title {
    font-size: 13px;
    font-weight: 600;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    /* See the note in LogViewer.svelte: without this the UA's button padding eats the
       fixed width and the icon shrinks to fit what's left. */
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
