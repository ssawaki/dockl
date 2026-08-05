export interface ContainerSummary {
  id: string;
  names: string[];
  image: string;
  state: string;
  status: string;
  ports: string;
  labels: Record<string, string>;
}

export type ContainerActionKind = "start" | "stop" | "restart" | "remove" | "pause" | "unpause";

export interface ImageSummary {
  id: string;
  /**
   * Every `repository:tag` name pointing at this image, sorted, and empty when it's
   * dangling. One image can answer to several names at once — see `imageDisplayName`
   * for the one the list shows.
   */
  tags: string[];
  size: string;
  created_since: string;
  containers: string;
}

export interface VolumeSummary {
  name: string;
  driver: string;
  mountpoint: string;
  scope: string;
  compose_project: string | null;
}

export interface NetworkSummary {
  id: string;
  /** "bridge"/"host"/"none" are Docker's built-ins — present everywhere, can't be removed. */
  name: string;
  driver: string;
  scope: string;
  internal: boolean;
  compose_project: string | null;
}

/**
 * Per-resource-type disk usage summary, matching `docker system df`'s own rows
 * ("Images" / "Containers" / "Local Volumes" / "Build Cache"). Every field is already
 * formatted as Docker's own human-readable string (e.g. "1.2GB", "11.45GB (72%)").
 */
export interface DiskUsageEntry {
  kind: string;
  total_count: string;
  active: string;
  size: string;
  reclaimable: string;
}

export interface DistroInfo {
  name: string;
  is_default: boolean;
  is_running: boolean;
  wsl_version: number;
}

export interface PortForward {
  host_ip: string;
  host_port: string;
  container_port: string;
  protocol: string;
}

export interface MountInfo {
  mount_type: string;
  source: string;
  destination: string;
}

/**
 * Shared between `ContainerDetailPanel` and `ComposeDetailPanel` — lifted to the parent
 * route so the selected tab survives switching between a container and a Compose
 * project (rather than each panel resetting to "info" on mount, since only one of the
 * two is ever in the DOM at a time).
 */
export type DetailTabId = "info" | "stats" | "logs" | "terminal";

export interface ContainerDetail {
  id: string;
  name: string;
  image: string;
  status: string;
  health: string | null;
  created: string;
  ip_address: string | null;
  ports: PortForward[];
  mounts: MountInfo[];
  labels: Record<string, string>;
  cpu_limit_cores: number | null;
  /** `--restart` policy as the Docker CLI spells it: "no" / "always" / "unless-stopped" / "on-failure" / "on-failure:3". */
  restart_policy: string;
}
