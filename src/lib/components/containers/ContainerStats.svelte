<script lang="ts">
  import { formatError } from "$lib/errors";
  import { onMount, onDestroy } from "svelte";
  import { getContainerStats, getHostCpuCount, getContainerDiskUsage } from "$lib/ipc/stats";
  import {
    parseStatsLine,
    parseDiskUsage,
    formatBytes,
    formatCores,
    type ContainerStatsPoint,
    type DiskUsage,
  } from "$lib/dockerStats";
  import LoadingState from "$lib/components/ui/LoadingState.svelte";
  import Sparkline from "$lib/components/ui/Sparkline.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { t } from "$lib/stores/i18n";
  import dismissCircleIcon from "@fluentui/svg-icons/icons/dismiss_circle_16_regular.svg?raw";

  // The parent wraps us in `{#key containerId}`, so one component instance only ever
  // handles a single containerId for its whole lifetime (mount -> destroy) — same
  // guarantee LogViewer/TerminalSession rely on.
  let {
    containerId,
    cpuLimitCores,
    isRunning,
  }: { containerId: string; cpuLimitCores: number | null; isRunning: boolean } = $props();

  const HISTORY_LIMIT = 60;
  const POLL_INTERVAL_MS = 2000;

  let history = $state<ContainerStatsPoint[]>([]);
  let errorMessage = $state<string | null>(null);

  // `docker stats` doesn't support a clean continuous-streaming mode under WSL (see
  // get_container_stats's doc comment), so this polls a `--no-stream` snapshot instead
  // of subscribing to an event stream.
  let pollHandle: ReturnType<typeof setTimeout> | undefined;
  let destroyed = false;

  // If the container has no `--cpus`/quota/cpuset limit, fall back to however many
  // cores the daemon itself sees — an unlimited container can use all of them.
  let hostCpuCount = $state<number | null>(null);
  let coresForMax = $derived(cpuLimitCores ?? hostCpuCount);

  async function loadHostCpuCountIfNeeded() {
    if (cpuLimitCores !== null) return;
    try {
      const count = await getHostCpuCount();
      if (!destroyed) hostCpuCount = count;
    } catch {
      // Best-effort: the CPU chart still works without a known ceiling, just less precisely scaled.
    }
  }

  // Disk usage doesn't change fast enough to justify polling it every 2s alongside
  // CPU/memory/IO, and (unlike `docker stats`) it's available even for a stopped
  // container, so it's fetched once up front rather than looping.
  let diskUsage = $state<DiskUsage | null>(null);

  async function loadDiskUsage() {
    try {
      const raw = await getContainerDiskUsage(containerId);
      if (!destroyed) diskUsage = parseDiskUsage(raw);
    } catch {
      // Best-effort: the rest of the stats view still works without this.
    }
  }

  /// Runs one sample, then schedules the next — deliberately not `setInterval`.
  ///
  /// A single `container_stats` call routinely takes longer than the interval: the daemon
  /// itself waits about a second between the two CPU samples it needs, on top of the
  /// transport (~0.8s more when shelling out). On a fixed interval those calls would
  /// overlap and pile up, and an older sample completing after a newer one would append
  /// out of order, making the sparkline run backwards. Chaining guarantees one in flight.
  async function poll() {
    try {
      const line = await getContainerStats(containerId);
      if (destroyed) return;
      const point = parseStatsLine(line);
      if (point) {
        errorMessage = null;
        history = [...history.slice(-(HISTORY_LIMIT - 1)), point];
      }
    } catch (e) {
      if (destroyed) return;
      // Most commonly this means the container stopped between polls — not worth
      // retrying, so stop and surface it rather than polling a dead container forever.
      errorMessage = formatError(e);
      return;
    }
    if (!destroyed) pollHandle = setTimeout(poll, POLL_INTERVAL_MS);
  }

  onMount(() => {
    void loadDiskUsage();
    // CPU/memory/IO/PIDs all come from `docker stats`, which only works for a running
    // container — skip polling it entirely rather than hitting (and displaying) the
    // same "container is not running" error on every 2s tick.
    if (isRunning) {
      void loadHostCpuCountIfNeeded();
      // `poll` schedules its own successor, so there's no interval to start here.
      void poll();
    }
  });

  onDestroy(() => {
    destroyed = true;
    if (pollHandle) clearTimeout(pollHandle);
  });

  let latest = $derived(history.at(-1) ?? null);
  let cpuHistory = $derived(history.map((h) => h.cpuPercent));
  let memHistory = $derived(history.map((h) => h.memPercent));

  // Docker's CPU% is relative to a single core, so a container using several cores can
  // read well above 100%. The chart's scale is normally the real ceiling (cores * 100),
  // but still grows past that if the data somehow exceeds it, rather than clipping.
  let cpuMaxPercent = $derived(coresForMax !== null ? coresForMax * 100 : 100);
  let cpuChartMax = $derived(Math.max(cpuMaxPercent, ...cpuHistory));
