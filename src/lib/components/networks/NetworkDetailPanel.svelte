<script lang="ts">
  import CopyableValue from "$lib/components/ui/CopyableValue.svelte";
  import DetailPanel from "$lib/components/layout/DetailPanel.svelte";
  import InfoTable from "$lib/components/ui/InfoTable.svelte";
  import { t } from "$lib/stores/i18n";
  import { isBuiltinNetwork } from "$lib/networks";
  import type { NetworkSummary } from "$lib/types";

  let { network }: { network: NetworkSummary | null } = $props();
</script>

<DetailPanel placeholder={$t("networks.detail.placeholder")} empty={!network}>
  {#if network}
    <h2 class="detail-title">{network.name}</h2>

    <InfoTable>
      <tr>
        <th>{$t("networks.detail.id")}</th>
        <td><CopyableValue value={network.id}>{network.id}</CopyableValue></td>
      </tr>
      <tr>
        <th>{$t("table.composeProject")}</th>
        <td>{network.compose_project ?? "—"}</td>
      </tr>
      <tr>
        <th>{$t("table.driver")}</th>
        <td>{network.driver}</td>
      </tr>
      <tr>
        <th>{$t("table.scope")}</th>
        <td>{network.scope}</td>
      </tr>
      <tr>
        <th>{$t("networks.detail.internal")}</th>
        <td>{network.internal ? $t("common.yes") : $t("common.no")}</td>
      </tr>
    </InfoTable>

    <!-- Explains why the row's remove button is disabled, which the list has no room to say. -->
    {#if isBuiltinNetwork(network)}
      <p class="note">{$t("networks.builtinCannotRemove")}</p>
    {/if}
  {/if}
</DetailPanel>

<style>
  .note {
    margin: 0;
    font-size: 12px;
    color: var(--dockl-text-secondary);
  }
</style>
