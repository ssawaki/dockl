<script lang="ts">
  import type { ContainerSummary, DetailTabId } from "$lib/types";
  import CopyableValue from "$lib/components/ui/CopyableValue.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import LogViewer from "$lib/components/terminal/LogViewer.svelte";
  import { rovingFocus } from "$lib/actions/rovingFocus";
  import { t } from "$lib/stores/i18n";
  import stopIcon from "@fluentui/svg-icons/icons/stop_16_filled.svg?raw";
  import playIcon from "@fluentui/svg-icons/icons/play_16_filled.svg?raw";

  // `activeTab` is bound from the parent route (shared with ContainerDetailPanel) so
  // that switching from a container to this Compose project — where "Stats"/"Terminal"
  // don't apply — and back restores whatever tab was active before, rather than always
  // resetting to "Info". See +page.svelte.
  let {
    project,
    containers,
    configFiles,
    onSelectContainer,
    activeTab = $bindable(),
  }: {
    project: string;
    containers: ContainerSummary[];
    configFiles: string[];
    onSelectContainer: (id: string) => void;
    activeTab: DetailTabId;
  } = $props();

  // The rows below are keyed by path, and `config_files` comes straight from a Docker
  // label rather than from anything that guarantees uniqueness — a repeated path would
  // be a duplicate-key error, and listing the same file twice says nothing anyway.
  let uniqueConfigFiles = $derived([...new Set(configFiles)]);

  // Only "Info" and "Logs" make sense for a whole Compose project (a project has no
  // single set of stats or a single shell to attach to) — the other two stay visible
  // for tab-position continuity with ContainerDetailPanel, but are inert.
  const disabledTabs: ReadonlySet<DetailTabId> = new Set(["stats", "terminal"]);

  let tabs = $derived<{ id: DetailTabId; label: string }[]>([
    { id: "info", label: $t("containers.tab.info") },
    { id: "stats", label: $t("containers.tab.stats") },
    { id: "logs", label: $t("containers.tab.logs") },
    { id: "terminal", label: $t("containers.tab.terminal") },
  ]);

  function selectTab(id: DetailTabId) {
    if (disabledTabs.has(id)) return;
    activeTab = id;
  }

  let runningCount = $derived(containers.filter((c) => c.state === "running").length);
  let status: "running" | "partial" | "stopped" = $derived(
    runningCount === 0 ? "stopped" : runningCount === containers.length ? "running" : "partial",
  );
  let statusLabel = $derived(
    status === "running"
      ? $t("containers.status.allRunning")
      : status === "partial"
        ? $t("containers.status.partiallyRunning")
        : $t("containers.section.stopped"),
  );

  // Compose's config_files label holds full absolute paths, which share the project's
  // working directory — shown separately here (always, regardless of file count) so
  // Config Files can list just the filenames instead of repeating that long prefix on
  // every row. The full path is still one click away via CopyableValue either way.
  let workingDir = $derived(containers[0]?.labels["com.docker.compose.project.working_dir"] ?? "");

  function relativeConfigFile(file: string): string {
    return workingDir && file.startsWith(`${workingDir}/`)
      ? file.slice(workingDir.length + 1)
      : file;
  }
</script>

<div class="detail-panel dockl-surface">
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
        class:disabled={disabledTabs.has(tab.id)}
        onclick={() => selectTab(tab.id)}
        role="tab"
        aria-selected={activeTab === tab.id}
        aria-disabled={disabledTabs.has(tab.id)}
        data-roving-item
        tabindex="-1"
      >
        {tab.label}
      </div>
    {/each}
  </div>

  <div class="tab-content" class:flush={activeTab === "logs"}>
    {#if disabledTabs.has(activeTab)}
      <div class="placeholder">{$t("containers.detail.placeholder")}</div>
    {:else if activeTab === "logs"}
      {#key project}
        <LogViewer {project} {configFiles} isRunning={runningCount > 0} />
      {/key}
    {:else}
      <div class="header">
        <h2>{project}</h2>
        <span
          class="status-badge"
          class:running={status === "running"}
          class:partial={status === "partial"}
        >
          <span class="status-dot"></span>
          {statusLabel}
        </span>
      </div>

      {#if workingDir}
        <h3>Working Directory</h3>
        <table class="info-table">
          <tbody>
            <tr>
              <td><CopyableValue value={workingDir}>{workingDir}</CopyableValue></td>
            </tr>
          </tbody>
        </table>
      {/if}

      {#if uniqueConfigFiles.length > 0}
        <h3>{uniqueConfigFiles.length > 1 ? "Config Files" : "Config File"}</h3>
        <table class="info-table">
          <tbody>
            {#each uniqueConfigFiles as file (file)}
              <tr>
                <td><CopyableValue value={file}>{relativeConfigFile(file)}</CopyableValue></td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}

      <h3>Containers ({containers.length})</h3>
      <table class="data-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Image</th>
            <th>Status</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each containers as c (c.id)}
            <tr class="container-row" onclick={() => onSelectContainer(c.id)}>
              <td>{c.names.join(", ")}</td>
              <td>{c.image}</td>
              <td>
                <span class="row-dot" class:running={c.state === "running"}></span>
                {c.status}
              </td>
              <td class="actions-cell">
                {#if c.state === "running"}
                  <Icon svg={stopIcon} size={13} />
                {:else}
                  <Icon svg={playIcon} size={13} />
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>

<style>
  .detail-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
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

  .tab.disabled {
    cursor: not-allowed;
    opacity: 0.4;
  }

  .tab.disabled:not(.active):hover {
    color: var(--dockl-text-secondary);
  }

  .tab-content {
    flex: 1;
    overflow: auto;
    padding: 14px 16px;
  }

  /* Matches ContainerDetailPanel: LogViewer manages its own scrolling (xterm's internal
     scrollback), so it gets the full panel with no padding rather than sitting inside
     another scrollable, padded box. */
  .tab-content.flush {
    overflow: hidden;
    padding: 0;
  }

  .placeholder {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--dockl-text-secondary);
  }

  .header {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  h2 {
    font-size: 16px;
    font-weight: 600;
    margin: 0;
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--dockl-text-secondary);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--dockl-text-secondary);
  }

  .status-badge.running .status-dot {
    background: var(--dockl-success);
  }

  .status-badge.partial .status-dot {
    background: var(--dockl-warning);
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

  .data-table th {
    color: var(--dockl-text-secondary);
    font-weight: 500;
  }

  .container-row {
    cursor: pointer;
  }

  .container-row:hover {
    background: var(--dockl-surface-hover);
  }

  .row-dot {
    display: inline-block;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--dockl-text-secondary);
    margin-right: 4px;
  }

  .row-dot.running {
    background: var(--dockl-success);
  }

  .actions-cell {
    color: var(--dockl-text-secondary);
    text-align: right;
  }
</style>
