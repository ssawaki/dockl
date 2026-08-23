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
    // Rounded, because both inputs are fractional at any display scale other than 100%:
    // `clientX`/`clientY` land between CSS pixels, and `getBoundingClientRect()` gives
    // sub-pixel dimensions. Text laid out on a half-pixel offset gets rasterised across
    // pixel boundaries, which reads as blurry rather than as a shifted menu.
    adjustedX = Math.round(Math.max(margin, nx));
    adjustedY = Math.round(Math.max(margin, ny));
  });

  function handleWindowClick(e: MouseEvent) {
    if (menuEl && !menuEl.contains(e.target as Node)) {
      onClose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
    // Inert rather than a way out: without this, Tab moves focus to whatever sits
    // behind the menu while the menu itself stays open. Escape is the way out; the
    // arrow keys move within (see `rovingFocus`).
    else if (e.key === "Tab") e.preventDefault();
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
      onpointerenter={(e) => e.currentTarget.focus()}
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
    min-width: 200px;
    /* No inline padding: items are full-bleed so their hover fill reaches both edges.
       The block padding is what keeps a square-cornered first/last item from colliding
       with the rounded corner below. */
    padding-block: 4px;
    padding-inline: 0;
    /* Solid, not translucent: this floats over arbitrary content (list rows, etc.),
       and letting that show through would hurt legibility. */
    background: var(--dockl-menu-bg);
    border: 1px solid var(--dockl-border);
    /* Tighter than --dockl-radius (8px), which is sized for panels — at this menu's
       scale it reads as a bubble rather than a menu. */
    border-radius: 4px;
    box-shadow: var(--dockl-menu-shadow);
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    font-size: 13px;
    /* Arrow, not the web's hand — native menus don't switch the cursor. */
    cursor: default;
    color: var(--dockl-text-primary);
  }

  /* Exactly one item is ever filled, and which pointing device gets to say so is decided
     here rather than in script.
     `:focus-visible` (not `:focus`) is the keyboard's: the first item is focused
     programmatically on open, so `:focus` would light it up on a plain right-click.
     The hover half is gated on *nothing* having keyboard focus, because a resting mouse
     fires no `pointerenter` — arrow keys would otherwise move the focus fill to a second
     item while the pointer kept its own lit underneath. */
  .menu-item:focus-visible,
  .context-menu:not(:has(:focus-visible)) .menu-item:hover {
    background: var(--dockl-menu-hover);
  }

  .menu-item.danger:focus-visible,
  .context-menu:not(:has(:focus-visible)) .menu-item.danger:hover {
    background: var(--dockl-menu-hover-danger);
  }

  /* The fill above stands in for the shared accent ring
     (theme.css's `[data-roving-item]:focus-visible`), which would otherwise double up
     on it. Menu items are the one roving group where the two coincide, because hover
     and keyboard selection are the same state here. */
  .menu-item:focus-visible {
    outline: none;
  }

  /* Icons stay secondary so the label carries the row, the way Fluent's own menus tint
     them. Excluded for danger items, where the red is the whole point of the styling. */
  .menu-item:not(.danger) :global(.icon) {
    color: var(--dockl-text-secondary);
  }

  .menu-item.danger {
    color: var(--dockl-danger-text);
  }

  .menu-separator {
    height: 1px;
    margin: 4px 6px;
    background: var(--dockl-border);
  }
</style>
