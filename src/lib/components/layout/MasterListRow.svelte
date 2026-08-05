<script lang="ts">
  import type { Snippet } from "svelte";

  /**
   * One selectable row: a name, a line of secondary text under it, and trailing actions.
   *
   * Shared by the image/volume/network lists, which are the same shape. The container
   * list stays bespoke — its rows nest under Compose groups and carry several actions and
   * a status dot, so bending this into that shape would cost more than it saves.
   */
  let {
    selected = false,
    /** Renders the name in a muted style — a dangling image, a built-in network. */
    dim = false,
    name,
    onSelect,
    meta,
    actions,
  }: {
    selected?: boolean;
    dim?: boolean;
    name: string;
    onSelect: () => void;
    meta?: Snippet;
    actions?: Snippet;
  } = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="row"
  class:selected
  class:dim
  onclick={onSelect}
  role="treeitem"
  aria-selected={selected}
  data-roving-item
  tabindex="-1"
>
  <div class="row-text">
    <!-- `title` so a name too long for the list is still readable on hover. -->
    <span class="name" title={name}>{name}</span>
    {#if meta}
      <span class="meta">{@render meta()}</span>
    {/if}
  </div>
  {#if actions}
    <div class="actions">{@render actions()}</div>
  {/if}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    border-radius: 4px;
    cursor: pointer;
  }

  .row:hover {
    background: var(--dockl-surface-hover);
  }

  .row.selected {
    background: var(--dockl-surface-selected);
  }

  .row-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    flex: 1;
    /* Without this a long name refuses to shrink and pushes the actions out of the
       fixed-width list. */
    min-width: 0;
  }

  .name {
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row.dim .name {
    color: var(--dockl-text-secondary);
    font-style: italic;
  }

  .meta {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--dockl-text-secondary);
    /* Long secondary text truncates rather than growing the row. */
    min-width: 0;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }
</style>
