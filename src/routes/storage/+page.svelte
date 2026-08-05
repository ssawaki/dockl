<script lang="ts">
  import { formatError } from "$lib/errors";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { getDiskUsage, pruneBuildCache } from "$lib/ipc/system";
  import { connection } from "$lib/stores/connection";
  import { refreshOnDockerEvents } from "$lib/dockerEvents.svelte";
  import { parseSize, formatBytes } from "$lib/dockerStats";
  import LoadingState from "$lib/components/ui/LoadingState.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import PageHeader from "$lib/components/layout/PageHeader.svelte";
  import { pushToast, resolveToast } from "$lib/stores/toasts";
  import { get } from "svelte/store";
  import { t, type MessageKey } from "$lib/stores/i18n";
  import broomIcon from "@fluentui/svg-icons/icons/broom_16_regular.svg?raw";
  import arrowClockwiseIcon from "@fluentui/svg-icons/icons/arrow_clockwise_16_regular.svg?raw";
  import chevronRightIcon from "@fluentui/svg-icons/icons/chevron_right_16_regular.svg?raw";
  import type { DiskUsageEntry } from "$lib/types";

  // Docker's own labels for `docker system df`'s "Type" column, mapped to this app's
  // vocabulary for the same resources (reusing the sidebar's nav.* keys where the
  // wording is identical).
  const kindKeys: Record<string, MessageKey> = {
    Images: "nav.images",
    Containers: "nav.containers",
    "Local Volumes": "storage.kind.localVolumes",
    "Build Cache": "storage.kind.buildCache",
  };

  // Images/Containers/Local Volumes already have a dedicated management page this row
  // can jump to; Build Cache doesn't, so it gets an inline prune action instead below.
  const kindHref: Record<string, string> = {
    Images: resolve("/images"),
    Containers: resolve("/"),
    "Local Volumes": resolve("/volumes"),
  };

  let entries = $state<DiskUsageEntry[]>([]);
  let loading = $state(true);
  let errorMessage = $state<string | null>(null);
  let pruning = $state(false);

  // The sum of each row's own "Size" total. Images/Containers/Volumes/Build Cache can
  // share underlying layers (e.g. an image's layers reused across containers), so this
  // isn't a perfectly non-overlapping partition of disk usage — but it's the same
  // approximation `docker system df` itself is built from, and matches what people
  // mean by "how much space is Docker using in total".
  let totalBytes = $derived(entries.reduce((sum, e) => sum + parseSize(e.size), 0));

  async function refresh() {
    try {
      entries = await getDiskUsage();
      errorMessage = null;
    } catch (e) {
      errorMessage = formatError(e);
    } finally {
      loading = false;
    }
  }

  // All four rows here (Images/Containers/Local Volumes/Build Cache) can shift from any
  // container/image/volume change, so this listens to all three kinds — `network`
  // events don't affect disk usage, so that one's deliberately left out.
  refreshOnDockerEvents(
    () => $connection.status === "connected",
    ["container", "image", "volume"],
    refresh,
  );

  let pruneConfirmOpen = $state(false);
  // Off by default, matching the image/volume prune dialogs: reclaiming cache Docker
  // would otherwise reuse for future builds is the more surprising/destructive scope.
  let pruneIncludeReusable = $state(false);
  let pruneCommand = $derived(`docker builder prune${pruneIncludeReusable ? " -a" : ""} -f`);

  function requestPruneBuildCache() {
    pruneConfirmOpen = true;
  }

  function cancelPrune() {
    pruneConfirmOpen = false;
    pruneIncludeReusable = false;
  }

  async function confirmPrune() {
    pruneConfirmOpen = false;
    pruning = true;
    const toastId = pushToast(get(t)("storage.pruneBuildCache.pending"));
    try {
      const summary = await pruneBuildCache(pruneIncludeReusable);
      await refresh();
      resolveToast(toastId, "success", summary.trim() || get(t)("storage.pruneBuildCache.success"));
    } catch (e) {
      resolveToast(toastId, "error", get(t)("prune.error", { error: formatError(e) }));
    } finally {
      pruning = false;
    }
  }
</script>

