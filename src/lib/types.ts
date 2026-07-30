export interface ContainerSummary {
  id: string;
  names: string[];
  image: string;
  state: string;
  status: string;
  ports: string;
  labels: Record<string, string>;
}

export type ContainerActionKind =
  | "start"
  | "stop"
  | "restart"
  | "remove"
  | "pause"
  | "unpause";

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
}
