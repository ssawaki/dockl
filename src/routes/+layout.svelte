<script lang="ts">
  import "../lib/styles/theme.css";
  import "@xterm/xterm/css/xterm.css";
  import "@fluentui/web-components/button/define.js";
  import "@fluentui/web-components/badge/define.js";
  import "@fluentui/web-components/switch/define.js";
  import "@fluentui/web-components/divider/define.js";
  import "@fluentui/web-components/radio-group/define.js";
  import "@fluentui/web-components/radio/define.js";
  import "@fluentui/web-components/spinner/define.js";
  import { goto } from "$app/navigation";
  import Titlebar from "$lib/components/Titlebar.svelte";
  import SidebarNav from "$lib/components/SidebarNav.svelte";
  import ToastStack from "$lib/components/ToastStack.svelte";
  import { initFluentTheme } from "$lib/fluentTheme";

  initFluentTheme();

  let { children } = $props();

  // Native Windows apps don't show a browser right-click menu (Back/Reload/Inspect...).
  // Individual components (e.g. ContainerMasterList) implement their own context menus
  // on top of this by calling preventDefault() themselves in their own handler.
  function disableDefaultContextMenu(e: MouseEvent) {
    e.preventDefault();
  }

  // Ctrl+, opens Settings, matching the convention used by VS Code/Chrome/Slack.
  function handleGlobalKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && !e.altKey && !e.metaKey && e.key === ",") {
      e.preventDefault();
      goto("/settings");
    }
  }
</script>

<svelte:window oncontextmenu={disableDefaultContextMenu} onkeydown={handleGlobalKeydown} />

<div class="dockl-app">
  <Titlebar />
  <div class="dockl-body">
    <SidebarNav />
    <div class="dockl-content">
      {@render children()}
    </div>
  </div>
  <ToastStack />
</div>

<style>
  .dockl-body {
    display: flex;
    flex: 1;
    min-height: 0;
  }
</style>
