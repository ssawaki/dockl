<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import MasterList from "$lib/components/layout/MasterList.svelte";
  import MasterListRow from "$lib/components/layout/MasterListRow.svelte";
  import { t } from "$lib/stores/i18n";
  import deleteIcon from "@fluentui/svg-icons/icons/delete_16_regular.svg?raw";
  import type { VolumeSummary } from "$lib/types";

  let {
    volumes,
    loading = false,
    selectedName = $bindable(),
    onRemove,
  }: {
    volumes: VolumeSummary[];
    loading?: boolean;
    selectedName: string | null;
    onRemove: (volume: VolumeSummary) => void;
  } = $props();
</script>

<MasterList
  {loading}
  empty={volumes.length === 0}
  loadingLabel={$t("volumes.loading")}
  emptyLabel={$t("volumes.empty")}
>
  {#each volumes as volume (volume.name)}
    <MasterListRow
      name={volume.name}
      selected={selectedName === volume.name}
      onSelect={() => (selectedName = volume.name)}
    >
      {#snippet meta()}
        <!-- The Compose project is the most useful thing to disambiguate by: volume names
             are often long generated hashes that differ only near the end. -->
        <span class="meta-text">{volume.compose_project ?? volume.driver}</span>
      {/snippet}
      {#snippet actions()}
        <button
          class="icon-btn"
          tabindex="-1"
          title={$t("action.remove")}
          aria-label={$t("action.remove")}
          onclick={(e) => {
            e.stopPropagation();
            onRemove(volume);
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

  .icon-btn:hover {
    background: var(--dockl-surface-hover);
    color: var(--dockl-danger);
  }
</style>
