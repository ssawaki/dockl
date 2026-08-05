<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import { rovingFocus, focusFirstRovingItem } from "$lib/actions/rovingFocus";

  export interface ContextMenuItem {
    label: string;
    icon: string;
    onClick: () => void;
    danger?: boolean;
    /** Renders a divider directly above this item. */
    separator?: boolean;
  }

  let {
    x,
    y,
    items,
    onClose,
  }: { x: number; y: number; items: ContextMenuItem[]; onClose: () => void } = $props();

  let menuEl: HTMLDivElement | undefined = $state();

  // Keep the menu on-screen even if it was opened near the window's right/bottom edge.
  // Reads `x`/`y` reactively (not just as an initial value) so re-opening the menu at a
  // new position while the component instance is reused still re-clamps correctly.
  let adjustedX = $state(0);
  let adjustedY = $state(0);

  $effect(() => {
    const targetX = x;
    const targetY = y;
    if (!menuEl) {
      adjustedX = targetX;
      adjustedY = targetY;
      return;
    }
    const rect = menuEl.getBoundingClientRect();
    const margin = 8;
    let nx = targetX;
    let ny = targetY;
    if (nx + rect.width > window.innerWidth - margin) nx = window.innerWidth - rect.width - margin;
    if (ny + rect.height > window.innerHeight - margin)
      ny = window.innerHeight - rect.height - margin;
    adjustedX = Math.max(margin, nx);
    adjustedY = Math.max(margin, ny);
  });

  function handleWindowClick(e: MouseEvent) {
    if (menuEl && !menuEl.contains(e.target as Node)) {
      onClose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }

  function select(item: ContextMenuItem) {
    item.onClick();
    onClose();
  }
</script>

<svelte:window
  onclick={handleWindowClick}
  oncontextmenu={handleWindowClick}
  onkeydown={handleKeydown}
  onblur={onClose}
/>

<div
  class="context-menu"
  bind:this={menuEl}
  style="left:{adjustedX}px; top:{adjustedY}px;"
  role="menu"
  use:rovingFocus={{ selector: "[data-roving-item]" }}
  use:focusFirstRovingItem={"[data-roving-item]"}
>
  {#each items as item (item.label)}
    {#if item.separator}
      <div class="menu-separator" role="separator"></div>
    {/if}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="menu-item"
      class:danger={item.danger}
      onclick={() => select(item)}
      role="menuitem"
      data-roving-item
      tabindex="-1"
    >
      <Icon svg={item.icon} size={15} />
      <span>{item.label}</span>
    </div>
  {/each}
</div>

<style>
  .context-menu {
    position: fixed;
    z-index: 2000;
    min-width: 170px;
    padding: 4px;
    /* Solid, not translucent: this floats over arbitrary content (list rows, etc.),
       and letting that show through would hurt legibility. */
    background: var(--dockl-menu-bg);
    border: 1px solid var(--dockl-border);
    border-radius: var(--dockl-radius);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 7px 10px;
    font-size: 13px;
    border-radius: 4px;
    cursor: pointer;
    color: var(--dockl-text-primary);
  }

  .menu-item:hover {
    background: var(--dockl-surface-hover);
  }

  .menu-item.danger {
    color: var(--dockl-danger);
  }

  .menu-separator {
    height: 1px;
    margin: 4px 6px;
    background: var(--dockl-border);
  }
</style>
