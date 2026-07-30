import { invoke } from "@tauri-apps/api/core";
import type { DistroInfo } from "$lib/types";

export function setupListDistros(): Promise<DistroInfo[]> {
  return invoke("setup_list_distros");
}

export function setupConnect(distro: string): Promise<void> {
  return invoke("setup_connect", { distro });
}

export function setupCurrentDistro(): Promise<string | null> {
  return invoke("setup_current_distro");
}
