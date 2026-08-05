<script lang="ts">
  import CopyableValue from "$lib/components/ui/CopyableValue.svelte";
  import DetailPanel from "$lib/components/layout/DetailPanel.svelte";
  import InfoTable from "$lib/components/ui/InfoTable.svelte";
  import { t } from "$lib/stores/i18n";
  import type { VolumeSummary } from "$lib/types";

  let { volume }: { volume: VolumeSummary | null } = $props();
</script>

<DetailPanel placeholder={$t("volumes.detail.placeholder")} empty={!volume}>
  {#if volume}
    <h2 class="detail-title">{volume.name}</h2>

    <InfoTable>
      <tr>
        <th>{$t("table.composeProject")}</th>
        <td>{volume.compose_project ?? "—"}</td>
      </tr>
      <tr>
        <th>{$t("table.driver")}</th>
        <td>{volume.driver}</td>
      </tr>
      <tr>
        <th>{$t("table.scope")}</th>
        <td>{volume.scope}</td>
      </tr>
      <tr>
        <!-- Copyable because this is the path you'd `cd` into from a WSL shell — the one
             thing about a volume that's useful outside the app. -->
        <th>{$t("volumes.table.mountpoint")}</th>
        <td><CopyableValue value={volume.mountpoint}>{volume.mountpoint}</CopyableValue></td>
      </tr>
    </InfoTable>
  {/if}
</DetailPanel>
