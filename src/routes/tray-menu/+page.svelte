<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { emit } from "@tauri-apps/api/event";
  import { listContainers } from "$lib/ipc/containers";
  import type { ContainerSummary } from "$lib/types";

  // Ported from the previous native menu (see `tray::build_tray` on the Rust side, which
  // now just builds/positions/shows this popup rather than building menu items itself).
  let containers = $state<ContainerSummary[]>([]);

  async function refresh() {
    try {
      containers = (await listContainers(false)).filter((c) => c.state === "running");
    } catch {
      // Not connected yet (or the connection dropped) — an empty list is the right
      // fallback, not an error the tray popup has any way to surface.
      containers = [];
    }
  }

  // This popup window is rebuilt fresh on every open (see `tray::flyout::show` — closed
  // outright rather than kept alive hidden, so the running list can't go stale), so a
  // plain fetch on mount always reflects the current list without needing to listen for
  // anything. It's also what makes `.popup`'s CSS animation below replay on every open.
  onMount(() => {
    void refresh();
  });

  function openMain() {
    void invoke("tray_menu_open_main");
  }

  // Same event `+layout.svelte` already listens for from the old native menu — this just
  // moves who fires it.
  async function openSettings() {
    await emit("tray:open-settings");
    await invoke("tray_menu_open_main");
  }

  async function selectContainer(id: string) {
    await emit("tray:select-container", id);
    await invoke("tray_menu_open_main");
  }

  function quit() {
    void invoke("tray_menu_quit");
  }
</script>

<div class="wrapper">
  <div class="popup">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="item" onclick={openMain}>Docklを開く</div>

    {#if containers.length > 0}
      <div class="separator"></div>
      <div class="section-label">起動中のコンテナ</div>
      <div class="container-list">
        {#each containers as c (c.id)}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="item container-item" onclick={() => selectContainer(c.id)}>
            <span class="dot dockl-status-dot running"></span>
            <span class="container-name">{c.names.join(", ")}</span>
          </div>
        {/each}
      </div>
    {/if}

    <div class="separator"></div>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="item" onclick={openSettings}>設定</div>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="item" onclick={quit}>終了</div>
  </div>
</div>

<style>
  /* `Position::TrayCenter` (Rust side) places this *window* by its bottom edge, against
     the tray icon it sits above — sized to the window's full configured height regardless
     of how much content there actually is. `.popup` below is `height: auto`, so without
     this wrapper it renders from the window's top edge and leaves empty space under it,
     which visibly detaches the menu from the tray icon whenever there's little content.
     Pinning this to the bottom keeps `.popup`'s own bottom edge, not the window's, against
     the tray. Scoped to this element rather than `:global(body)` — a global body rule from
     one route's own stylesheet has no business reaching any other route's window. */
  .wrapper {
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    height: 100%;
  }

  .popup {
    display: flex;
    flex-direction: column;
    height: auto;
    max-height: 100%;
    box-sizing: border-box;
    padding: 6px;
    background: var(--dockl-menu-bg);
    border: 1px solid var(--dockl-border);
    border-radius: 6px;
    box-shadow: var(--dockl-menu-shadow);
    /* Scales from the bottom edge, closest to the tray icon this sits above
       (`Position::TrayCenter` on the Rust side), so it reads as growing out of the tray
       rather than out of empty space. */
    transform-origin: bottom center;
    animation: popup-in 100ms ease-out;
  }

  @keyframes popup-in {
    from {
      opacity: 0;
      transform: scale(0.96);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .item {
    flex-shrink: 0;
    padding: 6px 12px;
    border-radius: 4px;
    font-size: 13px;
    color: var(--dockl-text-primary);
    cursor: default;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item:hover {
    background: var(--dockl-menu-hover);
  }

  .container-item {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  /* This popup only ever lists running containers (see `refresh`'s filter), so `running`
     is hardcoded in the markup rather than bound to state — sized down from
     .dockl-status-dot's default 8px to fit this compact list (see theme.css). */
  .dot {
    width: 7px;
    height: 7px;
  }

  .container-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .separator {
    flex-shrink: 0;
    height: 1px;
    margin: 6px 4px;
    background: var(--dockl-border);
  }

  .section-label {
    flex-shrink: 0;
    padding: 2px 12px 4px;
    font-size: 11px;
    color: var(--dockl-text-secondary);
  }

  .container-list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }
</style>
