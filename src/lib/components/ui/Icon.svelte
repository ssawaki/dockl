<script lang="ts">
  /**
   * Renders a raw Fluent System Icons SVG (from `@fluentui/svg-icons`, imported with
   * the `?raw` suffix). The source files don't set `fill`, so we inject
   * `fill="currentColor"` on the root <svg> to let CSS `color` control it, matching
   * how Fluent's own components tint their icons.
   */
  let { svg, size }: { svg: string; size?: number } = $props();

  let markup = $derived(svg.replace("<svg ", '<svg fill="currentColor" '));
</script>

<span class="icon" style={size ? `width:${size}px;height:${size}px` : undefined}>
  <!-- `svg` is always an icon file bundled at build time via `?raw`, never anything
       derived from Docker output or user input, so there is no injection vector here. -->
  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
  {@html markup}
</span>

<style>
  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 0;
  }

  .icon :global(svg) {
    display: block;
    width: 100%;
    height: 100%;
  }
</style>
