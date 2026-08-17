<script lang="ts">
  import { trapFocus } from "$lib/actions/trapFocus";
  import CopyIconButton from "$lib/components/ui/CopyIconButton.svelte";
  import { appIcon, appName } from "$lib/branding";
  import { buildInfo } from "$lib/buildInfo";
  import { t } from "$lib/stores/i18n";

  let { onClose }: { onClose: () => void } = $props();

  // Awaited in the markup rather than assigned to state from an effect — see a15cdbb,
  // which went through the app removing exactly that shape.
  const build = buildInfo();

  let closeBtn: HTMLElement | undefined = $state();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }

  $effect(() => {
    // Same as ConfirmDialog: wait for Fluent's custom element to upgrade, so Enter right
    // after opening closes the dialog rather than doing nothing.
    queueMicrotask(() => closeBtn?.focus());
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="dialog dockl-surface"
    role="dialog"
    aria-modal="true"
    aria-label={$t("about.title", { name: appName })}
    tabindex="-1"
    use:trapFocus
    onclick={(e) => e.stopPropagation()}
  >
    <img class="icon" src={appIcon} alt="" width="48" height="48" />
    <h2>{appName}</h2>
    <!-- CopyIconButton on its own rather than wrapped in CopyableValue: that wrapper hides
         the button until hover, which is right for a table of them but here left the text
         sitting off-centre by half a button until the pointer arrived. -->
    <div class="version">
      {#await build then b}
        <span>{$t("about.version", { version: b.label })}</span>
        <CopyIconButton value={b.copyValue} />
      {:catch}
        <!-- Deliberately empty: a failed read shouldn't take the dialog with it. -->
      {/await}
    </div>
    <div class="actions">
      <fluent-button bind:this={closeBtn} appearance="primary" onclick={onClose}>
        {$t("common.close")}
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
    /* Solid background for the same reason as ContextMenu and ConfirmDialog — this floats
       over arbitrary content and needs to stay legible. */
    background: var(--dockl-menu-bg);
    min-width: 260px;
    padding: 24px 28px;
    text-align: center;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
  }

  .icon {
    display: block;
    margin: 0 auto 10px;
  }

  h2 {
    font-size: 15px;
    font-weight: 600;
    margin: 0 0 4px;
  }

  .version {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    font-size: 13px;
    color: var(--dockl-text-secondary);
    /* Holds the row's height while getVersion() is in flight, so the dialog doesn't
       resize under the pointer a moment after it opens. */
    min-height: 26px;
    margin: 0 0 18px;
  }

  .actions {
    display: flex;
    justify-content: center;
  }
</style>
