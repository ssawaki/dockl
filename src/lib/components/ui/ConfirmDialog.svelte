<script lang="ts">
  import type { Snippet } from "svelte";
  import { trapFocus } from "$lib/actions/trapFocus";
  import { t } from "$lib/stores/i18n";

  let {
    title,
    message,
    confirmLabel,
    extra,
    onConfirm,
    onCancel,
  }: {
    title: string;
    message: string;
    confirmLabel?: string;
    /** Optional extra content (e.g. an options checkbox) between the warning and the buttons. */
    extra?: Snippet;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();

  let displayConfirmLabel = $derived(confirmLabel ?? $t("action.remove"));

  let confirmBtn: HTMLElement | undefined = $state();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onCancel();
  }

  $effect(() => {
    // Focus the confirm button once Fluent's custom element has upgraded, so Enter
    // right after the dialog opens confirms rather than doing nothing.
    queueMicrotask(() => confirmBtn?.focus());
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onCancel}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="dialog dockl-surface"
    role="alertdialog"
    aria-modal="true"
    tabindex="-1"
    use:trapFocus
    onclick={(e) => e.stopPropagation()}
  >
    <h2>{title}</h2>
    <p>{message}</p>
    <p class="warning">{$t("common.confirmIrreversible")}</p>
    {#if extra}
      <div class="extra">
        {@render extra()}
      </div>
    {/if}
    <div class="actions">
      <fluent-button appearance="outline" onclick={onCancel}>{$t("common.cancel")}</fluent-button>
      <fluent-button bind:this={confirmBtn} appearance="primary" class="danger" onclick={onConfirm}>
        {displayConfirmLabel}
      </fluent-button>
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
    /* Solid background for the same reason as ContextMenu — this floats over
       arbitrary content and needs to stay legible. */
    background: var(--dockl-menu-bg);
    padding: 20px;
    max-width: 380px;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
  }

  h2 {
    font-size: 15px;
    font-weight: 600;
    margin: 0 0 8px;
  }

  p {
    font-size: 13px;
    color: var(--dockl-text-secondary);
    margin: 0 0 8px;
  }

  .warning {
    font-weight: 700;
    margin: 0 0 18px;
  }

  .extra {
    margin: 0 0 18px;
    font-size: 13px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .actions :global(.danger) {
    --colorBrandBackground: var(--dockl-danger);
    --colorBrandBackgroundHover: var(--dockl-danger);
    --colorBrandBackgroundPressed: var(--dockl-danger);
  }
</style>
