import { invoke } from "@tauri-apps/api/core";
import type { ContainerActionKind, ContainerDetail, ContainerSummary } from "$lib/types";

export function listContainers(all = true): Promise<ContainerSummary[]> {
  return invoke("list_containers", { all });
}

export function containerAction(id: string, action: ContainerActionKind): Promise<void> {
  return invoke("container_action", { id, action });
}

export function inspectContainer(id: string): Promise<ContainerDetail> {
  return invoke("inspect_container", { id });
}
