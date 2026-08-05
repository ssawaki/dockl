import { invoke } from "@tauri-apps/api/core";
import type { NetworkSummary } from "$lib/types";

export function listNetworks(): Promise<NetworkSummary[]> {
  return invoke("list_networks");
}

export function removeNetwork(id: string): Promise<void> {
  return invoke("remove_network", { id });
}

/** Removes networks not used by any container. Returns docker's own summary text. */
export function pruneNetworks(): Promise<string> {
  return invoke("prune_networks");
}
