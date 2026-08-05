<script lang="ts">
  import type { Snippet } from "svelte";
  import ResizeHandle from "$lib/components/layout/ResizeHandle.svelte";

  /**
   * The list-beside-detail shell every resource page uses.
   *
   * Exists so the four pages can't drift apart on the things that should be identical —
   * the divider between the panes, the absence of a flex gap around it (the handle *is*
   * the gap, so all of it is draggable), and the fact that both panes are always
   * rendered. That last point matters: a page that swaps this whole area out for a
   * spinner throws away the detail the user was reading, so loading belongs inside the
   * list instead (see MasterList).
   */
  let { list, detail }: { list: Snippet; detail: Snippet } = $props();
</script>

<div class="master-detail">
  {@render list()}
  <ResizeHandle />
  {@render detail()}
</div>

<style>
  .master-detail {
    display: flex;
    /* No gap: ResizeHandle provides the separation and every pixel of it is draggable. */
    gap: 0;
    flex: 1;
    min-height: 0;
  }
</style>
