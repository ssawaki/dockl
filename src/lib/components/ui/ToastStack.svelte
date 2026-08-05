<script lang="ts">
  import { flip } from "svelte/animate";
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { toasts, dismissToast, pauseToastTimer, resumeToastTimer } from "$lib/stores/toasts";
  import { t } from "$lib/stores/i18n";
  import CopyIconButton from "$lib/components/ui/CopyIconButton.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import ToastOutputDialog from "$lib/components/ui/ToastOutputDialog.svelte";
  import checkmarkIcon from "@fluentui/svg-icons/icons/checkmark_circle_20_filled.svg?raw";
  import errorIcon from "@fluentui/svg-icons/icons/error_circle_20_filled.svg?raw";
  import dismissIcon from "@fluentui/svg-icons/icons/dismiss_20_regular.svg?raw";

  let openOutputToastId = $state<string | null>(null);
  let openOutputToast = $derived($toasts.find((toast) => toast.id === openOutputToastId) ?? null);
</script>

<div class="toast-stack" role="status" aria-live="polite">
  {#each $toasts as toast (toast.id)}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="toast dockl-surface toast-{toast.status}"
      animate:flip={{ duration: 200, easing: cubicOut }}
      in:fly={{ y: 14, duration: 220, easing: cubicOut }}
      out:fly={{ x: 48, duration: 180, easing: cubicOut }}
      onmouseenter={() => pauseToastTimer(toast.id)}
      onmouseleave={() => resumeToastTimer(toast.id)}
    >
      <div class="toast-main">
        <span class="toast-icon">
          {#if toast.status === "loading"}
            <fluent-spinner size="tiny"></fluent-spinner>
          {:else if toast.status === "success"}
            <Icon svg={checkmarkIcon} size={18} />
          {:else}
            <Icon svg={errorIcon} size={18} />
          {/if}
        </span>
        <span class="toast-message">{toast.message}</span>
        {#if toast.status === "error"}
          <CopyIconButton
            value={toast.message}
            label="toastStack.copyError"
            iconSize={18}
            size={28}
          />
        {/if}
        {#if toast.status !== "loading"}
          <button
            class="toast-btn"
            onclick={() => dismissToast(toast.id)}
            aria-label={$t("common.close")}
            title={$t("common.close")}
          >
            <Icon svg={dismissIcon} size={18} />
          </button>
        {/if}
      </div>
      {#if toast.output !== undefined}
        <div class="toast-footer">
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <fluent-button
            appearance="subtle"
            size="small"
            onclick={() => (openOutputToastId = toast.id)}
          >
            {$t("toastStack.showDetails")}
          </fluent-button>
        </div>
      {/if}
      {#if toast.duration}
        <span class="toast-progress" style={`animation-duration: ${toast.duration}ms`}></span>
      {/if}
    </div>
  {/each}
</div>

{#if openOutputToast}
  <ToastOutputDialog
    title={openOutputToast.message}
    output={openOutputToast.output ?? ""}
    onClose={() => (openOutputToastId = null)}
  />
{/if}

<style>
  .toast-stack {
    position: fixed;
    bottom: 16px;
    right: 16px;
    display: flex;
    flex-direction: column-reverse;
    gap: 10px;
    z-index: 1000;
    pointer-events: none;
  }

  .toast {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 14px 14px;
    min-width: 240px;
    max-width: 360px;
    border-radius: 10px;
    overflow: hidden;
    /* Toasts float over arbitrary page content, so — like ContextMenu/ConfirmDialog's
       dialogs — they use the opaque menu background instead of the translucent
       `.dockl-surface` one, or content behind them would show through and hurt legibility. */
    background: var(--dockl-menu-bg);
    /* Blur reaches past the stack's own `gap`, but low opacity keeps it a soft ambient
       glow rather than a hard edge that visibly overlaps the toast below. */
    box-shadow:
      0 3px 14px rgba(0, 0, 0, 0.11),
      0 1px 2px rgba(0, 0, 0, 0.08);
    pointer-events: auto;
    font-size: 13px;
  }

  .toast-main {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .toast-footer {
    display: flex;
    justify-content: flex-end;
  }

  .toast-icon {
    display: flex;
    flex-shrink: 0;
  }

  .toast-message {
    flex: 1;
    word-break: break-word;
  }

  .toast-success .toast-icon {
    color: var(--dockl-success);
  }

  .toast-error .toast-icon {
    color: var(--dockl-danger);
  }

  .toast-btn {
    border: none;
    /* See the note in LogViewer.svelte's `.icon-btn`: without this the UA's button
       padding eats the fixed width and the icon shrinks to fit what's left. */
    padding: 0;
    background: transparent;
    color: var(--dockl-text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .toast-btn:hover {
    background: var(--dockl-surface-hover);
  }

  .toast-btn:focus-visible {
    outline: 2px solid var(--dockl-accent);
    outline-offset: 1px;
  }

  /* Countdown to auto-dismiss. Deliberately understated — a hairline that drains
     rather than a loud, brand-colored progress bar — so it reads as ambient timing
     info, not another status signal competing with the icon/stripe. */
  .toast-progress {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 2px;
    background: var(--dockl-text-secondary);
    opacity: 0.3;
    transform-origin: left;
    animation-name: toast-progress-shrink;
    animation-timing-function: linear;
    animation-fill-mode: forwards;
  }

  /* Mirrors the JS timer pause in stores/toasts.ts — freezes visually in sync with the
     actual dismiss timer being paused, instead of the bar continuing to drain. */
  .toast:hover .toast-progress {
    animation-play-state: paused;
  }

  @keyframes toast-progress-shrink {
    from {
      transform: scaleX(1);
    }
    to {
      transform: scaleX(0);
    }
  }
</style>
