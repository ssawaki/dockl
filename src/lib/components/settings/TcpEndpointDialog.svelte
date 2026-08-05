<script lang="ts">
  import { formatError } from "$lib/errors";
  import { trapFocus } from "$lib/actions/trapFocus";
  import { checkTcpBridge } from "$lib/ipc/setup";
  import { TCP_BRIDGE_PORT } from "$lib/tcpBridge";
  import Icon from "$lib/components/ui/Icon.svelte";
  import CopyableValue from "$lib/components/ui/CopyableValue.svelte";
  import { t } from "$lib/stores/i18n";
  import dismissIcon from "@fluentui/svg-icons/icons/dismiss_20_regular.svg?raw";
  import warningIcon from "@fluentui/svg-icons/icons/warning_20_filled.svg?raw";

  let { onClose }: { onClose: () => void } = $props();

  // Dockl deliberately has no button that opens this port any more. An earlier version
  // did — it wrote a systemd override switching on `dockerd -H tcp://...` — but an
  // unauthenticated Engine API reachable by every process on the machine is precisely
  // what PLAN.md's security section rules out, and the relay-process mode reaches the
  // same API at the same measured speed without opening anything. What's left here is
  // the reverse operation, for anyone who ran that setup before it was removed.
  //
  // One single-quoted `sudo sh -c '...'` for the same reason the old script was: pasting
  // several lines into an interactive shell where an early one triggers a password
  // prompt lets the queued remainder be consumed as keystrokes for that prompt. As one
  // quoted argument, the shell keeps reading continuation lines without executing
  // anything until the closing quote, so nothing is left queued behind the prompt.
  const TCP_TEARDOWN_SCRIPT = `sudo sh -c '
  rm -f /etc/systemd/system/docker.service.d/dockl-tcp.conf &&
  systemctl daemon-reload &&
  systemctl restart docker
'`;

  let checkStatus = $state<"idle" | "checking" | "ok" | "error">("idle");
  let checkMessage = $state<string | null>(null);

  async function checkConnection() {
    checkStatus = "checking";
    checkMessage = null;
    try {
      await checkTcpBridge(TCP_BRIDGE_PORT);
      checkStatus = "ok";
    } catch (e) {
      checkStatus = "error";
      checkMessage = formatError(e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
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
    tabindex="-1"
    use:trapFocus
    onclick={(e) => e.stopPropagation()}
  >
    <div class="dialog-header">
      <span class="dialog-title">{$t("tcpEndpoint.title")}</span>
      <button
        class="icon-btn"
        title={$t("common.close")}
        aria-label={$t("common.close")}
        onclick={onClose}
      >
        <Icon svg={dismissIcon} size={16} />
      </button>
    </div>

    <div class="dialog-body">
      <p class="section-desc">
        {$t("tcpEndpoint.description1a")}<code>127.0.0.1:{TCP_BRIDGE_PORT}</code>{$t(
          "tcpEndpoint.description1b",
        )}
      </p>

      <p class="security-warning">
        <Icon svg={warningIcon} size={16} />
        <span>{$t("tcpEndpoint.warning")}</span>
      </p>

      <div class="row">
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <fluent-button
          appearance="outline"
          disabled={checkStatus === "checking"}
          onclick={checkConnection}
        >
          <span class="btn-content">
            {#if checkStatus === "checking"}
              <fluent-spinner size="tiny"></fluent-spinner>
            {/if}
            {$t("tcpEndpoint.checkConnection")}
          </span>
        </fluent-button>
        {#if checkStatus === "ok"}
          <span class="status-ok">{$t("tcpEndpoint.connectionOk")}</span>
        {:else if checkStatus === "error"}
          <span class="status-error">
            <CopyableValue
              value={$t("tcpEndpoint.connectionFailed", { error: checkMessage ?? "" })}
            >
              {$t("tcpEndpoint.connectionFailed", { error: checkMessage ?? "" })}
            </CopyableValue>
          </span>
        {/if}
      </div>

      <fluent-divider></fluent-divider>

      <p class="section-desc">{$t("tcpEndpoint.teardown")}</p>
      <CopyableValue value={TCP_TEARDOWN_SCRIPT}>
        <code class="command-preview">{TCP_TEARDOWN_SCRIPT}</code>
      </CopyableValue>
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
    width: min(620px, 90vw);
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--dockl-border);
    flex-shrink: 0;
  }

  .dialog-title {
    font-size: 14px;
    font-weight: 600;
  }

  .dialog-body {
    padding: 16px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
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

  .section-desc {
    font-size: 12px;
    color: var(--dockl-text-secondary);
    margin: 0;
  }

  /* `color-mix` against the surface rather than a fixed tint, so the same rule reads
     correctly on Mica/Acrylic and in both themes. */
  .security-warning {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin: 0;
    padding: 10px 12px;
    font-size: 12px;
    line-height: 1.5;
    color: var(--dockl-text-primary);
    background: color-mix(in srgb, var(--dockl-warning) 12%, transparent);
    border-left: 3px solid var(--dockl-warning);
    border-radius: 4px;
  }

  .security-warning :global(svg) {
    flex-shrink: 0;
    margin-top: 1px;
    color: var(--dockl-warning);
  }

  .command-preview {
    display: block;
    margin-top: 4px;
    padding: 6px 8px;
    font-size: 12px;
    font-family: Consolas, "Cascadia Code", monospace;
    color: var(--dockl-text-primary);
    background: var(--dockl-surface-hover);
    border-radius: 4px;
    white-space: pre-wrap;
    word-break: break-all;
    user-select: text;
    cursor: text;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .btn-content {
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }

  .status-ok {
    font-size: 12px;
    color: var(--dockl-success);
  }

  .status-error {
    font-size: 12px;
    color: var(--dockl-danger);
  }
</style>
