<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Icon from "$lib/components/ui/Icon.svelte";
  import WslShellDialog from "$lib/components/terminal/WslShellDialog.svelte";
  import { connection } from "$lib/stores/connection";
  import { t } from "$lib/stores/i18n";
  import { sidebarToggleExpanded, setSidebarToggleExpanded } from "$lib/stores/appearance";
  import subtractIcon from "@fluentui/svg-icons/icons/subtract_20_regular.svg?raw";
  import squareIcon from "@fluentui/svg-icons/icons/square_20_regular.svg?raw";
  import dismissIcon from "@fluentui/svg-icons/icons/dismiss_20_regular.svg?raw";
  import terminalIcon from "@fluentui/svg-icons/icons/window_console_20_regular.svg?raw";
  import panelLeftIcon from "@fluentui/svg-icons/icons/panel_left_20_regular.svg?raw";

  const appWindow = getCurrentWindow();

  // Closing the shell only hides it: the dialog stays mounted (see its `hidden` prop), so
  // the pty session, the scrollback and a half-typed command line all survive and
  // reopening drops you back into the same shell rather than a fresh one.
  let shellMounted = $state(false);
  let shellVisible = $state(false);

  function openShell() {
    shellMounted = true;
    shellVisible = true;
  }

  function hideShell() {
    shellVisible = false;
  }

  // `exit` (or the shell dying on its own) leaves nothing to come back to, so the dialog
  // goes away for real rather than lingering as a terminal that still takes focus and
  // still swallows keystrokes while being unable to do anything with them. Unmounting is
  // also what makes the next open a working shell in one keypress instead of two — the
  // first press would otherwise be spent dismissing the dead session.
  function endShell() {
    shellVisible = false;
    shellMounted = false;
  }

  function minimize() {
    appWindow.minimize();
  }

  function toggleMaximize() {
    appWindow.toggleMaximize();
  }

  function close() {
    appWindow.close();
  }

  function toggleSidebar() {
    void setSidebarToggleExpanded(!$sidebarToggleExpanded);
  }

  // Ctrl+` toggles the WSL shell, matching VS Code/Windows Terminal's terminal binding.
  // Shift is deliberately not excluded: on a JIS keyboard ` *is* Shift+@, so requiring
  // it to be up would make the shortcut unreachable there. That also means Ctrl+Shift+`
  // works on a US layout, which is VS Code's "new terminal" binding anyway.
  //
  // Handled in the capture phase (rather than alongside +layout.svelte's Ctrl+, in the
  // bubble phase) because the dialog this opens embeds an xterm terminal: once that has
  // focus, xterm's own textarea listener would see the keystroke first and forward it to
  // the pty. Capturing at the window lets us swallow it before that.
  function handleShellShortcut(e: KeyboardEvent) {
    if (e.key !== "`" || !e.ctrlKey || e.altKey || e.metaKey) return;
    e.preventDefault();
    e.stopPropagation();
    if (shellVisible) {
      hideShell();
      return;
    }
    // Every modal in the app carries these attributes, so this also covers ConfirmDialog
    // and the setup dialogs — stacking a shell on top of one of those would leave two
    // competing focus traps, and closing the shell would drop focus back into a dialog the
    // user can no longer see the whole of. Rendered-ness is checked rather than just
    // existence because a hidden-but-mounted shell session matches the selector too, and
    // that one must not block itself from being reopened.
    const modals = Array.from(
      document.querySelectorAll<HTMLElement>('[role="dialog"][aria-modal="true"]'),
    );
    if (modals.some((el) => el.getClientRects().length > 0)) return;
    openShell();
  }
</script>

<svelte:window onkeydowncapture={handleShellShortcut} />

<div class="titlebar" data-tauri-drag-region>
  <!-- Every button here carries `tabindex="-1"`: this is window chrome, and Windows' own
       title bar buttons aren't in the Tab order either — tabbing into the app should land
       on its content, not spend five stops on the frame first. They stay clickable, and
       the two that do something app-specific have their own routes in: the shell has
       Ctrl+` (see handleShellShortcut), and the sidebar can be reached with Tab and
       driven with the arrow keys once there.

       This button is always present: pinning the sidebar open is the primary way to
       expand it, and the hover-expand setting only adds a second, temporary one on top. -->
  <button
    class="titlebar-btn"
    tabindex="-1"
    onclick={toggleSidebar}
    title={$t("titlebar.toggleSidebar")}
    aria-label={$t("titlebar.toggleSidebar")}
  >
    <Icon svg={panelLeftIcon} size={16} />
  </button>
  <div class="titlebar-title" data-tauri-drag-region>Dockl</div>
  {#if $connection.distro}
    <span class="distro-badge dockl-surface" title={$t("titlebar.wslDistro")}
      >{$connection.distro}</span
    >
  {/if}
  <button
    class="titlebar-btn titlebar-shell-btn"
    tabindex="-1"
    onclick={openShell}
    title={`${$t("titlebar.openShell")} (Ctrl+\`)`}
    aria-label={$t("titlebar.openShell")}
  >
    <Icon svg={terminalIcon} size={16} />
  </button>
  <div class="titlebar-controls">
    <button
      class="titlebar-btn"
      tabindex="-1"
      onclick={minimize}
      aria-label={$t("titlebar.minimize")}
    >
      <Icon svg={subtractIcon} size={14} />
    </button>
    <button
      class="titlebar-btn"
      tabindex="-1"
      onclick={toggleMaximize}
      aria-label={$t("titlebar.maximize")}
    >
      <Icon svg={squareIcon} size={12} />
    </button>
    <button
      class="titlebar-btn titlebar-close"
      tabindex="-1"
      onclick={close}
      aria-label={$t("common.close")}
    >
      <Icon svg={dismissIcon} size={14} />
    </button>
  </div>
</div>

{#if shellMounted}
  <WslShellDialog hidden={!shellVisible} onClose={hideShell} onEnded={endShell} />
{/if}

<style>
  .titlebar {
    display: flex;
    align-items: center;
    height: 36px;
    flex-shrink: 0;
    background: var(--dockl-titlebar);
    border-bottom: 1px solid var(--dockl-border);
    user-select: none;
  }

  .titlebar-title {
    flex: 1;
    padding-left: 12px;
    font-size: 12px;
    color: var(--dockl-text-secondary);
  }

  .distro-badge {
    padding: 2px 10px;
    font-size: 11px;
    color: var(--dockl-text-secondary);
    flex-shrink: 0;
  }

  .titlebar-controls {
    display: flex;
    height: 100%;
  }

  .titlebar-shell-btn {
    width: 36px;
  }

  .titlebar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 100%;
    border: none;
    background: transparent;
    color: var(--dockl-text-secondary);
    cursor: pointer;
  }

  .titlebar-btn:hover {
    background: var(--dockl-surface-hover);
  }

  .titlebar-close:hover {
    background: var(--dockl-danger);
    color: white;
  }
</style>
