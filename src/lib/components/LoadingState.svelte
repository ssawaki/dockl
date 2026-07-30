<script lang="ts">
  let { message = "読み込み中..." }: { message?: string } = $props();
</script>

<div class="loading-state">
  <fluent-spinner size="medium"></fluent-spinner>
  <p>{message}</p>
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
