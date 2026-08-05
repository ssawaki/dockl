<script lang="ts">
  import type { Snippet } from "svelte";
  import CopyIconButton from "$lib/components/ui/CopyIconButton.svelte";

  let { value, children }: { value: string; children: Snippet } = $props();
</script>

<span class="copyable-value">
  <span class="copyable-value-content">{@render children()}</span>
  <CopyIconButton {value} focusable={false} />
</span>

<style>
  .copyable-value {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    max-width: 100%;
    /* Tables put one of these in every cell, so the buttons stay invisible until the
       value is hovered. `CopyIconButton` reads this custom property and overrides it
       while it's showing its "copied" checkmark. */
    --copy-btn-opacity: 0;
  }

  /* Hover only — deliberately not `:focus-within`, which a *click* also satisfies: the
     button keeps DOM focus after being pressed, so the icon would stay stuck on screen
     after the pointer moved away. Keyboard focus is handled by `CopyIconButton`'s own
     `:focus-visible` rule, which a mouse click doesn't trigger. */
  .copyable-value:hover {
    --copy-btn-opacity: 1;
  }

  /* Plain (non-flex) wrapper so `gap` above only ever applies between this whole
     block and the copy button — not between individual nodes rendered inside
     (e.g. a link followed by a bare ":tag" text node), which would otherwise each
     become their own flex item and get a gap inserted between them too. */
  .copyable-value-content {
    min-width: 0;
  }
</style>