</script>

<div class="container-stats">
  {#if errorMessage}
    <div class="stats-banner error">{errorMessage}</div>
  {/if}

  {#if !isRunning}
    <div class="stats-banner">
      <Icon svg={dismissCircleIcon} size={14} />
      <span>{$t("stats.notRunningHint")}</span>
    </div>
  {/if}

  <div class="stats-body">
    {#if isRunning && latest}
      <div class="stat-grid">
        <div class="stat-card">
          <div class="stat-label">CPU</div>
          <div class="stat-value">{latest.cpuPercent.toFixed(1)}%</div>
          {#if coresForMax !== null}
            <div class="stat-sub">
              {$t("stats.max", {
                percent: cpuMaxPercent.toFixed(0),
                cores: formatCores(coresForMax),
              })}
            </div>
          {/if}
          <div class="stat-chart">
            <Sparkline
              data={cpuHistory}
              max={cpuChartMax}
              color="var(--dockl-accent)"
              height={72}
            />
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-label">{$t("stats.memory")}</div>
          <div class="stat-value">
            {formatBytes(latest.memUsedBytes)} / {formatBytes(latest.memLimitBytes)}
          </div>
          <div class="stat-chart">
            <Sparkline data={memHistory} max={100} color="var(--dockl-success)" height={72} />
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-label">{$t("stats.blockIO")}</div>
          <div class="stat-value">
            {$t("stats.blockIO.value", {
              read: formatBytes(latest.blockReadBytes),
              write: formatBytes(latest.blockWriteBytes),
            })}
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-label">{$t("stats.networkIO")}</div>
          <div class="stat-value">
            ↓{formatBytes(latest.netRxBytes)} ↑{formatBytes(latest.netTxBytes)}
          </div>
        </div>

        <div class="stat-card stat-card-wide">
          <div class="stat-label">PIDs</div>
          <div class="stat-value">{latest.pids}</div>
        </div>
      </div>
    {:else if isRunning && !errorMessage}
      <LoadingState message={$t("stats.loading")} />
    {/if}

    {#if diskUsage}
      {#if isRunning}
        <hr class="section-divider" />
      {/if}
      <div class="stat-card">
        <div class="stat-label">{$t("nav.storage")}</div>
        <div class="stat-value">{diskUsage.writableSize}</div>
        {#if diskUsage.virtualSize}
          <div class="stat-sub">{$t("stats.storageTotal", { size: diskUsage.virtualSize })}</div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .container-stats {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .stats-banner {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    font-size: 12px;
    color: var(--dockl-text-secondary);
    border-bottom: 1px solid var(--dockl-border);
    flex-shrink: 0;
  }

  .stats-banner.error {
    color: var(--dockl-danger);
  }

  .stats-body {
    flex: 1;
    overflow: auto;
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  /* LoadingState normally uses `flex: 1` to center itself in whatever space it's given
     (fine when it's the only thing in a panel) — but here it shares `.stats-body` with
     the storage section below it, so that same `flex: 1` was stretching it to fill all
     remaining height, shoving storage down to the very bottom. It then jumped back up
     the moment stats loaded and the grid (which doesn't stretch) replaced it. Pinning
     it to its natural size keeps storage's position stable throughout. */
  .stats-body :global(.loading-state) {
    flex: none;
    padding: 24px;
  }

  .stat-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
  }

  .stat-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 18px 20px;
    min-height: 150px;
    border: 1px solid var(--dockl-border);
    border-radius: var(--dockl-radius);
  }

  .stat-card-wide {
    grid-column: 1 / -1;
    min-height: auto;
  }

  .section-divider {
    border: none;
    border-top: 1px solid var(--dockl-border);
    margin: 0;
  }

  .stat-label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--dockl-text-secondary);
  }

  .stat-value {
    font-size: 15px;
    font-weight: 600;
  }

  .stat-sub {
    font-size: 11px;
    color: var(--dockl-text-secondary);
  }

  .stat-chart {
    flex: 1;
    display: flex;
    align-items: flex-end;
  }
</style>
