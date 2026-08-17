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
  import { invoke } from "@tauri-apps/api/core";
  import AboutDialog from "$lib/components/ui/AboutDialog.svelte";

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

  // Not a menu this app builds: `show_system_menu` opens the window's real Win32 one (see
  // src-tauri/src/system_menu.rs). About is the single entry appended to it, and the only
  // one that comes back here — the command dispatches the rest itself.
  let iconBtn: HTMLElement | undefined = $state();
  let aboutOpen = $state(false);

  async function showAppMenu(x: number, y: number) {
    try {
      // CSS pixels; the Rust side converts to the screen coordinates TrackPopupMenu wants.
      const about = await invoke<boolean>("show_system_menu", {
        x,
        y,
        aboutLabel: $t("about.title"),
      });
      if (about) aboutOpen = true;
    } catch (e) {
      console.error("system menu failed to open", e);
    }
  }

  function openAppMenuAtIcon() {
    const rect = iconBtn?.getBoundingClientRect();
    // 36 is the title bar's height, so the fallback lands just under it.
    void showAppMenu(rect?.left ?? 0, rect?.bottom ?? 36);
  }

  // Windows opens this menu from three places, and a window drawing its own chrome has to
  // wire all three: the icon, a right-click anywhere on the title bar, and Alt+Space.
  function handleTitlebarContextMenu(e: MouseEvent) {
    e.preventDefault();
    void showAppMenu(e.clientX, e.clientY);
  }

  function handleAppMenuShortcut(e: KeyboardEvent) {
    if (e.key !== " " || !e.altKey || e.ctrlKey || e.metaKey) return;
    e.preventDefault();
    openAppMenuAtIcon();
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

<svelte:window onkeydowncapture={handleShellShortcut} onkeydown={handleAppMenuShortcut} />

<!-- No role or keyboard handler on the bar itself: Alt+Space opens the same menu, and is
     the binding Windows gives it. -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="titlebar" data-tauri-drag-region oncontextmenu={handleTitlebarContextMenu}>
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
  <!-- After the pane toggle rather than in the corner, which is what WinUI's own TitleBar
       control does — its anatomy runs back button, pane toggle, left header, icon, title.
       The corner stays with the toggle: that's the control lining up with the navigation
       rail below it. -->
  <button
    bind:this={iconBtn}
    class="titlebar-icon-btn"
    tabindex="-1"
    onclick={openAppMenuAtIcon}
    aria-label={$t("titlebar.appMenu")}
    aria-haspopup="menu"
  >
    <img src="/app-icon.png" alt="" width="16" height="16" />
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

{#if aboutOpen}
  <AboutDialog onClose={() => (aboutOpen = false)} />
{/if}

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

  /* Narrower than the 44px chrome buttons, and alone in taking no hover background:
     Windows' own title bar icon behaves this way too. It reads as the window's mark
     rather than a fourth control in the row. */
  .titlebar-icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 100%;
    border: none;
    background: transparent;
    cursor: default;
  }

  .titlebar-title {
    flex: 1;
    padding-left: 8px;
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
    /* Every button in this bar keeps the arrow, as Windows' own title bars do — including
       the ones apps add for themselves. Splitting it by what each button acts on (the
       window, or the app) is a distinction nobody can see in a single 36px strip; what
       does carry the affordance is the hover background. */
    cursor: default;
  }

  .titlebar-btn:hover {
    background: var(--dockl-surface-hover);
  }

  .titlebar-close:hover {
    background: var(--dockl-danger);
    color: white;
  }
</style>
