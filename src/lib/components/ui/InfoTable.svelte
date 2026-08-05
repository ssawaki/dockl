<script lang="ts">
  import type { Snippet } from "svelte";

  /**
   * The label/value table detail panels use. A plain `<table>` rather than a grid so the
   * label column sizes itself to the longest label — which differs per locale, and would
   * otherwise need a hardcoded width that's wrong in at least one of them.
   */
  let { children }: { children: Snippet } = $props();
</script>

<table class="info-table">
  <tbody>{@render children()}</tbody>
</table>

<style>
  .info-table {
    border-collapse: collapse;
    font-size: 12px;
  }

  .info-table :global(th) {
    text-align: left;
    font-weight: 400;
    color: var(--dockl-text-secondary);
    padding: 3px 16px 3px 0;
    white-space: nowrap;
    vertical-align: top;
  }

  .info-table :global(td) {
    padding: 3px 0;
    /* Mount points and long IDs wrap instead of widening the panel past the window. */
    word-break: break-all;
  }
</style>