<div class="page-view">
  <PageHeader title={$t("nav.storage")}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <fluent-button
      appearance="outline"
      icon-only
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

  {#if loading}
    <LoadingState message={$t("storage.loading")} />
  {:else}
    <div class="total-bar dockl-surface">
      <div class="total-text">
        <span class="label">{$t("storage.totalLabel")}</span>
        <span class="hint">{$t("storage.totalHint")}</span>
      </div>
      <span class="total-value">{formatBytes(totalBytes)}</span>
    </div>
    <div class="table-wrap dockl-surface">
      <table>
        <thead>
          <tr>
            <th>{$t("storage.table.kind")}</th>
            <th>{$t("storage.table.count")}</th>
            <th>{$t("storage.table.active")}</th>
            <th>{$t("table.size")}</th>
            <th>{$t("storage.table.reclaimable")}</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each entries as entry (entry.kind)}
            {@const href = kindHref[entry.kind]}
            <!-- `kindHref` already stores resolve()d paths; the rule just can't follow the
                 value through the lookup. -->
            <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
            <tr class:linkable={!!href} onclick={() => href && goto(href)}>
              <td>{kindKeys[entry.kind] ? $t(kindKeys[entry.kind]) : entry.kind}</td>
              <td>{entry.total_count}</td>
              <td>{entry.active}</td>
              <td>{entry.size}</td>
              <td>{entry.reclaimable}</td>
              <td class="actions-cell">
                {#if href}
                  <Icon svg={chevronRightIcon} size={14} />
                {:else}
                  <button
                    class="icon-btn"
                    title={$t("storage.pruneBuildCache")}
                    disabled={pruning}
                    onclick={(e) => {
                      e.stopPropagation();
                      requestPruneBuildCache();
                    }}
                  >
                    <Icon svg={broomIcon} size={14} />
                  </button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

{#if pruneConfirmOpen}
  <ConfirmDialog
    title={$t("storage.pruneBuildCache")}
    confirmLabel={$t("action.prune")}
    message={$t("storage.pruneBuildCache.message")}
    onConfirm={confirmPrune}
    onCancel={cancelPrune}
  >
    {#snippet extra()}
      <!-- svelte-ignore a11y_label_has_associated_control -->
      <label class="checkbox-row">
        <fluent-checkbox
          checked={pruneIncludeReusable}
          onchange={(e: Event) => (pruneIncludeReusable = (e.target as HTMLInputElement).checked)}
        ></fluent-checkbox>
        <span>{$t("storage.pruneBuildCache.includeReusable")}</span>
      </label>
      <code class="command-preview">{pruneCommand}</code>
    {/snippet}
  </ConfirmDialog>
{/if}

<style>
  .error-banner {
    padding: 8px 12px;
    color: var(--dockl-danger);
    border-color: var(--dockl-danger);
  }

  .total-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
    padding: 12px 16px;
  }

  .total-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .total-bar .label {
    font-size: 13px;
    color: var(--dockl-text-secondary);
  }

  .total-bar .hint {
    font-size: 11px;
    color: var(--dockl-text-secondary);
  }

  .total-value {
    font-size: 20px;
    font-weight: 600;
  }

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

  .table-wrap {
    flex: 1;
    overflow: auto;
    padding: 6px;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  th {
    text-align: left;
    padding: 8px 12px;
    color: var(--dockl-text-secondary);
    font-weight: 500;
    font-size: 12px;
    white-space: nowrap;
  }

  td {
    padding: 8px 12px;
    border-top: 1px solid var(--dockl-border);
    white-space: nowrap;
  }

  tr.linkable {
    cursor: pointer;
  }

  tr.linkable:hover {
    background: var(--dockl-surface-hover);
  }

  .actions-cell {
    text-align: right;
    color: var(--dockl-text-secondary);
  }

  .icon-btn {
    /* Centers the icon in the button's box — without it the icon sits on the text
       baseline and rides visibly high. `inline-flex` rather than `flex` so the button
       stays an inline box and `.actions-cell`'s `text-align: right` still moves it. */
    display: inline-flex;
    align-items: center;
    justify-content: center;
    vertical-align: middle;
    /* Reset alongside the flex display: a <button>'s UA padding (1px 6px) eats into the
       fixed width via `box-sizing: border-box`, and as a flex item the icon shrinks to
       the leftover content box instead of overflowing it. */
    padding: 0;
    border: none;
    background: transparent;
    color: var(--dockl-text-secondary);
    cursor: pointer;
    width: 26px;
    height: 26px;
    border-radius: 4px;
  }

  .icon-btn:hover {
    background: var(--dockl-surface-hover);
    color: var(--dockl-text-primary);
  }
</style>
