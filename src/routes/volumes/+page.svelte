<script lang="ts">
  import { formatError } from "$lib/errors";
  import { listVolumes, removeVolume, pruneVolumes } from "$lib/ipc/volumes";
  import { connection } from "$lib/stores/connection";
  import { refreshOnDockerEvents } from "$lib/dockerEvents.svelte";
  import { f5RefreshHandler } from "$lib/shortcuts";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import PageHeader from "$lib/components/layout/PageHeader.svelte";
  import MasterDetail from "$lib/components/layout/MasterDetail.svelte";
  import VolumeMasterList from "$lib/components/volumes/VolumeMasterList.svelte";
  import VolumeDetailPanel from "$lib/components/volumes/VolumeDetailPanel.svelte";
  import { pushToast, resolveToast } from "$lib/stores/toasts";
  import { get } from "svelte/store";
  import { t } from "$lib/stores/i18n";
  import broomIcon from "@fluentui/svg-icons/icons/broom_16_regular.svg?raw";
  import arrowClockwiseIcon from "@fluentui/svg-icons/icons/arrow_clockwise_16_regular.svg?raw";
  import type { VolumeSummary } from "$lib/types";

  let volumes = $state<VolumeSummary[]>([]);
  let loading = $state(true);
  let errorMessage = $state<string | null>(null);
  let pruning = $state(false);

  let selectedName = $state<string | null>(null);
  // Resolved from the live list rather than stored, so a refresh (or a removal) can't
  // leave the panel showing something that has since changed or gone.
  let selectedVolume = $derived(volumes.find((v) => v.name === selectedName) ?? null);

  async function refresh() {
    try {
      volumes = await listVolumes();
      errorMessage = null;
    } catch (e) {
      errorMessage = formatError(e);
    } finally {
      loading = false;
    }
  }

  // The root layout runs the actual WSL2/Docker connection check once at app startup;
  // this just loads this page's own data once that's confirmed, then reacts to
  // `volume` events (create/destroy) instead of polling.
  refreshOnDockerEvents(() => $connection.status === "connected", ["volume"], refresh);

  let confirmTarget = $state<VolumeSummary | null>(null);

  function requestRemove(volume: VolumeSummary) {
    confirmTarget = volume;
  }

  function closeConfirm() {
    confirmTarget = null;
  }

  async function confirmRemove() {
    const volume = confirmTarget;
    if (!volume) return;
    closeConfirm();

    const name = volume.name;
    const toastId = pushToast(get(t)("toast.remove.pending", { name }));
    try {
      await removeVolume(volume.name);
      await refresh();
      resolveToast(toastId, "success", get(t)("toast.remove.success", { name }));
    } catch (e) {
      resolveToast(toastId, "error", get(t)("toast.remove.error", { name, error: formatError(e) }));
    }
  }

  let pruneConfirmOpen = $state(false);
  // Off by default — see images/+page.svelte's pruneIncludeTagged: removing named
  // volumes destroys real data, so it should be an explicit opt-in each time.
  let pruneIncludeNamed = $state(false);
  let pruneCommand = $derived(`docker volume prune${pruneIncludeNamed ? " -a" : ""} -f`);

  function requestPrune() {
    pruneConfirmOpen = true;
  }

  function cancelPrune() {
    pruneConfirmOpen = false;
    pruneIncludeNamed = false;
  }

  async function confirmPrune() {
    pruneConfirmOpen = false;
    pruning = true;
    const resource = get(t)("resource.volumes");
    const toastId = pushToast(get(t)("prune.pending", { resource }));
    try {
      const summary = await pruneVolumes(pruneIncludeNamed);
      await refresh();
      resolveToast(toastId, "success", summary.trim() || get(t)("prune.success", { resource }));
    } catch (e) {
      resolveToast(toastId, "error", get(t)("prune.error", { error: formatError(e) }));
    } finally {
      pruning = false;
    }
  }
</script>

<svelte:window onkeydown={f5RefreshHandler(refresh)} />

<div class="page-view">
  <PageHeader title={$t("nav.volumes")}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <fluent-button appearance="outline" tabindex="-1" disabled={pruning} onclick={requestPrune}>
      <span class="btn-content">
        {#if pruning}
          <fluent-spinner size="tiny"></fluent-spinner>
        {:else}
          <Icon svg={broomIcon} size={14} />
        {/if}
        {$t("prune.button", { resource: $t("resource.volumes") })}
      </span>
    </fluent-button>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <fluent-button
      appearance="outline"
      icon-only
      tabindex="-1"
      title={$t("common.refresh")}
      aria-label={$t("common.refresh")}
      onclick={refresh}
    >
      <Icon svg={arrowClockwiseIcon} size={14} />
    </fluent-button>
  </PageHeader>

  {#if errorMessage}
    <div class="error-banner dockl-surface">{errorMessage}</div>
  {/if}

  <!-- The spinner lives inside the list rather than replacing this whole area: only
       the list is waiting on anything, and blanking the detail panel too would throw
       away what the user was reading. -->
  <MasterDetail>
    {#snippet list()}
      <VolumeMasterList {volumes} {loading} bind:selectedName onRemove={requestRemove} />
    {/snippet}
    {#snippet detail()}
      <VolumeDetailPanel volume={selectedVolume} />
    {/snippet}
  </MasterDetail>
</div>

{#if confirmTarget}
  <ConfirmDialog
    title={$t("volumes.confirmRemove.title")}
    message={$t("confirmRemove.messageInUse", { name: confirmTarget.name })}
    onConfirm={confirmRemove}
    onCancel={closeConfirm}
  />
{/if}

{#if pruneConfirmOpen}
  <ConfirmDialog
    title={$t("prune.button", { resource: $t("resource.volumes") })}
    confirmLabel={$t("action.prune")}
    message={$t("volumes.prune.message")}
    onConfirm={confirmPrune}
    onCancel={cancelPrune}
  >
    {#snippet extra()}
      <!-- svelte-ignore a11y_label_has_associated_control -->
      <label class="checkbox-row">
        <fluent-checkbox
          checked={pruneIncludeNamed}
          onchange={(e: Event) => (pruneIncludeNamed = (e.target as HTMLInputElement).checked)}
        ></fluent-checkbox>
        <span>{$t("volumes.prune.includeNamed")}</span>
      </label>
      <code class="command-preview">{pruneCommand}</code>
    {/snippet}
  </ConfirmDialog>
{/if}

<style>
  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .command-preview {
    display: block;
    margin-top: 10px;
    padding: 6px 8px;
    font-size: 12px;
    font-family: Consolas, "Cascadia Code", monospace;
    color: var(--dockl-text-secondary);
    background: var(--dockl-surface-hover);
    border-radius: 4px;
    white-space: nowrap;
    overflow-x: auto;
  }

  .btn-content {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .error-banner {
    padding: 8px 12px;
    color: var(--dockl-danger);
    border-color: var(--dockl-danger);
  }
</style>
