<script lang="ts">
  import CopyableValue from "$lib/components/ui/CopyableValue.svelte";
  import DetailPanel from "$lib/components/layout/DetailPanel.svelte";
  import InfoTable from "$lib/components/ui/InfoTable.svelte";
  import { t } from "$lib/stores/i18n";
  import { imageDisplayName } from "$lib/images";
  import type { ImageSummary } from "$lib/types";

  let { image }: { image: ImageSummary | null } = $props();
</script>

<DetailPanel placeholder={$t("images.detail.placeholder")} empty={!image}>
  {#if image}
    <h2 class="detail-title">{imageDisplayName(image)}</h2>

    <InfoTable>
      <tr>
        <th>{$t("images.detail.id")}</th>
        <td><CopyableValue value={image.id}>{image.id}</CopyableValue></td>
      </tr>
      <tr>
        <th>{$t("table.size")}</th>
        <td>{image.size}</td>
      </tr>
      <tr>
        <th>{$t("images.table.created")}</th>
        <td>{image.created_since}</td>
      </tr>
      <tr>
        <th>{$t("images.table.containersInUse")}</th>
        <td>{image.containers}</td>
      </tr>
    </InfoTable>

    <!-- The reason this panel exists: the list can only show one name, and an image
         frequently answers to several. -->
    <h3 class="detail-subhead">{$t("images.detail.tags")}</h3>
    {#if image.tags.length === 0}
      <p class="none">{$t("images.detail.noTags")}</p>
    {:else}
      <ul class="tags">
        {#each image.tags as tag (tag)}
          <li><CopyableValue value={tag}>{tag}</CopyableValue></li>
        {/each}
      </ul>
    {/if}
  {/if}
</DetailPanel>

<style>
  .tags {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 12px;
  }

  .tags li {
    word-break: break-all;
  }

  .none {
    margin: 0;
    font-size: 12px;
    color: var(--dockl-text-secondary);
  }
</style>
