<script lang="ts">
  import type { Snippet } from "svelte";

  let { title, children }: { title: string; children?: Snippet } = $props();
</script>

<div class="page-header">
  <h1>{title}</h1>
  {#if children}
    <div class="page-header-actions">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    flex-shrink: 0;
    /* Matches Fluent's default (medium) button min-height, so the heading sits at the
       same vertical position whether or not this page's header has action buttons next
       to it — without a shared fixed height, a page with buttons got a taller header
       row than a page without, visibly shifting the two headings relative to each other. */
    min-height: 32px;
  }

  h1 {
    font-size: 20px;
    font-weight: 600;
    margin: 0;
  }

  .page-header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
</style>
