<script lang="ts">
  /**
   * A placeholder block standing in for a piece of content that hasn't arrived yet.
   *
   * Preferred over a spinner wherever the final layout is already known: the placeholder
   * occupies the space the real content will, so nothing jumps when it lands. A spinner
   * has no size of its own and has to be centered in whatever it's given, which is why
   * the stats view previously needed a rule pinning it down to stop the storage section
   * below it from sliding around.
   *
   * Renders inline, so standing in for a line of text means putting one *inside* the
   * element that text would have lived in:
   *
   * ```svelte
   * <div class="stat-label"><Skeleton width="28px" /></div>
   * ```
   *
   * That element keeps its own font-size and line-height, so the row ends up exactly as
   * tall as it will be once the text arrives. Sizing a bare block to a guessed pixel
   * height instead gets this wrong every time — the box a line of text occupies is
   * `font-size * line-height`, not `font-size`.
   */
  let {
    width = "100%",
    height = "1em",
    radius = "4px",
  }: {
    width?: string;
    /** Defaults to `1em`, i.e. the font size of whatever element this sits in. */
    height?: string;
    /** Corner rounding. Pass "50%" for something standing in for a circular element. */
    radius?: string;
  } = $props();
</script>

<!-- aria-hidden: this is decoration standing in for content that isn't there yet. The
     container that swaps it for real data is what should carry aria-busy, so a screen
     reader hears "busy" once instead of reading out a pile of empty boxes. -->
<div
  class="skeleton"
  style="width: {width}; height: {height}; border-radius: {radius}"
  aria-hidden="true"
></div>

<style>
  .skeleton {
    /* Inline-block so a text placeholder sits in a real line box and inherits its height
       from the parent, rather than dictating one. Inside a flex container (a chart slot,
       say) this is ignored and it behaves as a block, which is what's wanted there. */
    display: inline-block;
    vertical-align: middle;
    flex-shrink: 0;
    /* Three stops rather than two so the highlight is a band travelling across the block,
       not a hard edge sweeping over it. The oversized background is what there is to
       move: at 100% there'd be nothing to slide. */
    background: linear-gradient(
      90deg,
      var(--dockl-skeleton) 25%,
      var(--dockl-skeleton-shimmer) 37%,
      var(--dockl-skeleton) 63%
    );
    background-size: 400% 100%;
    animation: skeleton-shimmer 1.4s ease infinite;
  }

  @keyframes skeleton-shimmer {
    from {
      background-position: 100% 50%;
    }
    to {
      background-position: 0 50%;
    }
  }

  /* Still a placeholder, just a still one — the layout it reserves is the point, and the
     movement is not worth triggering discomfort for anyone who has asked it to stop. */
  @media (prefers-reduced-motion: reduce) {
    .skeleton {
      background: var(--dockl-skeleton);
      animation: none;
    }
  }
</style>
