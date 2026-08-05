<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import MasterList from "$lib/components/layout/MasterList.svelte";
  import MasterListRow from "$lib/components/layout/MasterListRow.svelte";
  import { t } from "$lib/stores/i18n";
  import deleteIcon from "@fluentui/svg-icons/icons/delete_16_regular.svg?raw";
  import { imageDisplayName } from "$lib/images";
  import type { ImageSummary } from "$lib/types";

  let {
    images,
    loading = false,
    selectedId = $bindable(),
    onRemove,
  }: {
    images: ImageSummary[];
    loading?: boolean;
    selectedId: string | null;
    onRemove: (image: ImageSummary) => void;
  } = $props();

  // Split the way `docker images` users think about it: what's holding disk space you
  // can't reclaim versus what a prune would take.
  let inUse = $derived(images.filter((i) => Number(i.containers) > 0));
  let unused = $derived(images.filter((i) => !(Number(i.containers) > 0)));

  // Keyed by `key`, not `label`: the labels are translated strings, so keying on them
  // would rebuild both sections on every language switch, and any locale that ever gave
  // the two sections the same wording would collide into a duplicate-key error.
  let sections = $derived([
    { key: "inUse", label: $t("images.section.inUse"), items: inUse },
    { key: "unused", label: $t("images.section.unused"), items: unused },
  ]);
</script>

<MasterList {loading} empty={images.length === 0} loadingLabel={$t("images.loading")} emptyLabel={$t("images.empty")}>
  {#each sections as section (section.key)}
    {#if section.items.length > 0}
      <div class="section-header">{section.label}</div>
      {#each section.items as image (image.id)}
        <MasterListRow
          name={imageDisplayName(image)}
          selected={selectedId === image.id}
          dim={image.tags.length === 0}
          onSelect={() => (selectedId = image.id)}
        >
          {#snippet meta()}
            <span class="meta-text">{image.size}, {image.created_since}</span>
            <!-- Surfaced here because the row only has room for one name: without it an
                 image with several tags would look like it has exactly one. -->
            {#if image.tags.length > 1}
              <span class="tag-count">{$t("images.moreTags", { count: String(image.tags.length - 1) })}</span>
            {/if}
          {/snippet}
          {#snippet actions()}
            <button
              class="icon-btn"
              tabindex="-1"
              title={$t("action.remove")}
              aria-label={$t("action.remove")}
              onclick={(e) => {
                e.stopPropagation();
                onRemove(image);
              }}
            >
              <Icon svg={deleteIcon} size={16} />
            </button>
          {/snippet}
        </MasterListRow>
      {/each}
    {/if}
  {/each}
</MasterList>

<style>
  .section-header {
    padding: 8px 8px 4px;
    font-size: 11px;
    font-weight: 600;
    color: var(--dockl-text-secondary);
  }

  /* Truncates on its own so a long relative date ("about 2 months ago") shortens rather
     than pushing the tag badge out of the row. */
  .meta-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tag-count {
    flex-shrink: 0;
    padding: 0 4px;
    border-radius: 8px;
    background: var(--dockl-surface-hover);
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
