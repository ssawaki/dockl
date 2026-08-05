<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import MasterList from "$lib/components/layout/MasterList.svelte";
  import MasterListRow from "$lib/components/layout/MasterListRow.svelte";
  import { t } from "$lib/stores/i18n";
  import deleteIcon from "@fluentui/svg-icons/icons/delete_16_regular.svg?raw";
  import { isBuiltinNetwork } from "$lib/networks";
  import type { NetworkSummary } from "$lib/types";

  let {
    networks,
    loading = false,
    selectedId = $bindable(),
    onRemove,
  }: {
    networks: NetworkSummary[];
    loading?: boolean;
    selectedId: string | null;
    onRemove: (network: NetworkSummary) => void;
  } = $props();
</script>

<MasterList
  {loading}
  empty={networks.length === 0}
  loadingLabel={$t("networks.loading")}
  emptyLabel={$t("networks.empty")}
>
  {#each networks as network (network.id)}
    {@const builtin = isBuiltinNetwork(network)}
    <MasterListRow
      name={network.name}
      selected={selectedId === network.id}
      dim={builtin}
      onSelect={() => (selectedId = network.id)}
    >
      {#snippet meta()}
        <span class="meta-text">{network.compose_project ?? network.driver}</span>
      {/snippet}
      {#snippet actions()}
        <button
          class="icon-btn"
          tabindex="-1"
          title={builtin ? $t("networks.builtinCannotRemove") : $t("action.remove")}
          aria-label={$t("action.remove")}
          disabled={builtin}
          onclick={(e) => {
            e.stopPropagation();
            onRemove(network);
          }}
        >
          <Icon svg={deleteIcon} size={16} />
        </button>
      {/snippet}
    </MasterListRow>
  {/each}
</MasterList>

<style>
  .meta-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    /* See LogViewer.svelte: without this the UA's button padding eats the fixed width
       and the icon shrinks to fit what's left. */
    padding: 0;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--dockl-text-secondary);
    cursor: pointer;
  }

  .icon-btn:hover:not(:disabled) {
    background: var(--dockl-surface-hover);
    color: var(--dockl-danger);
  }

  .icon-btn:disabled {
    opacity: 0.35;
    cursor: default;
  }
</style>
