import { invoke } from "@tauri-apps/api/core";
import type { DiskUsageEntry } from "$lib/types";

export function getDiskUsage(): Promise<DiskUsageEntry[]> {
  return invoke("get_disk_usage");
}

/**
 * Removes build cache not associated with any image. `all: false` matches plain
 * `docker builder prune` (dangling cache only); `all: true` extends it to cache Docker
 * would otherwise keep around to speed up future builds (`-a`).
 */
export function pruneBuildCache(all: boolean): Promise<string> {
  return invoke("prune_build_cache", { all });
}
