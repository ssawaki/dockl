<script lang="ts">
  import { onDestroy } from "svelte";

  import { copySilently } from "$lib/clipboard";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { t, type MessageKey } from "$lib/stores/i18n";
  import copyIcon from "@fluentui/svg-icons/icons/copy_20_regular.svg?raw";
  import checkmarkIcon from "@fluentui/svg-icons/icons/checkmark_20_regular.svg?raw";

  let {
    value,
    label = "common.copy",
    iconSize = 17,
    size = 26,
    focusable = true,
  }: {
    value: string;
    label?: MessageKey;
    iconSize?: number;
    /** Square button box, in px — match whatever sits next to it (e.g. a dismiss button). */
    size?: number;
    /**
     * `false` skips the button in tab order. Tables render one of these per cell, so
     * making them all tab stops would turn a keyboard walk through the page into
     * hundreds of steps; the value is still reachable via the right-click copy menu.
     */
    focusable?: boolean;
  } = $props();

  /** How long the checkmark stays up after a copy. */
  const CONFIRM_MS = 1500;
  /** Keep in sync with the `opacity` transition in this file's stylesheet. */
  const FADE_MS = 150;

  /** Drives which icon is drawn. */
  let copied = $state(false);
  /** Forces the button visible regardless of whether its row is hovered. */
  let confirming = $state(false);
  let fadeTimer: ReturnType<typeof setTimeout> | undefined;
  let iconTimer: ReturnType<typeof setTimeout> | undefined;

  async function copy(e: MouseEvent) {
    // Whatever sits under the button is usually clickable itself (a container row, a
    // detail cell) — copying must not also select it.
    e.stopPropagation();
    if (!(await copySilently(value))) return;

    clearTimeout(fadeTimer);
    clearTimeout(iconTimer);
    copied = true;
    confirming = true;

    fadeTimer = setTimeout(() => {
      // Drop the visibility hold first and swap the icon back only once the fade has
      // finished. Doing both at once makes a hover-revealed button flash the copy icon
      // on its way out, which reads as the confirmation being taken back.
      confirming = false;
      iconTimer = setTimeout(() => (copied = false), FADE_MS);
    }, CONFIRM_MS);
  }

  onDestroy(() => {
    clearTimeout(fadeTimer);
    clearTimeout(iconTimer);
  });
</script>

<button
  class="copy-icon-btn"
  class:copied
  class:confirming
  style={`--copy-btn-size: ${size}px`}
  onclick={copy}
  title={$t(label)}
  aria-label={$t(label)}
  tabindex={focusable ? 0 : -1}
>
  <Icon svg={copied ? checkmarkIcon : copyIcon} size={iconSize} />
</button>

<style>
  .copy-icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--copy-btn-size);
    height: var(--copy-btn-size);
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--dockl-text-secondary);
    cursor: pointer;
    flex-shrink: 0;
    /* Hover-reveal is the *parent's* call — a dense table hides these until the row is
       hovered (`--copy-btn-opacity: 0`), while a lone button in a dialog header stays
       visible. Custom properties cross Svelte's style scoping, so a parent sets the
       policy and the two rules below still override it where it would hide feedback the
       user needs to see. */
    opacity: var(--copy-btn-opacity, 1);
    /* Duration must stay in sync with `FADE_MS` above. */
    transition: opacity 150ms;
  }

  /* The checkmark is the only confirmation these buttons give (no toast follows), so it
     has to stay visible even after the pointer leaves — otherwise clicking and
     immediately moving away looks like nothing happened. */
  .copy-icon-btn.confirming,
  .copy-icon-btn:focus-visible {
    opacity: 1;
  }

  .copy-icon-btn:hover {
    background: var(--dockl-surface-hover);
    color: var(--dockl-text-primary);
  }

  .copy-icon-btn:focus-visible {
    outline: 2px solid var(--dockl-accent);
    outline-offset: 1px;
  }

  .copy-icon-btn.copied,
  .copy-icon-btn.copied:hover {
    color: var(--dockl-success);
  }
</style>
