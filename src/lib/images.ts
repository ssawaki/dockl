import type { ImageSummary } from "$lib/types";

/**
 * The single name shown wherever there's only room for one — the list row, a toast, a
 * confirmation dialog.
 *
 * An image can answer to several `repository:tag` names at once (`docker tag` adds an
 * alias without copying anything), and the backend sorts them so this pick stays the same
 * between refreshes. An image with no name at all is rendered the way the Docker CLI
 * renders it, as `<none>`.
 */
export function imageDisplayName(image: ImageSummary): string {
  return image.tags[0] ?? "<none>";
}
