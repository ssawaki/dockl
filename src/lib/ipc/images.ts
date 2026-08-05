import { invoke } from "@tauri-apps/api/core";
import type { ImageSummary } from "$lib/types";

export function listImages(): Promise<ImageSummary[]> {
  return invoke("list_images");
}

export function removeImage(id: string): Promise<void> {
  return invoke("remove_image", { id });
}

/**
 * Removes unused images. `all: false` limits this to dangling (untagged) images;
 * `all: true` also removes unused-but-tagged ones. Returns docker's own summary text.
 */
export function pruneImages(all: boolean): Promise<string> {
  return invoke("prune_images", { all });
}
