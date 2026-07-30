<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Icon from "$lib/components/Icon.svelte";
  import subtractIcon from "@fluentui/svg-icons/icons/subtract_20_regular.svg?raw";
  import squareIcon from "@fluentui/svg-icons/icons/square_20_regular.svg?raw";
  import dismissIcon from "@fluentui/svg-icons/icons/dismiss_20_regular.svg?raw";

  const appWindow = getCurrentWindow();

  function minimize() {
    appWindow.minimize();
  }

  function toggleMaximize() {
    appWindow.toggleMaximize();
  }

  function close() {
    appWindow.close();
  }
</script>

<div class="titlebar" data-tauri-drag-region>
  <div class="titlebar-title" data-tauri-drag-region>Dockl</div>
  <div class="titlebar-controls">
    <button class="titlebar-btn" onclick={minimize} aria-label="Minimize">
      <Icon svg={subtractIcon} size={14} />
    </button>
    <button class="titlebar-btn" onclick={toggleMaximize} aria-label="Maximize">
      <Icon svg={squareIcon} size={12} />
    </button>
    <button class="titlebar-btn titlebar-close" onclick={close} aria-label="Close">
      <Icon svg={dismissIcon} size={14} />
    </button>
  </div>
</div>

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

  .titlebar-controls {
    display: flex;
    height: 100%;
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
