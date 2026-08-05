<script lang="ts">
  import { formatError } from "$lib/errors";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { setupListDistros, setupConnect } from "$lib/ipc/setup";
  import { persistConnectedDistro } from "$lib/connection";
  import { connection } from "$lib/stores/connection";
  import LoadingState from "$lib/components/ui/LoadingState.svelte";
  import { rovingFocus } from "$lib/actions/rovingFocus";
  import { t } from "$lib/stores/i18n";
  import type { DistroInfo } from "$lib/types";

  let distros = $state<DistroInfo[]>([]);
  let selected = $state<string | null>(null);
  let loading = $state(true);
  let connecting = $state(false);
  let errorMessage = $state<string | null>(null);

  onMount(async () => {
    try {
      distros = await setupListDistros();
      const wsl2 = distros.filter((d) => d.wsl_version === 2);
      selected = (wsl2.find((d) => d.is_default) ?? wsl2[0])?.name ?? null;
    } catch (e) {
      errorMessage = formatError(e);
    } finally {
      loading = false;
    }
  });

  async function connect() {
    if (!selected) return;
    connecting = true;
    errorMessage = null;
    try {
      await setupConnect(selected);
      await persistConnectedDistro(selected);
      // The root layout only checks the connection once at app startup — completing
      // setup here is the other way a fresh connection is established, so it needs to
      // update the same shared store or every page's `$connection.status === "connected"`
      // guard would wait forever.
      connection.set({ status: "connected", distro: selected });
      goto(resolve("/"));
    } catch (e) {
      errorMessage = formatError(e);
    } finally {
      connecting = false;
    }
  }
</script>

<div class="setup-view">
  <h1>{$t("setup.welcome")}</h1>
  <p class="lead">{$t("setup.lead")}</p>

  {#if errorMessage}
    <div class="error-banner dockl-surface">{errorMessage}</div>
  {/if}

  {#if loading}
    <LoadingState message={$t("setup.detecting")} />
  {:else if distros.filter((d) => d.wsl_version === 2).length === 0}
    <div class="dockl-surface empty-state">
      <p>{$t("setup.noneFound")}</p>
      <p class="hint">{$t("setup.noneFoundHint")}</p>
    </div>
  {:else}
    <div class="distro-list" role="radiogroup" use:rovingFocus={{ selector: "[data-roving-item]" }}>
      {#each distros.filter((d) => d.wsl_version === 2) as d (d.name)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div
          class="distro-card dockl-surface"
          class:selected={selected === d.name}
          onclick={() => (selected = d.name)}
          role="radio"
          aria-checked={selected === d.name}
          data-roving-item
          tabindex={selected === d.name ? 0 : -1}
        >
          <div class="distro-name">{d.name}</div>
          <div class="distro-meta">
            <span class="badge" class:running={d.is_running}
              >{d.is_running ? $t("setup.running") : $t("setup.stopped")}</span
            >
            {#if d.is_default}<span class="badge">{$t("setup.default")}</span>{/if}
          </div>
        </div>
      {/each}
    </div>

    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <fluent-button appearance="primary" disabled={!selected || connecting} onclick={connect}>
      <span class="btn-content">
        {#if connecting}
          <fluent-spinner size="tiny"></fluent-spinner>
        {/if}
        {connecting ? $t("common.connecting") : $t("setup.connect")}
      </span>
    </fluent-button>
  {/if}
</div>

<style>
  .setup-view {
    display: flex;
    flex-direction: column;
    gap: 14px;
    max-width: 520px;
  }

  h1 {
    font-size: 22px;
    font-weight: 600;
    margin: 0;
  }

  .lead {
    color: var(--dockl-text-secondary);
    margin: 0;
  }

  .hint {
    color: var(--dockl-text-secondary);
  }

  .error-banner {
    padding: 8px 12px;
    color: var(--dockl-danger);
    border-color: var(--dockl-danger);
  }

  .empty-state {
    padding: 16px;
  }

  .distro-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .distro-card {
    padding: 12px 14px;
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .distro-card.selected {
    border-color: var(--dockl-accent);
  }

  .distro-name {
    font-weight: 500;
  }

  .distro-meta {
    display: flex;
    gap: 6px;
  }

  .badge {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 10px;
    background: var(--dockl-surface-hover);
    color: var(--dockl-text-secondary);
  }

  .badge.running {
    color: var(--dockl-success);
  }

  .btn-content {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
</style>
