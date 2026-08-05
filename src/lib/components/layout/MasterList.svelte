<script lang="ts">
  import type { Snippet } from "svelte";
  import LoadingState from "$lib/components/ui/LoadingState.svelte";
  import { rovingFocus } from "$lib/actions/rovingFocus";
  import { masterListWidth } from "$lib/stores/appearance";

  /**
   * The scrolling, resizable surface a resource list lives in.
   *
   * Owns the parts that must not differ between pages: the width (shared across every
   * master/detail page via `masterListWidth`, so dragging the divider on one page moves
   * it on all of them), arrow-key roving focus over `[data-roving-item]` rows, and the
   * order of the loading and empty states.
   */
  let {
    loading = false,
    empty = false,
    loadingLabel,
    emptyLabel,
    children,
  }: {
    /** Only the initial fetch — later refreshes reuse the list already on screen. */
    loading?: boolean;
    empty?: boolean;
    loadingLabel?: string;
    emptyLabel: string;
    children: Snippet;
  } = $props();
</script>

<div
  class="master-list dockl-surface"
  style="width: {$masterListWidth}px"
  role="tree"
  use:rovingFocus={{ selector: "[data-roving-item]" }}
>
  {@render children()}

  <!-- Loading is checked first so an empty result never briefly reads as "nothing here"
       before the first fetch has even come back. -->
  {#if loading}
    <LoadingState message={loadingLabel} />
  {:else if empty}
    <p class="empty">{emptyLabel}</p>
  {/if}
</div>

<style>
  .master-list {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    padding: 4px;
    gap: 1px;
  }

  /* A flex item shrinks to fit by default, so once the list is taller than the panel
     every row gets squashed instead of the list scrolling. Rows must keep their natural
     height and let the container scroll — which is what `overflow-y: auto` above is for. */
  .master-list > :global(*) {
    flex-shrink: 0;
  }

  .empty {
    margin: 0;
    padding: 12px 8px;
    font-size: 12px;
    color: var(--dockl-text-secondary);
  }
</style>
