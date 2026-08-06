<script lang="ts">
  import type { Snippet } from "svelte";

  /**
   * The label/value table detail panels use. A plain `<table>` rather than a grid so the
   * label column sizes itself to the longest label — which differs per locale, and would
   * otherwise need a hardcoded width that's wrong in at least one of them.
   *
   * Sizing to the label is only half of it: the column also has to *stop* there, which is
   * what the `width: 1%` below is for. See its comment.
   */
  let { children }: { children: Snippet } = $props();
</script>

<table class="info-table">
  <tbody>{@render children()}</tbody>
</table>

<style>
  .info-table {
    /* Fills the pane so the value column has a fixed amount of room to work with. Left to
       shrink-to-fit, the whole table changed width with whatever value happened to be
       longest, so the layout moved from one selection to the next. */
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .info-table :global(th) {
    text-align: left;
    font-weight: 500;
    color: var(--dockl-text-secondary);
    /* The right padding is the gap between the label and its value — the columns sit flush
       otherwise, since `width: 1%` leaves the label column no slack of its own. */
    padding: 4px 60px 4px 0;
    white-space: nowrap;
    /* `baseline`, not `top`: a value cell holding a CopyableValue is as tall as its 26px
       copy button, well past the label's own line. Pinning the label to the top of that
       row while the value sits centred in it left the two lines visibly offset. Matching
       baselines aligns the text itself, and a wrapping value still aligns on its first
       line — what `top` was after. */
    vertical-align: baseline;
    /* Pins the label column to its own content. `table-layout: auto` otherwise splits the
       spare width in proportion to each column's contents, moving the label/value divide
       every time a value's length changed. A percentage the browser can't satisfy without
       clipping a nowrap cell falls back to the content width — "as wide as the longest
       label, no wider" — and unlike a fixed px value it still adapts per locale. */
    width: 1%;
  }

  .info-table :global(td) {
    padding: 4px 0;
    /* Every row the same height, whether or not its value is one worth copying. Only some
       are (an id, a mount path — not "yes" or a driver name), so before this the rows that
       carried a button stood taller than the rest and the spacing down the table came out
       uneven. 30px is the 22px content band plus the 4px padding above and below; `height`
       on a table cell acts as a minimum, so a value that wraps still grows past it. */
    height: 30px;
    /* Mount points and long IDs wrap instead of widening the panel past the window. */
    word-break: break-all;
    /* Table cells default to `middle`; see the matching note on `th`. */
    vertical-align: baseline;
    /* Values are worth selecting by hand (an id to paste elsewhere, part of a path); the
       app disables selection globally in `body`, so it has to be granted back here. */
    user-select: text;
    cursor: text;
  }

  /* Pulls the 26px copy button down to the height of a line of text, so it stops having
     any say in how the cell is laid out. -4px at each end is what it takes: anything less
     leaves the button the tallest thing in the value, which makes CopyableValue's box
     taller than the text it wraps — and since that box centres its contents, the text then
     sat a couple of pixels lower than in rows with no button, reading as "this one value
     is centred and the rest are top-aligned". Matching the text height puts every value on
     the same line regardless.

     The 4px it gives up at each end comes out of the cell's own padding, which is exactly
     4px, so the full 26px hit area survives without reaching the row above or below.
     Scoped to this table rather than done inside CopyableValue: elsewhere (the
     ports/mounts tables, the toasts) the button isn't competing with anything and has no
     reason to move. */
  .info-table :global(.copy-icon-btn) {
    margin-block: -4px;
  }
</style>
