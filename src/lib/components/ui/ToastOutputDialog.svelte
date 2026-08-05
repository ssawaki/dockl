<script lang="ts">
  import { trapFocus } from "$lib/actions/trapFocus";
  import CopyIconButton from "$lib/components/ui/CopyIconButton.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { t } from "$lib/stores/i18n";
  import dismissIcon from "@fluentui/svg-icons/icons/dismiss_20_regular.svg?raw";

  let { title, output, onClose }: { title: string; output: string; onClose: () => void } = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="dialog dockl-surface"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    use:trapFocus
    onclick={(e) => e.stopPropagation()}
  >
    <div class="dialog-header">
      <span class="dialog-title">{title}</span>
      <div class="dialog-actions">
        <CopyIconButton value={output} iconSize={15} />
        <button class="icon-btn" title={$t("common.close")} aria-label={$t("common.close")} onclick={onClose}>
          <Icon svg={dismissIcon} size={16} />
        </button>
      </div>
    </div>
    <div class="dialog-body">
      {#if output.trim()}
        <pre class="output">{output}</pre>
      {:else}
        <p class="empty">{$t("toastOutput.empty")}</p>
      {/if}
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 3000;
  }

  .dialog {
    background: var(--dockl-menu-bg);
    width: min(640px, 90vw);
    max-height: 70vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-bottom: 1px solid var(--dockl-border);
    flex-shrink: 0;
  }

  .dialog-title {
    font-size: 13px;
    font-weight: 600;
  }

  .dialog-actions {
    display: flex;
    gap: 2px;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    /* See the note in LogViewer.svelte: without this the UA's button padding eats the
       fixed width and the icon shrinks to fit what's left. */
    padding: 0;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--dockl-text-secondary);
    cursor: pointer;
  }

  .icon-btn:hover {
    background: var(--dockl-surface-hover);
    color: var(--dockl-text-primary);
  }

  .dialog-body {
    padding: 12px;
    overflow: auto;
  }

  .output {
    margin: 0;
    font-family: Consolas, "Cascadia Code", monospace;
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-word;
    user-select: text;
    cursor: text;
  }

  .empty {
    margin: 0;
    font-size: 13px;
    color: var(--dockl-text-secondary);
  }
</style>
