import { invoke } from "@tauri-apps/api/core";
import type { VolumeSummary } from "$lib/types";

export function listVolumes(): Promise<VolumeSummary[]> {
  return invoke("list_volumes");
}

export function removeVolume(name: string): Promise<void> {
  return invoke("remove_volume", { name });
}

/**
 * Removes unused volumes. `all: false` limits this to anonymous volumes; `all: true`
 * also removes unused named volumes. Returns docker's own summary text.
 */
export function pruneVolumes(all: boolean): Promise<string> {
  return invoke("prune_volumes", { all });
}
