<script lang="ts">
  import { formatError } from "$lib/errors";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import type { ContainerActionKind, ContainerSummary } from "$lib/types";
  import type { ComposeActionKind } from "$lib/ipc/compose";
  import Icon from "$lib/components/ui/Icon.svelte";
  import ContextMenu, { type ContextMenuItem } from "$lib/components/ui/ContextMenu.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import MasterList from "$lib/components/layout/MasterList.svelte";
  import { connection } from "$lib/stores/connection";
  import { showToast } from "$lib/stores/toasts";
  import { get } from "svelte/store";
  import { SvelteSet } from "svelte/reactivity";
  import { t } from "$lib/stores/i18n";
  import stopIcon from "@fluentui/svg-icons/icons/stop_16_filled.svg?raw";
  import playIcon from "@fluentui/svg-icons/icons/play_16_filled.svg?raw";
  import pauseIcon from "@fluentui/svg-icons/icons/pause_16_filled.svg?raw";
  import restartIcon from "@fluentui/svg-icons/icons/arrow_clockwise_16_filled.svg?raw";
  import deleteIcon from "@fluentui/svg-icons/icons/delete_16_regular.svg?raw";
  import chevronRightIcon from "@fluentui/svg-icons/icons/chevron_right_16_regular.svg?raw";
  import layerIcon from "@fluentui/svg-icons/icons/layer_20_regular.svg?raw";
  import openFolderIcon from "@fluentui/svg-icons/icons/open_folder_16_regular.svg?raw";

  type ComposeStatus = "running" | "partial" | "stopped";

  interface ProjectEntry {
    kind: "project";
    name: string;
    containers: ContainerSummary[];
    running: boolean;
    /** Aggregate state across the project's containers, for the group row's status dot. */
    status: ComposeStatus;
    configFiles: string[];
  }
  interface StandaloneEntry {
    kind: "standalone";
    container: ContainerSummary;
  }
  type Entry = ProjectEntry | StandaloneEntry;

  let {
    containers,
    loading = false,
    selectedId = $bindable(null),
    selectedProject = $bindable(null),
    onAction,
    onComposeAction,
  }: {
    containers: ContainerSummary[];
    /** Only the initial fetch — later refreshes reuse the list already on screen. */
    loading?: boolean;
    selectedId: string | null;
    selectedProject: string | null;
    onAction: (id: string, action: ContainerActionKind) => void;
    onComposeAction: (project: string, configFiles: string[], action: ComposeActionKind) => void;
  } = $props();

  // A SvelteSet tracks its own mutations, so the membership checks below re-run without
  // having to swap in a fresh Set on every toggle.
  let collapsed = new SvelteSet<string>();
  let collapseInitialized = false;

  function toggleGroup(name: string) {
    if (collapsed.has(name)) collapsed.delete(name);
    else collapsed.add(name);
  }

  // ArrowLeft/ArrowRight collapse/expand the focused Compose group row, matching the
  // roving-tabindex's own ArrowUp/ArrowDown (which move focus, not state) without
  // conflicting with them — rovingFocus only handles Up/Down in vertical orientation.
  function handleGroupRowKeydown(e: KeyboardEvent, name: string) {
    if (e.key === "ArrowLeft" && !collapsed.has(name)) {
      e.preventDefault();
      toggleGroup(name);
    } else if (e.key === "ArrowRight" && collapsed.has(name)) {
      e.preventDefault();
      toggleGroup(name);
    }
  }

  function buildEntries(list: ContainerSummary[]): { running: Entry[]; stopped: Entry[] } {
    // Local scratch map for the grouping pass, discarded before this function returns —
    // nothing observes it, so a reactive SvelteMap would only add overhead.
    // eslint-disable-next-line svelte/prefer-svelte-reactivity
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
      const runningCount = group.filter((c) => c.state === "running").length;
      const status: ComposeStatus =
        runningCount === 0 ? "stopped" : runningCount === group.length ? "running" : "partial";
      entries.push({
        kind: "project",
        name,
        containers: group,
        running: runningCount > 0,
        status,
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
      collapsed.clear();
      for (const name of stoppedProjectNames) collapsed.add(name);
    }
  });

  function select(id: string) {
    selectedId = id;
    selectedProject = null;
  }

  function selectProject(name: string) {
    selectedProject = name;
    selectedId = null;
  }

  // Native `dblclick` only reliably pairs up the first two clicks of a rapid burst in
  // some browsers/webviews — a 4-click burst can yield one dblclick instead of two, so
  // 連打しても開閉が1回しか反映されない. Tracking click timestamps ourselves makes every
  // pair of clicks within the threshold toggle, so N rapid clicks always yield floor(N/2)
  // toggles.
  const DOUBLE_CLICK_MS = 400;
  let lastGroupRowClick: { name: string; time: number } | null = null;

  function handleGroupRowClick(e: MouseEvent, name: string) {
    selectProject(name);
    const now = e.timeStamp;
    if (lastGroupRowClick && lastGroupRowClick.name === name && now - lastGroupRowClick.time <= DOUBLE_CLICK_MS) {
      toggleGroup(name);
      lastGroupRowClick = null;
    } else {
      lastGroupRowClick = { name, time: now };
    }
  }

  function fireAction(e: MouseEvent, id: string, action: ContainerActionKind) {
    e.stopPropagation();
    onAction(id, action);
  }

  let confirmDialog = $state<{ title: string; message: string; onConfirm: () => void } | null>(
    null,
  );

  function closeConfirmDialog() {
    confirmDialog = null;
  }

  function requestRemoveContainer(e: MouseEvent | null, c: ContainerSummary) {
    e?.stopPropagation();
    confirmDialog = {
      title: get(t)("containers.confirmRemove.title"),
      message: get(t)("containers.confirmRemove.message", { name: c.names.join(", ") }),
      onConfirm: () => {
        onAction(c.id, "remove");
        closeConfirmDialog();
      },
    };
  }

  function requestComposeDown(e: MouseEvent | null, project: string, configFiles: string[]) {
    e?.stopPropagation();
    confirmDialog = {
      title: get(t)("containers.confirmComposeDown.title"),
      message: get(t)("containers.confirmComposeDown.message", { project }),
      onConfirm: () => {
        onComposeAction(project, configFiles, "down");
        closeConfirmDialog();
      },
    };
  }

  let contextMenu = $state<{ x: number; y: number; items: ContextMenuItem[] } | null>(null);

  function openContainerContextMenu(e: MouseEvent, c: ContainerSummary) {
    e.preventDefault();
    e.stopPropagation();
    select(c.id);
    contextMenu = { x: e.clientX, y: e.clientY, items: buildMenuItems(c) };
  }

  function openComposeContextMenu(e: MouseEvent, entry: ProjectEntry) {
    e.preventDefault();
    e.stopPropagation();
    selectProject(entry.name);
    contextMenu = { x: e.clientX, y: e.clientY, items: buildComposeMenuItems(entry) };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function buildMenuItems(c: ContainerSummary): ContextMenuItem[] {
    const items: ContextMenuItem[] = [];

    if (c.state === "running") {
      items.push({ label: get(t)("action.stop"), icon: stopIcon, onClick: () => onAction(c.id, "stop") });
      items.push({ label: get(t)("action.restart"), icon: restartIcon, onClick: () => onAction(c.id, "restart") });
      items.push({ label: get(t)("action.pause"), icon: pauseIcon, onClick: () => onAction(c.id, "pause") });
    } else if (c.state === "paused") {
      items.push({ label: get(t)("action.unpause"), icon: playIcon, onClick: () => onAction(c.id, "unpause") });
    } else {
      items.push({ label: get(t)("action.start"), icon: playIcon, onClick: () => onAction(c.id, "start") });
    }

    items.push({
      label: get(t)("action.remove"),
      icon: deleteIcon,
      onClick: () => requestRemoveContainer(null, c),
      danger: true,
    });
    return items;
  }

  // Compose config files live inside the WSL2 distro's own filesystem, so Explorer (a
  // Windows-native process) can't browse them by their Linux path directly — it has to
  // go through the `\\wsl.localhost\<distro>\...` UNC path Windows exposes for that.
  async function showComposeFileInExplorer(configFiles: string[]) {
    const distro = $connection.distro;
    const file = configFiles[0];
    if (!distro || !file) return;
    const windowsPath = `\\\\wsl.localhost\\${distro}${file.replace(/\//g, "\\")}`;
    try {
      await revealItemInDir(windowsPath);
    } catch (e) {
      showToast("error", get(t)("errors.explorerOpenFailed", { error: formatError(e) }));
    }
  }

  function buildComposeMenuItems(e: ProjectEntry): ContextMenuItem[] {
    const items: ContextMenuItem[] = [];

    if (e.running) {
      items.push({
        label: get(t)("action.stop"),
        icon: stopIcon,
        onClick: () => onComposeAction(e.name, e.configFiles, "stop"),
      });
      items.push({
        label: get(t)("action.restart"),
        icon: restartIcon,
        onClick: () => onComposeAction(e.name, e.configFiles, "restart"),
      });
    } else {
      items.push({
        label: get(t)("action.up"),
        icon: playIcon,
        onClick: () => onComposeAction(e.name, e.configFiles, "up"),
      });
    }

    items.push({
      label: get(t)("action.down"),
      icon: deleteIcon,
      onClick: () => requestComposeDown(null, e.name, e.configFiles),
      danger: true,
    });
    items.push({
      label: get(t)("containers.showInExplorer"),
      icon: openFolderIcon,
      onClick: () => void showComposeFileInExplorer(e.configFiles),
      separator: true,
    });
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
  <div
    class="row"
    class:indent
    class:selected={selectedId === c.id}
    onclick={() => select(c.id)}
    oncontextmenu={(e) => openContainerContextMenu(e, c)}
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
        <button class="icon-btn" tabindex="-1" title={$t("action.stop")} onclick={(e) => fireAction(e, c.id, "stop")}>
          <Icon svg={stopIcon} size={14} />
        </button>
      {:else}
        <button class="icon-btn" tabindex="-1" title={$t("action.start")} onclick={(e) => fireAction(e, c.id, "start")}>
          <Icon svg={playIcon} size={14} />
        </button>
      {/if}
      <button class="icon-btn" tabindex="-1" title={$t("action.remove")} onclick={(e) => requestRemoveContainer(e, c)}>
        <Icon svg={deleteIcon} size={14} />
      </button>
    </div>
  </div>
{/snippet}

{#snippet entryView(e: Entry)}
  {#if e.kind === "project"}
    <div
      class="row group-row"
      class:selected={selectedProject === e.name}
      onclick={(ev) => handleGroupRowClick(ev, e.name)}
      oncontextmenu={(ev) => openComposeContextMenu(ev, e)}
      onkeydown={(ev) => handleGroupRowKeydown(ev, e.name)}
      role="treeitem"
      aria-expanded={!collapsed.has(e.name)}
      aria-selected={selectedProject === e.name}
      data-roving-item
      tabindex="-1"
    >
      <button
        class="toggle-btn"
        tabindex="-1"
        onclick={(ev) => {
          ev.stopPropagation();
          toggleGroup(e.name);
        }}
        title={collapsed.has(e.name) ? $t("containers.expand") : $t("containers.collapse")}
        aria-label={collapsed.has(e.name)
          ? $t("containers.expandAriaLabel", { name: e.name })
          : $t("containers.collapseAriaLabel", { name: e.name })}
      >
        <span class="chevron" class:open={!collapsed.has(e.name)}>
          <Icon svg={chevronRightIcon} size={12} />
        </span>
      </button>
      <span
        class="compose-icon"
        class:running={e.status === "running"}
        class:partial={e.status === "partial"}
        title={e.status === "running"
          ? $t("containers.status.allRunning")
          : e.status === "partial"
            ? $t("containers.status.partiallyRunning")
            : $t("containers.section.stopped")}
      >
        <Icon svg={layerIcon} size={22} />
      </span>
      <div class="row-text">
        <div class="row-name">{e.name}</div>
      </div>
      <div class="row-actions">
        {#if e.running}
          <button
            class="icon-btn"
            tabindex="-1"
            title={$t("action.stop")}
            onclick={(ev) => fireComposeAction(ev, e.name, e.configFiles, "stop")}
          >
            <Icon svg={stopIcon} size={14} />
          </button>
        {:else}
          <button
            class="icon-btn"
            tabindex="-1"
            title={$t("action.up")}
            onclick={(ev) => fireComposeAction(ev, e.name, e.configFiles, "up")}
          >
            <Icon svg={playIcon} size={14} />
          </button>
        {/if}
        <button
          class="icon-btn"
          tabindex="-1"
          title={$t("action.down")}
          onclick={(ev) => requestComposeDown(ev, e.name, e.configFiles)}
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

<!-- Rows stay bespoke here (Compose groups nest, and each row carries several actions and
     a status dot), but the surface itself is the shared one, so this list can't drift
     from the others on width, scrolling, roving focus or the loading/empty ordering. -->
<MasterList
  {loading}
  empty={containers.length === 0}
  loadingLabel={$t("containers.loading")}
  emptyLabel={$t("containers.empty")}
>
  {#if entries.running.length > 0}
    <div class="section-label">{$t("containers.section.running")}</div>
    {#each entries.running as e (e.kind === "project" ? e.name : e.container.id)}
      {@render entryView(e)}
    {/each}
  {/if}

  {#if entries.stopped.length > 0}
    <div class="section-label">{$t("containers.section.stopped")}</div>
    {#each entries.stopped as e (e.kind === "project" ? e.name : e.container.id)}
      {@render entryView(e)}
    {/each}
  {/if}
</MasterList>

{#if contextMenu}
  <ContextMenu x={contextMenu.x} y={contextMenu.y} items={contextMenu.items} onClose={closeContextMenu} />
{/if}

{#if confirmDialog}
  <ConfirmDialog
    title={confirmDialog.title}
    message={confirmDialog.message}
    onConfirm={confirmDialog.onConfirm}
    onCancel={closeConfirmDialog}
  />
{/if}

<style>

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

  /* Tint rather than an accent outline: the outline was near-identical to the focus
     ring, so a focused row and a selected one looked the same. Selection is a fill,
     focus is a ring — see `[data-roving-item]:focus-visible` in theme.css. */
  .row.selected {
    background: var(--dockl-surface-selected);
  }

  .row.indent {
    margin-left: 16px;
  }

  .group-row {
    font-weight: 500;
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    align-self: stretch;
    flex-shrink: 0;
    border: none;
    background: transparent;
    color: var(--dockl-text-secondary);
    cursor: pointer;
    /* `.row`'s own padding (6px 8px) would otherwise inset this button from the row's
       true edges, shrinking its clickable area away from where the user's cursor
       naturally lands when aiming for the left/top/bottom of the row. Negative margin
       cancels that inset so the button's hit area reaches the real edges, while equal
       padding keeps the chevron's visual position unchanged. */
    margin: -6px 0 -6px -8px;
    padding: 6px 6px 6px 8px;
    border-radius: 6px 0 0 6px;
  }

  .toggle-btn:hover {
    background: var(--dockl-surface-hover-strong);
    color: var(--dockl-text-primary);
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

  .compose-icon {
    display: flex;
    flex-shrink: 0;
    color: var(--dockl-text-secondary);
  }

  .compose-icon.running {
    color: var(--dockl-success);
  }

  .compose-icon.partial {
    color: var(--dockl-warning);
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
    /* Centers the icon in the button's box. Without this the button falls back to
       inline-block and its icon sits on the text baseline, leaving the line's descender
       space below it — i.e. the icon rides visibly high. */
    display: flex;
    align-items: center;
    justify-content: center;
    /* Must be reset alongside `display: flex`: a <button>'s UA padding (1px 6px) eats
       into the fixed width via `box-sizing: border-box`, leaving a 10px content box that
       squashes the icon — as a flex item it shrinks to fit instead of overflowing the
       way it did while the button was inline-block. */
    padding: 0;
    border: none;
    background: transparent;
    color: var(--dockl-text-secondary);
    cursor: pointer;
    width: 22px;
    height: 22px;
    border-radius: 4px;
  }

  .icon-btn:hover {
    background: var(--dockl-surface-hover);
    color: var(--dockl-text-primary);
  }

</style>
