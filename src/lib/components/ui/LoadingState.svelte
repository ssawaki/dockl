<script lang="ts">
  import { t } from "$lib/stores/i18n";

  let { message }: { message?: string } = $props();
  let displayMessage = $derived(message ?? $t("common.loading"));
</script>

<div class="loading-state">
  <fluent-spinner size="medium"></fluent-spinner>
  <p>{displayMessage}</p>
</div>

<style>
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    padding: 48px;
    flex: 1;
    color: var(--dockl-text-secondary);
    font-size: 13px;
  }
</style>
