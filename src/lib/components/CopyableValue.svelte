<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon from "$lib/components/Icon.svelte";
  import copyIcon from "@fluentui/svg-icons/icons/copy_20_regular.svg?raw";
  import { copyToClipboard } from "$lib/clipboard";

  let { value, children }: { value: string; children: Snippet } = $props();

  function copy(e: MouseEvent) {
    e.stopPropagation();
    void copyToClipboard(value);
  }
</script>

<span class="copyable-value">
  <span class="copyable-value-content">{@render children()}</span>
  <button class="copy-icon-btn" onclick={copy} title="コピー" tabindex="-1">
    <Icon svg={copyIcon} size={17} />
  </button>
</span>

<style>
  .copyable-value {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    max-width: 100%;
  }

  /* Plain (non-flex) wrapper so `gap` above only ever applies between this whole
     block and the copy button — not between individual nodes rendered inside
     (e.g. a link followed by a bare ":tag" text node), which would otherwise each
     become their own flex item and get a gap inserted between them too. */
  .copyable-value-content {
    min-width: 0;
  }

  .copy-icon-btn {
    opacity: 0;
    border: none;
    background: transparent;
    color: var(--dockl-text-secondary);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: 4px;
    flex-shrink: 0;
    transition: opacity 0.1s;
  }

  .copyable-value:hover .copy-icon-btn,
  .copy-icon-btn:focus-visible {
    opacity: 1;
  }

  .copy-icon-btn:hover {
    background: var(--dockl-surface-hover);
    color: var(--dockl-text-primary);
  }
</style>
