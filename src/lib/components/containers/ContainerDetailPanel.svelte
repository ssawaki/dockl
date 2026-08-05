<script lang="ts">
  import { formatError } from "$lib/errors";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { inspectContainer } from "$lib/ipc/containers";
  import LoadingState from "$lib/components/ui/LoadingState.svelte";
  import ContextMenu, { type ContextMenuItem } from "$lib/components/ui/ContextMenu.svelte";
  import CopyableValue from "$lib/components/ui/CopyableValue.svelte";
  import { rovingFocus } from "$lib/actions/rovingFocus";
  import { imageRegistryUrl, splitImageTag } from "$lib/dockerImage";
  import { copyToClipboard } from "$lib/clipboard";
  import LogViewer from "$lib/components/terminal/LogViewer.svelte";
  import TerminalSession from "$lib/components/terminal/TerminalSession.svelte";
  import ContainerStats from "$lib/components/containers/ContainerStats.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { get } from "svelte/store";
  import { t } from "$lib/stores/i18n";
  import copyIcon from "@fluentui/svg-icons/icons/copy_16_regular.svg?raw";
  import dismissCircleIcon from "@fluentui/svg-icons/icons/dismiss_circle_16_regular.svg?raw";
  import type { ContainerDetail, DetailTabId } from "$lib/types";
  import { groupPortForwards } from "$lib/ports";

  // `liveState` comes from the container list the parent route already polls every 5s
  // (`ContainerSummary.state`) — reused here instead of this panel separately re-polling
  // `inspect_container` on its own timer just to notice a status change. That's what an
  // earlier version of this fix did, and it was a real regression: `inspect_container`
  // returns much more than just status (mounts/ports/labels/health/...), so polling it
  // every 5s purely to track liveness was both redundant with data already being
  // fetched and needlessly expensive (an extra `wsl.exe` spawn or HTTP round trip every
  // 5s for the whole time any container is selected).
  //
  // `activeTab` is bound from the parent route rather than owned locally, so it
  // survives switching to a Compose project and back (see +page.svelte).
  let {
    containerId,
    liveState,
    activeTab = $bindable(),
  }: { containerId: string | null; liveState: string | null; activeTab: DetailTabId } = $props();

  let detail = $state<ContainerDetail | null>(null);
  let loading = $state(false);
  let errorMessage = $state<string | null>(null);

  let tabs = $derived<{ id: DetailTabId; label: string }[]>([
    { id: "info", label: $t("containers.tab.info") },
    { id: "stats", label: $t("containers.tab.stats") },
    { id: "logs", label: $t("containers.tab.logs") },
    { id: "terminal", label: $t("containers.tab.terminal") },
  ]);

  // Falls back to `detail.status` (same underlying value, just from a one-shot
  // `inspect_container`) for the brief window right after selecting a container, before
  // `liveState` has resolved from the props path — cosmetic only, both settle to the
  // same value once `detail` loads.
  let isRunning = $derived((liveState ?? detail?.status) === "running");

  let displayPorts = $derived(groupPortForwards(detail?.ports ?? []));

  // Guards against rapidly switching the selected container: without this, an older
  // `inspectContainer` call that happens to resolve after a newer one would overwrite
  // `detail` with stale data for whatever's currently selected.
  let loadToken = 0;

  async function loadDetail(id: string) {
    const token = ++loadToken;
    loading = true;
    errorMessage = null;
    try {
      const result = await inspectContainer(id);
      if (token !== loadToken) return;
      detail = result;
    } catch (e) {
      if (token !== loadToken) return;
      errorMessage = formatError(e);
      detail = null;
    } finally {
      if (token === loadToken) loading = false;
    }
  }

  // Debounced: rapid-fire selection changes (e.g. clicking through several rows, or
  // holding a key that repeatedly re-selects) would otherwise fire one `inspectContainer`
  // per intermediate selection — each an `inspect_container` call (a `wsl.exe` spawn in
  // shell-out mode) — even though only the final selection's result ever gets shown.
  $effect(() => {
    if (!containerId) {
      detail = null;
      return;
    }
    const id = containerId;
    const timer = setTimeout(() => loadDetail(id), 150);
    return () => clearTimeout(timer);
  });

  /** Opens in the system browser rather than letting the webview navigate to it. */
  function openHostPort(e: MouseEvent, url: string) {
    e.preventDefault();
    void openUrl(url);
  }

  function openExternal(e: MouseEvent, url: string) {
    e.preventDefault();
    void openUrl(url);
  }

  /** Text selected within `el`, or null if there's no selection (or it's elsewhere). */
  function selectionWithin(el: HTMLElement): string | null {
    const sel = window.getSelection();
    if (!sel || sel.isCollapsed || sel.toString().length === 0) return null;
    const { anchorNode, focusNode } = sel;
    if (anchorNode && el.contains(anchorNode) && focusNode && el.contains(focusNode)) {
      return sel.toString();
    }
    return null;
  }

  let copyMenu = $state<{ x: number; y: number; items: ContextMenuItem[] } | null>(null);

  function handleValueContextMenu(e: MouseEvent) {
    const cell = (e.target as HTMLElement).closest<HTMLElement>("td");
    if (!cell) return;
    e.preventDefault();
    e.stopPropagation();

    const text = selectionWithin(cell) ?? cell.textContent?.trim() ?? "";
    if (!text) return;

    copyMenu = {
      x: e.clientX,
      y: e.clientY,
      items: [
        {
          label: get(t)("common.copy"),
          icon: copyIcon,
          onClick: () => void copyToClipboard(text),
        },
      ],
    };
  }

  function closeCopyMenu() {
    copyMenu = null;
  }
