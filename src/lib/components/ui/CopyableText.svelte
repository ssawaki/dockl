<script lang="ts">
  import CopyIconButton from "$lib/components/ui/CopyIconButton.svelte";

  let { value }: { value: string } = $props();
</script>

<div class="copyable-text">
  <span class="copyable-text-content">{value}</span>
  <CopyIconButton {value} iconSize={15} />
</div>

<style>
  .copyable-text {
    display: flex;
    align-items: flex-start;
    gap: 6px;
  }

  .copyable-text-content {
    flex: 1;
    white-space: pre-wrap;
    user-select: text;
    cursor: text;
  }
</style>
