<script lang="ts">
  import type { XtermController } from "$lib/xterm/XtermController";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { t } from "$lib/stores/i18n";
  import chevronUpIcon from "@fluentui/svg-icons/icons/chevron_up_16_regular.svg?raw";
  import chevronDownIcon from "@fluentui/svg-icons/icons/chevron_down_16_regular.svg?raw";
  import dismissIcon from "@fluentui/svg-icons/icons/dismiss_16_regular.svg?raw";

  // Rendered by callers behind `{#if searchOpen}`, so one instance covers one open
  // search session — `controller.onSearchRequested` (Ctrl+F) toggles that flag rather
  // than being handled in here.
  let { controller, onClose }: { controller: XtermController; onClose: () => void } = $props();

  let query = $state("");
  let resultIndex = $state(-1);
  let resultCount = $state(0);
  let inputEl: HTMLInputElement | undefined = $state();

  // Colors are literal hex (not `--dockl-*` tokens) because `ISearchDecorationOptions`
  // takes real `#RRGGBB` strings baked into xterm's own decoration rendering, not CSS —
  // there's no way to point it at a custom property instead.
  const searchOptions = {
    decorations: {
      matchBackground: "#f9d949",
      matchBorder: "#f9d949",
      matchOverviewRuler: "#f9d949",
      activeMatchBackground: "#f78500",
      activeMatchBorder: "#f78500",
      activeMatchColorOverviewRuler: "#f78500",
    },
  };

  $effect(() => {
    const sub = controller.searchAddon.onDidChangeResults((e) => {
      resultIndex = e.resultIndex;
      resultCount = e.resultCount;
    });
    return () => sub.dispose();
  });

  $effect(() => {
    // xterm keeps an invisible textarea focused to capture typing for the terminal
    // itself — explicitly releasing that first, rather than just calling `.focus()`
    // on our own input and assuming it wins, avoids a race where keystrokes meant for
    // this search box go to the terminal instead (nothing typed would ever reach
    // `query`, which looks exactly like "search matches nothing").
    controller.terminal.blur();
    inputEl?.focus();
    inputEl?.select();
  });

  function findNext() {
    if (query) controller.searchAddon.findNext(query, searchOptions);
  }

  function findPrevious() {
    if (query) controller.searchAddon.findPrevious(query, searchOptions);
  }

  // Reads the input's value directly from the event rather than relying on `query`
  // (updated via `bind:value` on the same element) — with both an `oninput` handler and
  // `bind:value` on one element, which one observes the new value first isn't something
  // to depend on; reading `e.currentTarget.value` here is unambiguous.
  function handleInput(e: Event & { currentTarget: HTMLInputElement }) {
    query = e.currentTarget.value;
    if (!query) {
      controller.searchAddon.clearDecorations();
      resultIndex = -1;
      resultCount = 0;
      return;
    }
    findNext();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      if (e.shiftKey) findPrevious();
      else findNext();
    } else if (e.key === "Escape") {
      // Also stops this from bubbling to a parent modal's own Escape-to-close handler
      // (e.g. WslShellDialog) — Escape here should close the search bar, not the whole
      // dialog underneath it.
      e.preventDefault();
      e.stopPropagation();
      close();
    }
  }

  function close() {
    controller.searchAddon.clearDecorations();
    onClose();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="search-bar dockl-surface"
  onkeydown={handleKeydown}
  onclick={(e) => e.stopPropagation()}
>
  <input
    bind:this={inputEl}
    value={query}
    oninput={handleInput}
    placeholder={$t("terminal.search.placeholder")}
    class="search-input"
    aria-label={$t("terminal.search.ariaLabel")}
  />
  <span class="result-count">
    {#if query}
      {resultCount > 0 ? `${resultIndex + 1}/${resultCount}` : $t("terminal.search.noMatches")}
    {/if}
  </span>
  <button
    class="icon-btn"
    title={$t("terminal.search.previous")}
    aria-label={$t("terminal.search.previousAriaLabel")}
    disabled={!query}
    onclick={findPrevious}
  >
    <Icon svg={chevronUpIcon} size={14} />
  </button>
  <button
    class="icon-btn"
    title={$t("terminal.search.next")}
    aria-label={$t("terminal.search.nextAriaLabel")}
    disabled={!query}
    onclick={findNext}
  >
    <Icon svg={chevronDownIcon} size={14} />
  </button>
  <button
    class="icon-btn"
    title={$t("common.close")}
    aria-label={$t("terminal.search.closeAriaLabel")}
    onclick={close}
  >
    <Icon svg={dismissIcon} size={14} />
  </button>
</div>

<style>
  .search-bar {
    position: absolute;
    top: 8px;
    right: 8px;
    z-index: 10;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 6px;
    background: var(--dockl-menu-bg);
    border: 1px solid var(--dockl-border);
    border-radius: var(--dockl-radius);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
  }

  .search-input {
    width: 160px;
    padding: 4px 6px;
    font-size: 12px;
    font-family: inherit;
    color: var(--dockl-text-primary);
    background: var(--dockl-surface-hover);
    border: 1px solid var(--dockl-border);
    border-radius: 4px;
  }

  .search-input:focus-visible {
    outline: 2px solid var(--dockl-accent);
    outline-offset: -1px;
  }

  .result-count {
    min-width: 60px;
    font-size: 11px;
    color: var(--dockl-text-secondary);
    text-align: center;
    user-select: none;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    /* See the note in LogViewer.svelte: without this the UA's button padding eats the
       fixed width and the icon shrinks to fit what's left. */
    padding: 0;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--dockl-text-secondary);
    cursor: pointer;
  }

  .icon-btn:hover:not(:disabled) {
    background: var(--dockl-surface-hover);
    color: var(--dockl-text-primary);
  }

  .icon-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }
</style>
