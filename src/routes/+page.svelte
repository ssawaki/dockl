<script lang="ts">
  import { formatError } from "$lib/errors";
  import { listContainers, containerAction } from "$lib/ipc/containers";
  import { composeAction, type ComposeActionKind } from "$lib/ipc/compose";
  import ContainerMasterList from "$lib/components/containers/ContainerMasterList.svelte";
  import ContainerDetailPanel from "$lib/components/containers/ContainerDetailPanel.svelte";
  import ComposeDetailPanel from "$lib/components/containers/ComposeDetailPanel.svelte";
  import MasterDetail from "$lib/components/layout/MasterDetail.svelte";
  import PageHeader from "$lib/components/layout/PageHeader.svelte";
  import { pushToast, resolveToast } from "$lib/stores/toasts";
  import { connection } from "$lib/stores/connection";
  import { refreshOnDockerEvents } from "$lib/dockerEvents.svelte";
  import type { ContainerSummary, ContainerActionKind, DetailTabId } from "$lib/types";
  import { get } from "svelte/store";
  import { t, type MessageKey } from "$lib/stores/i18n";

  let containers = $state<ContainerSummary[]>([]);
  let selectedId = $state<string | null>(null);
  let selectedProject = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let loading = $state(true);
  // Lifted here (rather than kept as local state in each detail panel) so it survives
  // switching between a container and a Compose project — the two panels are siblings
  // toggled by `{#if selectedProject}`, so only one is ever mounted at a time and either
  // one's own local state would reset every time the other is shown instead.
  let activeTab = $state<DetailTabId>("info");

  async function refresh() {
    try {
      containers = await listContainers(true);
      errorMessage = null;
      if (selectedId && !containers.some((c) => c.id === selectedId)) {
        selectedId = null;
      }
      if (selectedProject && !containers.some((c) => c.labels["com.docker.compose.project"] === selectedProject)) {
        selectedProject = null;
      }
    } catch (e) {
      errorMessage = formatError(e);
    } finally {
      loading = false;
    }
  }

  let selectedProjectContainers = $derived(
    selectedProject
      ? containers.filter((c) => c.labels["com.docker.compose.project"] === selectedProject)
      : [],
  );
  let selectedProjectConfigFiles = $derived(
    (selectedProjectContainers[0]?.labels["com.docker.compose.project.config_files"] ?? "")
      .split(",")
      .filter((f) => f.length > 0),
  );

  function selectContainerFromProject(id: string) {
    selectedProject = null;
    selectedId = id;
  }

  async function runAction(id: string, action: ContainerActionKind) {
    const name = containers.find((c) => c.id === id)?.names.join(", ") ?? id.slice(0, 12);
    const toastId = pushToast(get(t)(`toast.${action}.pending` as MessageKey, { name }));
    try {
      await containerAction(id, action);
      await refresh();
      resolveToast(toastId, "success", get(t)(`toast.${action}.success` as MessageKey, { name }));
    } catch (e) {
      resolveToast(toastId, "error", get(t)(`toast.${action}.error` as MessageKey, { name, error: formatError(e) }));
    }
  }

  async function runComposeAction(project: string, configFiles: string[], action: ComposeActionKind) {
    const toastId = pushToast(get(t)(`toast.${action}.pending` as MessageKey, { name: project }));
    try {
      const output = await composeAction(project, configFiles, action);
      await refresh();
      resolveToast(
        toastId,
        "success",
        get(t)(`toast.${action}.success` as MessageKey, { name: project }),
        output.trim() ? output : undefined,
      );
    } catch (e) {
      resolveToast(
        toastId,
        "error",
        get(t)(`toast.${action}.error` as MessageKey, { name: project, error: formatError(e) }),
        formatError(e),
      );
    }
  }

  // The root layout runs the actual WSL2/Docker connection check once at app startup
  // (which also starts the backend's `docker events` subscription); this just does the
  // initial load and starts reacting to it once that's confirmed (handles both order —
  // mounting after $connection is already "connected", or while it's still
  // "connecting"). Event-driven rather than polled: a container start/stop/die/etc.
  // refreshes this list within `watchDockerEvents`'s debounce window instead of up to
  // 5s late, without spending anything while nothing's actually changing.
  refreshOnDockerEvents(() => $connection.status === "connected", ["container"], refresh);
</script>

<div class="page-view">
  <PageHeader title={$t("nav.containers")} />

  {#if errorMessage}
    <div class="error-banner dockl-surface">{errorMessage}</div>
  {/if}

  <!-- The spinner lives inside the list rather than replacing this whole area: only the
       list is waiting on anything, and blanking the detail panel too would throw away
       what the user was reading. The app-wide "connecting to WSL2" state still covers
       everything (see +layout.svelte) — there, nothing can be shown yet. -->
  <MasterDetail>
    {#snippet list()}
      <ContainerMasterList
        {containers}
        {loading}
        bind:selectedId
        bind:selectedProject
        onAction={runAction}
        onComposeAction={runComposeAction}
      />
    {/snippet}
    {#snippet detail()}
      {#if selectedProject}
        <ComposeDetailPanel
          project={selectedProject}
          containers={selectedProjectContainers}
          configFiles={selectedProjectConfigFiles}
          onSelectContainer={selectContainerFromProject}
          bind:activeTab
        />
      {:else}
        <ContainerDetailPanel
          containerId={selectedId}
          liveState={containers.find((c) => c.id === selectedId)?.state ?? null}
          bind:activeTab
        />
      {/if}
    {/snippet}
  </MasterDetail>
</div>

<style>

  .error-banner {
    padding: 8px 12px;
    color: var(--dockl-danger);
    border-color: var(--dockl-danger);
  }

</style>
