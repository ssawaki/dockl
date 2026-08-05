<script lang="ts">
  import {
    masterListWidth,
    setMasterListWidth,
    clampMasterListWidth,
  } from "$lib/stores/appearance";

  let dragging = $state(false);
  let startX = 0;
  let startWidth = 0;

  function onPointerDown(e: PointerEvent) {
    // Pointer capture rather than window listeners: the pointer routinely leaves this
    // 6px-wide element mid-drag, and capture keeps the events coming here anyway —
    // including the `pointerup` that ends the drag, which would otherwise be missed and
    // leave the divider stuck to the cursor.
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    dragging = true;
    startX = e.clientX;
    startWidth = $masterListWidth;
    e.preventDefault();
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    // The store is updated live so the layout tracks the cursor; only the persisted copy
    // waits for the drag to end.
    masterListWidth.set(clampMasterListWidth(startWidth + (e.clientX - startX)));
  }

  function onPointerUp(e: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
    void setMasterListWidth($masterListWidth);
  }
</script>

<!-- Not focusable, and `separator` rather than `slider`: a slider's contract is that it
     responds to arrow keys, which a control you can't focus never will. As a plain
     separator it's a divider that happens to be draggable — which is all it is to anyone
     not using a pointer. The width it sets isn't lost to them either: it only affects how
     much room the list gets, and both panes stay fully usable at any width. -->
<div
  class="resize-handle"
  class:dragging
  role="separator"
  aria-orientation="vertical"
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
></div>

<style>
  /* The element itself spans the full height so the whole seam is grabbable; only the
     short bar drawn by ::before is visible. A full-height line would read as a border
     between the panes rather than as something you can drag. */
  .resize-handle {
    flex-shrink: 0;
    /* This *is* the space between the panes — the surrounding flex `gap` is 0, so the
       whole separation is grabbable rather than a thin strip floating in dead space. */
    width: 12px;
    cursor: col-resize;
    /* Above both panes so the cursor changes over it rather than over their edges. */
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .resize-handle::before {
    content: "";
    /* Kept well inside the 12px handle so there's still a margin of grabbable area on
       either side of the visible grip. */
    width: 5px;
    height: 100px;
    /* Capped so the grip never outgrows a short window — on a small screen a fixed
       100px would reach both edges and become the full-height line this avoids. */
    max-height: 40%;
    /* Half the width, so the ends stay fully rounded as the bar thickens. */
    border-radius: 3px;
    background: transparent;
    transition: background 120ms;
  }

  .resize-handle:hover::before {
    /* Same grey the scrollbars use — it already reads as "a thing you can drag" and is
       defined for both themes. */
    background: var(--dockl-scrollbar-thumb);
  }

  .resize-handle.dragging::before {
    background: var(--dockl-scrollbar-thumb-hover);
  }
</style>
