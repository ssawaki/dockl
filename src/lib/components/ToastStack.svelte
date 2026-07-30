<script lang="ts">
  import { toasts, dismissToast } from "$lib/stores/toasts";
  import Icon from "$lib/components/Icon.svelte";
  import checkmarkIcon from "@fluentui/svg-icons/icons/checkmark_circle_20_filled.svg?raw";
  import errorIcon from "@fluentui/svg-icons/icons/error_circle_20_filled.svg?raw";
  import dismissIcon from "@fluentui/svg-icons/icons/dismiss_20_regular.svg?raw";
</script>

<div class="toast-stack">
  {#each $toasts as t (t.id)}
    <div class="toast dockl-surface toast-{t.status}">
      <span class="toast-icon">
        {#if t.status === "loading"}
          <fluent-spinner size="tiny"></fluent-spinner>
        {:else if t.status === "success"}
          <Icon svg={checkmarkIcon} size={18} />
        {:else}
          <Icon svg={errorIcon} size={18} />
        {/if}
      </span>
      <span class="toast-message">{t.message}</span>
      {#if t.status !== "loading"}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <button class="toast-close" onclick={() => dismissToast(t.id)} aria-label="閉じる">
          <Icon svg={dismissIcon} size={13} />
        </button>
      {/if}
    </div>
  {/each}
</div>

<style>
  .toast-stack {
    position: fixed;
    bottom: 16px;
    right: 16px;
    display: flex;
    flex-direction: column-reverse;
    gap: 8px;
    z-index: 1000;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    min-width: 220px;
    max-width: 340px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
    pointer-events: auto;
    font-size: 13px;
    animation: toast-in 0.15s ease-out;
  }

  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .toast-icon {
    display: flex;
    flex-shrink: 0;
  }

  .toast-message {
    flex: 1;
    word-break: break-word;
  }

  .toast-success {
    border-color: var(--dockl-success);
  }

  .toast-success .toast-icon {
    color: var(--dockl-success);
  }

  .toast-error {
    border-color: var(--dockl-danger);
  }

  .toast-error .toast-icon {
    color: var(--dockl-danger);
  }

  .toast-close {
    border: none;
    background: transparent;
    color: var(--dockl-text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .toast-close:hover {
    background: var(--dockl-surface-hover);
  }
</style>
