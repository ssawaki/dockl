<script lang="ts">
  import type { Snippet } from "svelte";

  /**
   * The pane beside a master list. Always rendered — even with nothing selected, where it
   * shows `placeholder` — so the layout doesn't jump as the selection comes and goes.
   */
  let {
    /** Shown instead of `children` when nothing is selected. */
    placeholder,
    empty = false,
    children,
  }: {
    placeholder: string;
    empty?: boolean;
    children: Snippet;
  } = $props();
</script>

<div class="detail-panel dockl-surface">
  {#if empty}
    <div class="placeholder">{placeholder}</div>
  {:else}
    <div class="body">{@render children()}</div>
  {/if}
</div>

<style>
  .detail-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }

  .placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 24px;
    font-size: 12px;
    color: var(--dockl-text-secondary);
    text-align: center;
  }

  .body {
    padding: 16px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>
