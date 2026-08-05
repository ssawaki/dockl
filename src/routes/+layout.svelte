<script lang="ts">
  import { formatError } from "$lib/errors";
  import "../lib/styles/theme.css";
  import "@xterm/xterm/css/xterm.css";
  import "@fluentui/web-components/button/define.js";
  import "@fluentui/web-components/badge/define.js";
  import "@fluentui/web-components/switch/define.js";
  import "@fluentui/web-components/checkbox/define.js";
  import "@fluentui/web-components/divider/define.js";
  import "@fluentui/web-components/radio-group/define.js";
  import "@fluentui/web-components/radio/define.js";
  import "@fluentui/web-components/spinner/define.js";
  import "@fluentui/web-components/dropdown/define.js";
  import "@fluentui/web-components/listbox/define.js";
  import "@fluentui/web-components/option/define.js";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { page } from "$app/stores";
  import Titlebar from "$lib/components/layout/Titlebar.svelte";
  import SidebarNav from "$lib/components/layout/SidebarNav.svelte";
  import ToastStack from "$lib/components/ui/ToastStack.svelte";
  import LoadingState from "$lib/components/ui/LoadingState.svelte";
  import { initAppearance } from "$lib/stores/appearance";
  import { ensureConnected, getConnectedDistro } from "$lib/connection";
  import { setupCurrentDistro, setupDistroIsRunning } from "$lib/ipc/setup";
  import { connection } from "$lib/stores/connection";
  import { initI18n, t } from "$lib/stores/i18n";

  void initAppearance();
  void initI18n();

  let { children } = $props();

  // Routes that stay usable while the connection is still being established (or has
  // failed). Settings is the important one: it's where the connection method and the
  // reconnect button live, so gating it behind a working connection means the one screen
  // that could fix a bad connection is unreachable exactly when it's needed.
  const ALWAYS_AVAILABLE = new Set(["/settings", "/setup"]);
  let gated = $derived(
    ($connection.status === "connecting" || $connection.status === "starting") &&
      !ALWAYS_AVAILABLE.has($page.url.pathname),
  );

  // Runs once for the app's whole lifetime (this layout instance persists across
  // client-side navigations) — individual routes used to each repeat this same
  // `docker ps`-based check on every visit; now they just read `$connection`.
  onMount(connect);

  async function connect() {
    // Everything is inside the try, including reading the saved distro: that goes through
    // the settings store, which can reject (a corrupt or locked file). Left outside, such
    // a rejection escaped unhandled, `connection` stayed on its initial `connecting`, and
    // the spinner below never cleared — with no retry button, since that only renders for
    // `failed`. A guard against hanging forever must not itself be able to hang forever.
    let saved: string | null = null;
    try {
      saved = await getConnectedDistro();
      // Asked before connecting, not after it stalls: booting a stopped distro takes tens
      // of seconds, and without knowing which case we're in, that wait is indistinguishable
      // from a hang. Answered by the Windows-side WSL service, so it stays fast even when
      // the distro itself isn't responding.
      const running = saved ? await setupDistroIsRunning(saved).catch(() => true) : true;
      connection.set({ status: running ? "connecting" : "starting", distro: saved });

      const ok = await ensureConnected();
      if (!ok) {
        connection.set({ status: "disconnected", distro: null });
        await goto(resolve("/setup"));
        return;
      }
      const distro = await setupCurrentDistro();
      connection.set({ status: "connected", distro });
    } catch (e) {
      // Reached when the connection attempt times out (AppError::ConnectTimeout) — WSL
      // said the distro was there but never answered. Retryable in place rather than a
      // reason to send the user to /setup, which wouldn't help.
      connection.set({ status: "failed", distro: saved, error: formatError(e) });
    }
  }

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
      goto(resolve("/settings"));
    }
  }
</script>

<svelte:window oncontextmenu={disableDefaultContextMenu} onkeydown={handleGlobalKeydown} />

<div class="dockl-app">
  <Titlebar />
  <div class="dockl-body">
    <SidebarNav />
    <div class="dockl-content">
      {#if gated}
        <LoadingState
          message={$connection.status === "starting" ? $t("app.starting") : $t("app.connecting")}
        />
      {:else if $connection.status === "failed" && !ALWAYS_AVAILABLE.has($page.url.pathname)}
        <div class="connect-failed">
          <p class="failed-title">{$t("app.connectFailed")}</p>
          <p class="failed-detail">{$connection.error}</p>
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <fluent-button appearance="accent" onclick={connect}>{$t("app.retry")}</fluent-button>
        </div>
      {:else}
        {@render children()}
      {/if}
    </div>
  </div>
  <ToastStack />
</div>

<style>
  .dockl-body {
    display: flex;
    flex: 1;
    min-height: 0;
    /* Sidebar's hover box-shadow blurs outward past its own box; without this the blur
       bleeds upward into the Titlebar above. Fixed-position overlays (ConfirmDialog etc.)
       are unaffected since their containing block is the viewport, not this element. */
    overflow: hidden;
  }

  .connect-failed {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    height: 100%;
    padding: 24px;
    text-align: center;
  }

  .failed-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .failed-detail {
    margin: 0;
    max-width: 420px;
    font-size: 12px;
    color: var(--dockl-text-secondary);
    user-select: text;
  }
</style>
