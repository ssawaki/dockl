<script lang="ts">
  /**
   * Floating label for an element whose own text is hidden or truncated — chiefly the
   * sidebar rail while it's collapsed to icons.
   *
   * Mounted while it should be visible, the same way ContextMenu/ConfirmDialog are:
   * `{#if tip}<Tooltip anchor={tip.el} label={tip.label} onClose={...} />{/if}`.
   *
   * Rendered as a native popover rather than the plain fixed-position div ContextMenu
   * uses, because a tooltip is mounted *inside* the component that owns the anchor: the
   * sidebar rail clips its content with `overflow: hidden` (that's what hides the labels
   * while it's narrow), and a normal child element would be clipped right along with
   * them. A popover renders in the browser's top layer, so no ancestor's overflow or
   * z-index can touch it. `popover="manual"` (not `"auto"`) because auto popovers
   * light-dismiss each other — an auto one here would close any other popover-based UI
   * just by hovering a nav item.
   */
  export type TooltipPlacement = "right" | "left" | "top" | "bottom";

  let {
    anchor,
    label,
    placement = "right",
    onClose,
  }: {
    /** The element being labelled — the tooltip positions itself against its box. */
    anchor: HTMLElement;
    label: string;
    /** Preferred side. Flips to the opposite one if the tooltip wouldn't fit there. */
    placement?: TooltipPlacement;
    /** Called when something invalidates the tooltip's position (scroll/resize/Escape). */
    onClose?: () => void;
  } = $props();

  /** Distance between the anchor's edge and the tooltip. */
  const GAP = 8;
  /** Minimum distance kept from the window edges when clamping. */
  const MARGIN = 8;

  /**
   * The edge the open animation scales up from, per placement — always the one facing
   * the anchor, so the tooltip reads as growing out of what it labels.
   */
  const GROW_ORIGIN: Record<TooltipPlacement, string> = {
    right: "left center",
    left: "right center",
    top: "center bottom",
    bottom: "center top",
  };

  let tipEl: HTMLDivElement | undefined = $state();
  /** Null until the effect below has measured the tooltip — it isn't visible yet either. */
  let pos = $state<{ x: number; y: number; side: TooltipPlacement } | null>(null);

  $effect(() => {
    const el = tipEl;
    if (!el) return;

    // Read reactively so re-pointing at another anchor (or a changed label, which
    // changes the tooltip's width) re-runs the placement.
    const target = anchor;
    const preferred = placement;
    void label;

    // Showing first is what makes the element measurable at all: until then it's
    // `display: none` and every dimension reads as 0.
    if (!el.matches(":popover-open")) el.showPopover();

    const a = target.getBoundingClientRect();
    const t = el.getBoundingClientRect();
    const maxX = window.innerWidth - t.width - MARGIN;
    const maxY = window.innerHeight - t.height - MARGIN;

    // Flip to the opposite side when the preferred one would overflow the window — a
    // tooltip clamped back inside would otherwise sit on top of its own anchor.
    let s = preferred;
    if (s === "right" && a.right + GAP > maxX) s = "left";
    else if (s === "left" && a.left - GAP - t.width < MARGIN) s = "right";
    else if (s === "top" && a.top - GAP - t.height < MARGIN) s = "bottom";
    else if (s === "bottom" && a.bottom + GAP > maxY) s = "top";

    let nx: number;
    let ny: number;
    if (s === "right" || s === "left") {
      nx = s === "right" ? a.right + GAP : a.left - GAP - t.width;
      ny = a.top + (a.height - t.height) / 2;
    } else {
      nx = a.left + (a.width - t.width) / 2;
      ny = s === "bottom" ? a.bottom + GAP : a.top - GAP - t.height;
    }

    pos = {
      x: Math.round(Math.max(MARGIN, Math.min(nx, maxX))),
      y: Math.round(Math.max(MARGIN, Math.min(ny, maxY))),
      side: s,
    };
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose?.();
  }
</script>

<!-- Anything that moves the anchor out from under the tooltip invalidates its position;
     closing is more honest than leaving it pointing at empty space. Scroll is captured
     since it fires on the scrolling container, not on window. -->
<svelte:window
  onkeydown={handleKeydown}
  onresize={() => onClose?.()}
  onscrollcapture={() => onClose?.()}
/>

<div
  class="tooltip"
  bind:this={tipEl}
  popover="manual"
  role="tooltip"
  style="left: {pos?.x ?? 0}px; top: {pos?.y ?? 0}px; transform-origin: {GROW_ORIGIN[
    pos?.side ?? placement
  ]};"
>
  {label}
</div>

<style>
  .tooltip {
    position: fixed;
    /* Overrides the UA's own `[popover]` defaults (centered inset, auto margin, border,
       padding) — everything below is ours. */
    inset: auto;
    margin: 0;
    max-width: 240px;
    padding: 5px 9px;
    /* Solid, like ContextMenu and the dialogs: this floats over arbitrary content, and
       letting that show through would hurt legibility at this text size. */
    background: var(--dockl-menu-bg);
    color: var(--dockl-text-primary);
    border: 1px solid var(--dockl-border);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.28);
    font-size: 12px;
    line-height: 1.35;
    /* Never let the tooltip itself become a hover target: it sits right next to the
       anchor, and intercepting the pointer there would fire the anchor's pointerleave
       and immediately close it again. */
    pointer-events: none;
    /* A CSS animation rather than Svelte's `in:` transition: the element starts out
       `display: none` (a popover is hidden until `showPopover()`), and a transition
       driven from the moment of *mount* would already be partway through by the time
       it's actually visible. An animation simply starts when the element does. */
    animation: tooltip-in 140ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes tooltip-in {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .tooltip {
      animation: none;
    }
  }
</style>
