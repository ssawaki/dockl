import { invoke } from "@tauri-apps/api/core";

/** One `docker stats --no-stream` JSON snapshot for the given container, as a raw string. */
export function getContainerStats(id: string): Promise<string> {
  return invoke("get_container_stats", { id });
}

/** How many CPUs the Docker daemon itself sees — the fallback ceiling for containers with no `--cpus` limit. */
export function getHostCpuCount(): Promise<number> {
  return invoke("get_host_cpu_count");
}

/** The container's disk usage as Docker formats it, e.g. `"16.4kB (virtual 146MB)"`. */
export function getContainerDiskUsage(id: string): Promise<string> {
  return invoke("get_container_disk_usage", { id });
}