</script>

<div class="detail-panel dockl-surface">
  {#if !containerId}
    <div class="placeholder">{$t("containers.detail.placeholder")}</div>
  {:else}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="tabs"
      role="tablist"
      use:rovingFocus={{ selector: "[data-roving-item]", orientation: "horizontal" }}
    >
      {#each tabs as tab (tab.id)}
        <div
          class="tab"
          class:active={activeTab === tab.id}
          onclick={() => (activeTab = tab.id)}
          role="tab"
          aria-selected={activeTab === tab.id}
          data-roving-item
          tabindex="-1"
        >
          {tab.label}
        </div>
      {/each}
    </div>

    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="tab-content"
      class:flush={activeTab === "logs" || activeTab === "terminal"}
      class:no-padding={activeTab === "stats"}
      oncontextmenu={handleValueContextMenu}
    >
      {#if errorMessage}
        <div class="error-banner">{errorMessage}</div>
      {:else if loading}
        <LoadingState />
      {:else if activeTab === "info" && detail}
        {@const image = detail.image}
        {@const imageUrl = imageRegistryUrl(image)}
        {@const { repo, tag } = splitImageTag(image)}
        {@const idShort = detail.id.slice(0, 12)}
        {@const statusText = `${detail.status}${detail.health ? ` (${detail.health})` : ""}`}
        <table class="info-table">
          <tbody>
            <tr>
              <th>Name</th>
              <td><CopyableValue value={detail.name}>{detail.name}</CopyableValue></td>
            </tr>
            <tr>
              <th>ID</th>
              <td><CopyableValue value={idShort}>{idShort}</CopyableValue></td>
            </tr>
            <tr>
              <th>Image</th>
              <td>
                <CopyableValue value={image}>
                  {#if imageUrl}
                    <!-- Registry URL on the public internet, not an app route. `resolve()`
                         throws on anything that isn't an absolute internal pathname. -->
                    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
                    <a class="ext-link" href={imageUrl} onclick={(e) => openExternal(e, imageUrl)}
                      >{repo}</a
                    >{tag ? `:${tag}` : ""}
                  {:else}
                    {image}
                  {/if}
                </CopyableValue>
              </td>
            </tr>
            <tr>
              <th>Status</th>
              <td><CopyableValue value={statusText}>{statusText}</CopyableValue></td>
            </tr>
            <tr>
              <th>Restart Policy</th>
              <td
                ><CopyableValue value={detail.restart_policy}>{detail.restart_policy}</CopyableValue
                ></td
              >
            </tr>
            {#if detail.ip_address}
              <tr>
                <th>IP</th>
                <td><CopyableValue value={detail.ip_address}>{detail.ip_address}</CopyableValue></td
                >
              </tr>
            {/if}
          </tbody>
        </table>

        {#if displayPorts.length > 0}
          <h3>Port Forwards</h3>
          <table class="data-table">
            <thead>
              <tr><th>Host Port</th><th>Container Port</th><th>Protocol</th></tr>
            </thead>
            <tbody>
              {#each displayPorts as p (`${p.address}\t${p.containerPort}\t${p.protocol}`)}
                <tr>
                  <td>
                    <!-- Address and port copy as one value: either half on its own is
                         not something that can be pasted anywhere useful. -->
                    <CopyableValue value={p.address}>
                      {#if p.url}
                        <!-- Points at the published host port (http://localhost:…), not an
                             app route, so `resolve()` would throw on it. -->
                        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
                        <a class="ext-link" href={p.url} onclick={(e) => openHostPort(e, p.url!)}
                          >{p.address}</a
                        >
                      {:else}
                        {p.address}
                      {/if}
                    </CopyableValue>
                  </td>
                  <td><CopyableValue value={p.containerPort}>{p.containerPort}</CopyableValue></td>
                  <td><CopyableValue value={p.protocol}>{p.protocol}</CopyableValue></td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}

        {#if detail.mounts.length > 0}
          <h3>Mounts</h3>
          <table class="data-table">
            <thead>
              <tr><th>Source</th><th>Destination</th></tr>
            </thead>
            <tbody>
              {#each detail.mounts as m (m.destination)}
                <tr>
                  <td><CopyableValue value={m.source}>{m.source}</CopyableValue></td>
                  <td><CopyableValue value={m.destination}>{m.destination}</CopyableValue></td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}

        {#if Object.keys(detail.labels).length > 0}
          <h3>Labels</h3>
          <table class="data-table">
            <thead>
              <tr><th>Key</th><th>Value</th></tr>
            </thead>
            <tbody>
              {#each Object.entries(detail.labels) as [k, v] (k)}
                <tr>
                  <td><CopyableValue value={k}>{k}</CopyableValue></td>
                  <td><CopyableValue value={v}>{v}</CopyableValue></td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      {:else if activeTab === "stats" && containerId}
        {#key containerId}
          <ContainerStats
            {containerId}
            cpuLimitCores={detail?.cpu_limit_cores ?? null}
            {isRunning}
          />
        {/key}
      {:else if activeTab === "logs" && containerId}
        {#key containerId}
          <LogViewer {containerId} {isRunning} />
        {/key}
      {:else if activeTab === "terminal" && containerId}
        {#if isRunning}
          {#key containerId}
            <TerminalSession {containerId} />
          {/key}
        {:else}
          <div class="terminal-unavailable">
            <Icon svg={dismissCircleIcon} size={20} />
            <p>{$t("containers.detail.terminalUnavailable")}</p>
            <p class="hint-sub">
              {$t("containers.detail.currentStatus", {
                status: detail?.status ?? $t("common.unknown"),
              })}
            </p>
          </div>
        {/if}
      {:else if activeTab !== "info"}
        <p class="hint">{$t("common.comingSoon")}</p>
      {/if}
    </div>
  {/if}
</div>

{#if copyMenu}
  <ContextMenu x={copyMenu.x} y={copyMenu.y} items={copyMenu.items} onClose={closeCopyMenu} />
{/if}

<style>
  .detail-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }

  .placeholder {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--dockl-text-secondary);
  }

  .terminal-unavailable {
    /* `.tab-content` (the parent) isn't itself a flex container, so `flex: 1` here
       would be inert — `height: 100%` is what actually stretches this to the tab's
       full height, which the `justify-content: center` below needs to have any
       vertical room to center within. */
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 24px;
    color: var(--dockl-text-secondary);
    text-align: center;
  }

  .terminal-unavailable p {
    margin: 0;
    font-size: 13px;
  }

  .terminal-unavailable .hint-sub {
    font-size: 12px;
  }

  .tabs {
    display: flex;
    gap: 4px;
    padding: 8px 12px 0;
    border-bottom: 1px solid var(--dockl-border);
  }

  .tab {
    padding: 8px 14px;
    font-size: 13px;
    color: var(--dockl-text-secondary);
    cursor: pointer;
    border-bottom: 2px solid transparent;
  }

  .tab:hover {
    color: var(--dockl-text-primary);
  }

  .tab.active {
    color: var(--dockl-accent);
    border-bottom-color: var(--dockl-accent);
  }

  .tab-content {
    flex: 1;
    overflow: auto;
    padding: 14px 16px;
  }

  /* The log/terminal views manage their own scrolling (xterm's internal scrollback),
     so they get the full panel with no padding rather than sitting inside another
     scrollable, padded box. */
  .tab-content.flush {
    overflow: hidden;
    padding: 0;
  }

  /* Stats needs the same edge-to-edge "not running" banner as logs/terminal (so it
     isn't indented by this element's own padding), but — unlike logs/terminal — its
     content is plain DOM, not a self-scrolling xterm viewport, so it keeps the normal
     `overflow: auto` and manages its own inner padding instead (see
     ContainerStats.svelte's `.stats-body`). */
  .tab-content.no-padding {
    padding: 0;
  }

  .hint {
    color: var(--dockl-text-secondary);
  }

  .error-banner {
    color: var(--dockl-danger);
  }

  h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--dockl-text-secondary);
    margin: 18px 0 6px;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .info-table th {
    text-align: left;
    color: var(--dockl-text-secondary);
    font-weight: 500;
    padding: 4px 12px 4px 0;
    white-space: nowrap;
    vertical-align: top;
  }

  .info-table td {
    padding: 4px 0;
    word-break: break-all;
    user-select: text;
    cursor: text;
  }

  .data-table th,
  .data-table td {
    text-align: left;
    padding: 6px 10px;
    border-bottom: 1px solid var(--dockl-border);
  }

  .data-table td {
    user-select: text;
    cursor: text;
  }

  .data-table th {
    color: var(--dockl-text-secondary);
    font-weight: 500;
  }

  .ext-link {
    color: var(--dockl-link);
    text-decoration: none;
    cursor: pointer;
  }

  .ext-link:hover {
    text-decoration: underline;
  }
</style>
