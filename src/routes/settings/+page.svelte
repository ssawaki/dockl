<script lang="ts">
  import { formatError } from "$lib/errors";
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { load, type Store } from "@tauri-apps/plugin-store";
  import { isEnabled as autostartIsEnabled, enable as autostartEnable, disable as autostartDisable } from "@tauri-apps/plugin-autostart";
  import { setupCurrentDistro, setupConnect, connectTcpBridge, connectDialStdio } from "$lib/ipc/setup";
  import { ensureConnected, getConnectionMode, persistConnectionMode, type ConnectionMode } from "$lib/connection";
  import { TCP_BRIDGE_PORT } from "$lib/tcpBridge";
  import { connection } from "$lib/stores/connection";
  import {
    themeMode,
    windowMaterial,
    sidebarHoverExpand,
    setThemeMode,
    setWindowMaterial,
    setSidebarHoverExpand,
    type ThemeMode,
    type WindowMaterial,

  } from "$lib/stores/appearance";
  import PageHeader from "$lib/components/layout/PageHeader.svelte";
  import TcpEndpointDialog from "$lib/components/settings/TcpEndpointDialog.svelte";
  import { t, locale, setLocale, type Locale } from "$lib/stores/i18n";

  let store: Store | null = null;
  let trayEnabled = $state(false);
  let autostartEnabled = $state(false);
  let reconnecting = $state(false);
  let errorMessage = $state<string | null>(null);

  // fluent-radio-group still governs its children's checked state itself (setting each
  // <fluent-radio>'s own `checked` property directly gets overridden once the group
  // finishes registering them) — same timing issue as the appearance radios earlier:
  // the group's `value` PROPERTY setter runs before its child radios have registered
  // themselves (async, via slotchange), so setting it too early is a no-op. Re-applying
  // once after mount, once `connectionMode` has actually loaded, works around it.
  let connectionRadioGroup: (HTMLElement & { value: string }) | undefined = $state();

  onMount(async () => {
    store = await load("settings.json", { autoSave: true });
    trayEnabled = (await store.get<boolean>("trayEnabled")) ?? false;
    autostartEnabled = await autostartIsEnabled();
    connectionMode = await getConnectionMode();
    requestAnimationFrame(() => {
      if (connectionRadioGroup) connectionRadioGroup.value = connectionMode;
    });
  });

  async function toggleTray(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    trayEnabled = checked;
    await store?.set("trayEnabled", checked);
  }

  async function toggleAutostart(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    try {
      if (checked) {
        await autostartEnable();
      } else {
        await autostartDisable();
      }
      autostartEnabled = checked;
    } catch (err) {
      errorMessage = formatError(err);
    }
  }

  function handleThemeModeChange(e: Event) {
    void setThemeMode((e.target as HTMLElement & { value: string }).value as ThemeMode);
  }

  function handleWindowMaterialChange(e: Event) {
    void setWindowMaterial((e.target as HTMLElement & { value: string }).value as WindowMaterial);
  }

  function handleLocaleChange(e: Event) {
    void setLocale((e.target as HTMLElement & { value: string }).value as Locale);
  }

  let tcpEndpointDialogOpen = $state(false);
  let connectionMode = $state<ConnectionMode>("shell_out");
  let switchingMode = $state(false);

  // `state.connection` (container/image/volume/network list/action/prune) is what this
  // actually switches — Compose/logs/stats/attach always shell out regardless, see
  // `connect_tcp_bridge`'s doc comment.
  async function handleConnectionModeChange(e: Event) {
    const nextMode = (e.target as HTMLElement & { value: string }).value as ConnectionMode;
    if (nextMode === connectionMode || switchingMode) return;

    switchingMode = true;
    errorMessage = null;
    try {
      if (nextMode === "user_managed_tcp") {
        await connectTcpBridge(TCP_BRIDGE_PORT);
      } else if (nextMode === "dial_stdio") {
        await connectDialStdio();
      } else {
        const distro = $connection.distro;
        if (!distro) throw new Error(get(t)("errors.noDistroSelected"));
        await setupConnect(distro);
      }
      connectionMode = nextMode;
      await persistConnectionMode(nextMode);
    } catch (err) {
      errorMessage = get(t)("errors.connectionSwitchFailed", { error: formatError(err) });
      // The click already flipped the radio's own visual state before this handler
      // even ran; since the switch didn't actually happen, put it back.
      if (connectionRadioGroup) connectionRadioGroup.value = connectionMode;
    } finally {
      switchingMode = false;
    }
  }

  async function reconnect() {
    reconnecting = true;
    errorMessage = null;
    try {
      const connected = await ensureConnected();
      if (!connected) {
        errorMessage = get(t)("errors.notConnectedSetupRequired");
        connection.set({ status: "disconnected", distro: null });
        return;
      }
      const distro = await setupCurrentDistro();
      connection.set({ status: "connected", distro });
    } catch (err) {
      errorMessage = formatError(err);
    } finally {
      reconnecting = false;
    }
  }
