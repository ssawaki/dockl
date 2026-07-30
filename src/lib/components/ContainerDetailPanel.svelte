<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { inspectContainer } from "$lib/ipc/containers";
  import LoadingState from "$lib/components/LoadingState.svelte";
  import ContextMenu, { type ContextMenuItem } from "$lib/components/ContextMenu.svelte";
  import CopyableValue from "$lib/components/CopyableValue.svelte";
  import { rovingFocus } from "$lib/actions/rovingFocus";
  import { imageRegistryUrl, splitImageTag } from "$lib/dockerImage";
  import { copyToClipboard } from "$lib/clipboard";
  import LogViewer from "$lib/components/LogViewer.svelte";
  import TerminalSession from "$lib/components/TerminalSession.svelte";
  import copyIcon from "@fluentui/svg-icons/icons/copy_16_regular.svg?raw";
  import type { ContainerDetail } from "$lib/types";

  let { containerId }: { containerId: string | null } = $props();

  type TabId = "info" | "stats" | "logs" | "terminal";
  let activeTab = $state<TabId>("info");
  let detail = $state<ContainerDetail | null>(null);
  let loading = $state(false);
  let errorMessage = $state<string | null>(null);

  const tabs: { id: TabId; label: string }[] = [
    { id: "info", label: "情報" },
    { id: "stats", label: "統計" },
    { id: "logs", label: "ログ" },
    { id: "terminal", label: "ターミナル" },
  ];

  async function loadDetail(id: string) {
    loading = true;
    errorMessage = null;
    try {
      detail = await inspectContainer(id);
    } catch (e) {
      errorMessage = String(e);
      detail = null;
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (containerId) {
      loadDetail(containerId);
    } else {
      detail = null;
    }
  });

  function openHostPort(e: MouseEvent, hostPort: string) {
    e.preventDefault();
    void openUrl(`http://localhost:${hostPort}`);
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
          label: "コピー",
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
    <div class="placeholder">左の一覧からコンテナを選択してください。</div>
  {:else}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
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
      oncontextmenu={handleValueContextMenu}
    >
      {#if errorMessage}
        <div class="error-banner">{errorMessage}</div>
      {:else if loading}
        <LoadingState message="読み込み中..." />
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
                    <a class="ext-link" href={imageUrl} onclick={(e) => openExternal(e, imageUrl)}>{repo}</a>{tag ? `:${tag}` : ""}
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
            {#if detail.ip_address}
              <tr>
                <th>IP</th>
                <td><CopyableValue value={detail.ip_address}>{detail.ip_address}</CopyableValue></td>
              </tr>
            {/if}
          </tbody>
        </table>

        {#if detail.ports.length > 0}
          <h3>Port Forwards</h3>
          <table class="data-table">
            <thead>
              <tr><th>Host Port</th><th>Container Port</th><th>Protocol</th></tr>
            </thead>
            <tbody>
              {#each detail.ports as p}
                <tr>
                  <td>
                    <CopyableValue value={p.host_port}>
                      <a
                        class="ext-link"
                        href={`http://localhost:${p.host_port}`}
                        onclick={(e) => openHostPort(e, p.host_port)}
                      >{p.host_port}</a>
                    </CopyableValue>
                  </td>
                  <td><CopyableValue value={p.container_port}>{p.container_port}</CopyableValue></td>
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
              {#each detail.mounts as m}
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
              {#each Object.entries(detail.labels) as [k, v]}
                <tr>
                  <td><CopyableValue value={k}>{k}</CopyableValue></td>
                  <td><CopyableValue value={v}>{v}</CopyableValue></td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      {:else if activeTab === "logs" && containerId}
        <LogViewer {containerId} />
      {:else if activeTab === "terminal" && containerId}
        <TerminalSession {containerId} />
      {:else if activeTab !== "info"}
        <p class="hint">近日対応予定です。</p>
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
