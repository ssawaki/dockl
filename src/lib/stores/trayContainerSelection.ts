import { writable } from "svelte/store";

/**
 * The container id from the tray's right-click menu (see `tray::build_tray` on the Rust
 * side). `+layout.svelte` is what actually receives the `tray:select-container` event
 * (it's mounted regardless of which route is showing) and navigates to `/`, but the
 * containers page owns `selectedId` as local state — this store is how the click's
 * target crosses that gap. The containers page clears it back to `null` once consumed.
 */
export const pendingTraySelection = writable<string | null>(null);