</script>

<div class="settings-view">
  <PageHeader title={$t("nav.settings")} />

  {#if errorMessage}
    <div class="error-banner dockl-surface">{errorMessage}</div>
  {/if}

  <section class="dockl-surface">
    <h2>{$t("settings.language.heading")}</h2>
    <fluent-dropdown style="min-width: 200px" onchange={handleLocaleChange}>
      <fluent-listbox>
        <fluent-option value="ja" selected={$locale === "ja"}>{$t("settings.language.ja")}</fluent-option>
        <fluent-option value="ja-en" selected={$locale === "ja-en"}>{$t("settings.language.jaEn")}</fluent-option>
        <fluent-option value="en" selected={$locale === "en"}>{$t("settings.language.en")}</fluent-option>
      </fluent-listbox>
    </fluent-dropdown>
  </section>

  <section class="dockl-surface">
    <h2>{$t("settings.appearance.heading")}</h2>

    <svelte:boundary>
      <div class="appearance-group">
        <span class="label">{$t("settings.appearance.theme.label")}</span>
        <fluent-dropdown style="min-width: 160px" onchange={handleThemeModeChange}>
          <fluent-listbox>
            <fluent-option value="system" selected={$themeMode === "system"}>{$t("settings.appearance.theme.system")}</fluent-option>
            <fluent-option value="light" selected={$themeMode === "light"}>{$t("settings.appearance.theme.light")}</fluent-option>
            <fluent-option value="dark" selected={$themeMode === "dark"}>{$t("settings.appearance.theme.dark")}</fluent-option>
          </fluent-listbox>
        </fluent-dropdown>
      </div>

      <div class="appearance-group">
        <span class="label">{$t("settings.appearance.background.label")}</span>
        <fluent-dropdown style="min-width: 160px" onchange={handleWindowMaterialChange}>
          <fluent-listbox>
            <fluent-option value="mica" selected={$windowMaterial === "mica"}>{$t("settings.appearance.background.mica")}</fluent-option>
            <fluent-option value="acrylic" selected={$windowMaterial === "acrylic"}>{$t("settings.appearance.background.acrylic")}</fluent-option>
            <fluent-option value="solid" selected={$windowMaterial === "solid"}>{$t("settings.appearance.background.solid")}</fluent-option>
          </fluent-listbox>
        </fluent-dropdown>
      </div>

      <!-- svelte-ignore a11y_label_has_associated_control -->
      <label class="row toggle-row">
        <fluent-switch
          checked={$sidebarHoverExpand}
          onchange={(e: Event) => void setSidebarHoverExpand((e.target as HTMLInputElement).checked)}
        ></fluent-switch>
        <span>{$t("settings.appearance.sidebarHoverExpand")}</span>
      </label>

      {#snippet failed(error)}
        <p class="error-banner dockl-surface">{$t("settings.appearance.error", { error: String(error) })}</p>
      {/snippet}
    </svelte:boundary>
  </section>

  <section class="dockl-surface">
    <h2>{$t("settings.connection.heading")}</h2>
    <p class="section-desc">{$t("settings.connection.description")}</p>

    <fluent-radio-group bind:this={connectionRadioGroup} orientation="vertical" onchange={handleConnectionModeChange}>
      <!-- svelte-ignore a11y_label_has_associated_control -->
      <label class="mode-option"
        ><fluent-radio value="dial_stdio" disabled={switchingMode}></fluent-radio>
        <span class="mode-text">
          <span class="mode-name">
            {$t("settings.connection.dialStdio")}
            <span class="mode-badge">{$t("settings.connection.recommended")}</span>
          </span>
          <span class="mode-desc">{$t("settings.connection.dialStdio.desc")}</span>
        </span></label
      >
      <!-- svelte-ignore a11y_label_has_associated_control -->
      <label class="mode-option"
        ><fluent-radio value="shell_out" checked disabled={switchingMode}></fluent-radio>
        <span class="mode-text">
          <span class="mode-name">{$t("settings.connection.shellOut")}</span>
          <span class="mode-desc">{$t("settings.connection.shellOut.desc")}</span>
        </span></label
      >
      <!-- svelte-ignore a11y_label_has_associated_control -->
      <label class="mode-option"
        ><fluent-radio value="user_managed_tcp" disabled={switchingMode}></fluent-radio>
        <span class="mode-text">
          <span class="mode-name">{$t("settings.connection.tcp")}</span>
          <span class="mode-desc">{$t("settings.connection.tcp.desc")}</span>
        </span></label
      >
    </fluent-radio-group>
    {#if switchingMode}
      <div class="row">
        <fluent-spinner size="tiny"></fluent-spinner>
        <span class="label">{$t("settings.connection.switching")}</span>
      </div>
    {/if}

    <div class="row">
      <span class="label">{$t("settings.connection.current")}</span>
      <span>{$connection.distro ?? $t("settings.connection.notConnected")}</span>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <fluent-button appearance="outline" disabled={reconnecting} onclick={reconnect}>
        <span class="btn-content">
          {#if reconnecting}
            <fluent-spinner size="tiny"></fluent-spinner>
          {/if}
          {reconnecting ? $t("common.connecting") : $t("settings.connection.reconnect")}
        </span>
      </fluent-button>
    </div>

    <fluent-divider></fluent-divider>

    <div class="row">
      <span class="label">{$t("settings.connection.tcpSetupLabel")}</span>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <fluent-button appearance="outline" onclick={() => (tcpEndpointDialogOpen = true)}>{$t("settings.connection.setupButton")}</fluent-button>
    </div>
  </section>

  <section class="dockl-surface">
    <h2>{$t("settings.tray.heading")}</h2>
    <!-- svelte-ignore a11y_label_has_associated_control -->
    <label class="row toggle-row">
      <fluent-switch checked={trayEnabled} onchange={toggleTray}></fluent-switch>
      <span>{$t("settings.tray.toggle")}</span>
    </label>
  </section>

  <section class="dockl-surface">
    <h2>{$t("settings.autostart.heading")}</h2>
    <!-- svelte-ignore a11y_label_has_associated_control -->
    <label class="row toggle-row">
      <fluent-switch checked={autostartEnabled} onchange={toggleAutostart}></fluent-switch>
      <span>{$t("settings.autostart.toggle")}</span>
    </label>
  </section>

</div>

{#if tcpEndpointDialogOpen}
  <TcpEndpointDialog onClose={() => (tcpEndpointDialogOpen = false)} />
{/if}

<style>
  .settings-view {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 640px;
  }

  section {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  h2 {
    font-size: 14px;
    font-weight: 600;
    margin: 0;
  }

  .section-desc {
    font-size: 12px;
    color: var(--dockl-text-secondary);
    margin: 0;
  }

  /* The three connection modes differ in ways a bare label can't convey (one opens an
     unauthenticated port, one is markedly slower), so each carries a one-line summary
     of the tradeoff rather than leaving the user to guess from the name. */
  .mode-option {
    /* Overrides the shared `fluent-radio-group label` rule above (inline-flex, centered):
       these rows are full-width and their text is multi-line, so the radio has to sit
       against the first line rather than the vertical middle of the whole block. */
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 4px 0;
  }

  /* Without this the radio is a shrinkable flex item sitting next to a much longer
     description, so each option's circle gets squeezed by a different amount depending on
     how long its own text is and the three end up visibly different sizes. Same UA-sizing
     -versus-flex trap as the icon buttons (see LogViewer.svelte's note). */
  .mode-option > fluent-radio {
    flex-shrink: 0;
    /* Nudged down to sit on the first text line's baseline rather than its box top. */
    margin-top: 1px;
  }

  .mode-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    /* A flex item's default `min-width: auto` refuses to shrink below its longest word,
       which would push these descriptions out past the radio group instead of wrapping. */
    min-width: 0;
  }

  .mode-name {
    display: flex;
    align-items: center;
    gap: 8px;
    /* The badge drops to its own line on a narrow window instead of compressing the name. */
    flex-wrap: wrap;
  }

  .mode-badge {
    padding: 1px 6px;
    font-size: 11px;
    font-weight: 600;
    color: var(--dockl-success);
    background: color-mix(in srgb, var(--dockl-success) 15%, transparent);
    border-radius: 10px;
  }

  .mode-desc {
    font-size: 12px;
    line-height: 1.4;
    color: var(--dockl-text-secondary);
  }


  .row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .appearance-group {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  /* fluent-radio has no default slot for label text at all (only a named slot for its
     checked-indicator dot) — any text placed inside it is silently dropped. The label
     has to live outside it instead, as plain sibling text inside a wrapping <label>
     (same pattern <fluent-switch>/<fluent-checkbox> already use elsewhere on this page). */
  fluent-radio-group label {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
  }

  .appearance-group .label {
    flex-shrink: 0;
    min-width: 48px;
  }

  .toggle-row {
    cursor: pointer;
  }

  .label {
    color: var(--dockl-text-secondary);
  }

  .error-banner {
    padding: 8px 12px;
    color: var(--dockl-danger);
    border-color: var(--dockl-danger);
  }

  .btn-content {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
</style>
