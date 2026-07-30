<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import { listContainers, containerAction } from "$lib/ipc/containers";
  import { composeAction, type ComposeActionKind } from "$lib/ipc/compose";
  import { setupCurrentDistro } from "$lib/ipc/setup";
  import { ensureConnected } from "$lib/connection";
  import ContainerMasterList from "$lib/components/ContainerMasterList.svelte";
  import ContainerDetailPanel from "$lib/components/ContainerDetailPanel.svelte";
  import LoadingState from "$lib/components/LoadingState.svelte";
  import { pushToast, resolveToast } from "$lib/stores/toasts";
  import type { ContainerSummary } from "$lib/types";

  const actionLabels: Record<string, string> = {
    start: "開始",
    stop: "停止",
    restart: "再起動",
    remove: "削除",
    pause: "一時停止",
    unpause: "再開",
  };

  let containers = $state<ContainerSummary[]>([]);
  let selectedId = $state<string | null>(null);
  let distro = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let loading = $state(true);
  let pollHandle: ReturnType<typeof setInterval> | undefined;

  async function refresh() {
    try {
      containers = await listContainers(true);
      errorMessage = null;
      if (selectedId && !containers.some((c) => c.id === selectedId)) {
        selectedId = null;
      }
    } catch (e) {
      errorMessage = String(e);
    }
  }

  async function connectAndLoad() {
    loading = true;
    try {
      const connected = await ensureConnected();
      if (!connected) {
        await goto("/setup");
        return;
      }
      distro = await setupCurrentDistro();
      await refresh();
    } catch (e) {
      errorMessage = String(e);
    } finally {
      loading = false;
    }
  }

  async function runAction(id: string, action: Parameters<typeof containerAction>[1]) {
    const label = actionLabels[action] ?? action;
    const name = containers.find((c) => c.id === id)?.names.join(", ") ?? id.slice(0, 12);
    const toastId = pushToast(`${name} を${label}しています...`);
    try {
      await containerAction(id, action);
      await refresh();
      resolveToast(toastId, "success", `${name} を${label}しました`);
    } catch (e) {
      resolveToast(toastId, "error", `${name} の${label}に失敗しました: ${String(e)}`);
    }
  }

  const composeActionLabels: Record<ComposeActionKind, string> = {
    up: "開始",
    stop: "停止",
    down: "削除",
  };

  async function runComposeAction(project: string, configFiles: string[], action: ComposeActionKind) {
    const label = composeActionLabels[action];
    const toastId = pushToast(`${project} を${label}しています...`);
    try {
      await composeAction(project, configFiles, action);
      await refresh();
      resolveToast(toastId, "success", `${project} を${label}しました`);
    } catch (e) {
      resolveToast(toastId, "error", `${project} の${label}に失敗しました: ${String(e)}`);
    }
  }

  onMount(() => {
    connectAndLoad();
    pollHandle = setInterval(refresh, 5000);
  });

  onDestroy(() => {
    if (pollHandle) clearInterval(pollHandle);
  });
</script>

<div class="containers-view">
  <div class="header-row">
    <h1>コンテナ</h1>
    {#if distro}
      <span class="distro-badge dockl-surface">{distro}</span>
    {/if}
  </div>

  {#if errorMessage}
    <div class="error-banner dockl-surface">{errorMessage}</div>
  {/if}

  {#if loading}
    <LoadingState message="WSL2に接続中..." />
  {:else}
    <div class="master-detail">
      <ContainerMasterList {containers} bind:selectedId onAction={runAction} onComposeAction={runComposeAction} />
      <ContainerDetailPanel containerId={selectedId} />
    </div>
  {/if}
</div>

<style>
  .containers-view {
    display: flex;
    flex-direction: column;
    gap: 12px;
    height: 100%;
    min-height: 0;
  }

  .header-row {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }

  h1 {
    font-size: 20px;
    font-weight: 600;
    margin: 0;
  }

  .distro-badge {
    padding: 2px 10px;
    font-size: 12px;
    color: var(--dockl-text-secondary);
  }

  .error-banner {
    padding: 8px 12px;
    color: var(--dockl-danger);
    border-color: var(--dockl-danger);
  }

  .master-detail {
    display: flex;
    gap: 12px;
    flex: 1;
    min-height: 0;
  }
</style>
