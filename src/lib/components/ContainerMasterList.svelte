<script lang="ts">
  import type { ContainerActionKind, ContainerSummary } from "$lib/types";
  import type { ComposeActionKind } from "$lib/ipc/compose";
  import Icon from "$lib/components/Icon.svelte";
  import ContextMenu, { type ContextMenuItem } from "$lib/components/ContextMenu.svelte";
  import { rovingFocus } from "$lib/actions/rovingFocus";
  import stopIcon from "@fluentui/svg-icons/icons/stop_16_filled.svg?raw";
  import playIcon from "@fluentui/svg-icons/icons/play_16_filled.svg?raw";
  import pauseIcon from "@fluentui/svg-icons/icons/pause_16_filled.svg?raw";
  import restartIcon from "@fluentui/svg-icons/icons/arrow_clockwise_16_filled.svg?raw";
  import deleteIcon from "@fluentui/svg-icons/icons/delete_16_regular.svg?raw";
  import chevronRightIcon from "@fluentui/svg-icons/icons/chevron_right_16_regular.svg?raw";

  interface ProjectEntry {
    kind: "project";
    name: string;
    containers: ContainerSummary[];
    running: boolean;
    configFiles: string[];
  }
  interface StandaloneEntry {
    kind: "standalone";
    container: ContainerSummary;
  }
  type Entry = ProjectEntry | StandaloneEntry;

  let {
    containers,
    selectedId = $bindable(null),
    onAction,
    onComposeAction,
  }: {
    containers: ContainerSummary[];
    selectedId: string | null;
    onAction: (id: string, action: ContainerActionKind) => void;
    onComposeAction: (project: string, configFiles: string[], action: ComposeActionKind) => void;
  } = $props();

  let collapsed = $state<Set<string>>(new Set());
  let collapseInitialized = false;

  function toggleGroup(name: string) {
    const next = new Set(collapsed);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    collapsed = next;
  }

  function buildEntries(list: ContainerSummary[]): { running: Entry[]; stopped: Entry[] } {
    const projects = new Map<string, ContainerSummary[]>();
    const standalone: ContainerSummary[] = [];

    for (const c of list) {
      const project = c.labels["com.docker.compose.project"];
      if (project) {
        if (!projects.has(project)) projects.set(project, []);
        projects.get(project)!.push(c);
      } else {
        standalone.push(c);
      }
    }

    const entries: Entry[] = [];
    for (const [name, group] of projects) {
      const configFilesRaw = group[0]?.labels["com.docker.compose.project.config_files"] ?? "";
      const configFiles = configFilesRaw.split(",").filter((f) => f.length > 0);
      entries.push({
        kind: "project",
        name,
        containers: group,
        running: group.some((c) => c.state === "running"),
        configFiles,
      });
    }
    for (const c of standalone) {
      entries.push({ kind: "standalone", container: c });
    }

    const isRunning = (e: Entry) => (e.kind === "project" ? e.running : e.container.state === "running");
    return {
      running: entries.filter(isRunning),
      stopped: entries.filter((e) => !isRunning(e)),
    };
  }

  let entries = $derived(buildEntries(containers));

  // Stopped project groups start collapsed (running ones stay expanded) so the list
  // isn't dominated by containers nobody is currently looking at. This only runs once
  // per initial data load, not on every poll refresh, so a group the user manually
  // expands/collapses afterwards won't keep getting reset.
  $effect(() => {
    if (!collapseInitialized && containers.length > 0) {
      collapseInitialized = true;
      const stoppedProjectNames = entries.stopped
        .filter((e): e is ProjectEntry => e.kind === "project")
        .map((e) => e.name);
      collapsed = new Set(stoppedProjectNames);
    }
  });

  function select(id: string) {
    selectedId = id;
  }

  function fireAction(e: MouseEvent, id: string, action: ContainerActionKind) {
    e.stopPropagation();
    onAction(id, action);
  }

  let contextMenu = $state<{ container: ContainerSummary; x: number; y: number } | null>(null);

  function openContextMenu(e: MouseEvent, c: ContainerSummary) {
    e.preventDefault();
    e.stopPropagation();
    selectedId = c.id;
    contextMenu = { container: c, x: e.clientX, y: e.clientY };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function buildMenuItems(c: ContainerSummary): ContextMenuItem[] {
    const items: ContextMenuItem[] = [];

    if (c.state === "running") {
      items.push({ label: "停止", icon: stopIcon, onClick: () => onAction(c.id, "stop") });
      items.push({ label: "再起動", icon: restartIcon, onClick: () => onAction(c.id, "restart") });
      items.push({ label: "一時停止", icon: pauseIcon, onClick: () => onAction(c.id, "pause") });
    } else if (c.state === "paused") {
      items.push({ label: "再開", icon: playIcon, onClick: () => onAction(c.id, "unpause") });
    } else {
      items.push({ label: "開始", icon: playIcon, onClick: () => onAction(c.id, "start") });
    }

    items.push({ label: "削除", icon: deleteIcon, onClick: () => onAction(c.id, "remove"), danger: true });
    return items;
  }

  function fireComposeAction(
    e: MouseEvent,
    project: string,
    configFiles: string[],
    action: ComposeActionKind,
  ) {
    e.stopPropagation();
    onComposeAction(project, configFiles, action);
  }
</script>

{#snippet row(c: ContainerSummary, indent: boolean)}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="row"
    class:indent
    class:selected={selectedId === c.id}
    onclick={() => select(c.id)}
    oncontextmenu={(e) => openContextMenu(e, c)}
    role="treeitem"
    aria-selected={selectedId === c.id}
    data-roving-item
    tabindex="-1"
  >
    <span class="dot" class:running={c.state === "running"}></span>
    <div class="row-text">
      <div class="row-name">{c.names.join(", ")}</div>
      <div class="row-image">{c.image}</div>
    </div>
    <div class="row-actions">
      {#if c.state === "running"}
        <button class="icon-btn" title="停止" onclick={(e) => fireAction(e, c.id, "stop")}>
          <Icon svg={stopIcon} size={14} />
        </button>
      {:else}
        <button class="icon-btn" title="開始" onclick={(e) => fireAction(e, c.id, "start")}>
          <Icon svg={playIcon} size={14} />
        </button>
      {/if}
      <button class="icon-btn" title="削除" onclick={(e) => fireAction(e, c.id, "remove")}>
        <Icon svg={deleteIcon} size={14} />
      </button>
    </div>
  </div>
{/snippet}

{#snippet entryView(e: Entry)}
  {#if e.kind === "project"}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="row group-row"
      onclick={() => toggleGroup(e.name)}
      role="treeitem"
      aria-expanded={!collapsed.has(e.name)}
      aria-selected="false"
      data-roving-item
      tabindex="-1"
    >
      <span class="chevron" class:open={!collapsed.has(e.name)}>
        <Icon svg={chevronRightIcon} size={12} />
      </span>
      <div class="row-text">
        <div class="row-name">{e.name}</div>
      </div>
      <div class="row-actions">
        {#if e.running}
          <button
            class="icon-btn"
            title="停止"
            onclick={(ev) => fireComposeAction(ev, e.name, e.configFiles, "stop")}
          >
            <Icon svg={stopIcon} size={14} />
          </button>
        {:else}
          <button
            class="icon-btn"
            title="開始"
            onclick={(ev) => fireComposeAction(ev, e.name, e.configFiles, "up")}
          >
            <Icon svg={playIcon} size={14} />
          </button>
        {/if}
        <button
          class="icon-btn"
          title="削除"
          onclick={(ev) => fireComposeAction(ev, e.name, e.configFiles, "down")}
        >
          <Icon svg={deleteIcon} size={14} />
        </button>
      </div>
    </div>
    {#if !collapsed.has(e.name)}
      {#each e.containers as c (c.id)}
        {@render row(c, true)}
      {/each}
    {/if}
  {:else}
    {@render row(e.container, false)}
  {/if}
{/snippet}

<div
  class="master-list dockl-surface"
  role="tree"
  use:rovingFocus={{ selector: "[data-roving-item]" }}
>
  {#if entries.running.length > 0}
    <div class="section-label">Running</div>
    {#each entries.running as e (e.kind === "project" ? e.name : e.container.id)}
      {@render entryView(e)}
    {/each}
  {/if}

  {#if entries.stopped.length > 0}
    <div class="section-label">Stopped</div>
    {#each entries.stopped as e (e.kind === "project" ? e.name : e.container.id)}
      {@render entryView(e)}
    {/each}
  {/if}

  {#if containers.length === 0}
    <p class="empty">コンテナが見つかりません。</p>
  {/if}
</div>

{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    items={buildMenuItems(contextMenu.container)}
    onClose={closeContextMenu}
  />
{/if}

<style>
  .master-list {
    width: 280px;
    flex-shrink: 0;
    overflow-y: auto;
    padding: 6px;
  }

  .section-label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--dockl-text-secondary);
    padding: 10px 8px 4px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 6px;
    cursor: pointer;
    min-height: 40px;
  }

  .row:hover {
    background: var(--dockl-surface-hover);
  }

  .row.selected {
    background: var(--dockl-surface-hover);
    outline: 1px solid var(--dockl-accent);
  }

  .row.indent {
    margin-left: 16px;
  }

  .group-row {
    font-weight: 500;
  }

  .chevron {
    font-size: 10px;
    color: var(--dockl-text-secondary);
    transition: transform 0.1s;
    width: 10px;
  }

  .chevron.open {
    transform: rotate(90deg);
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--dockl-text-secondary);
    flex-shrink: 0;
  }

  .dot.running {
    background: var(--dockl-success);
  }

  .row-text {
    flex: 1;
    min-width: 0;
  }

  .row-name {
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-image {
    font-size: 11px;
    color: var(--dockl-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-actions {
    display: flex;
    gap: 2px;
  }

  .icon-btn {
    border: none;
    background: transparent;
    color: var(--dockl-text-secondary);
    cursor: pointer;
    width: 22px;
    height: 22px;
    border-radius: 4px;
    font-size: 11px;
  }

  .icon-btn:hover {
    background: var(--dockl-surface-hover);
    color: var(--dockl-text-primary);
  }

  .empty {
    padding: 16px;
    color: var(--dockl-text-secondary);
    font-size: 13px;
  }
</style>
