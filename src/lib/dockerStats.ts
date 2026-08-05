export interface ContainerStatsPoint {
  cpuPercent: number;
  memUsedBytes: number;
  memLimitBytes: number;
  memPercent: number;
  netRxBytes: number;
  netTxBytes: number;
  blockReadBytes: number;
  blockWriteBytes: number;
  pids: number;
}

/** Raw shape of one `docker stats --format '{{json .}}'` line. */
interface RawDockerStats {
  CPUPerc: string;
  MemUsage: string;
  MemPerc: string;
  NetIO: string;
  BlockIO: string;
  PIDs: string;
}

// Docker renders MemUsage with go-units' binary sizes (KiB/MiB/GiB) but NetIO/BlockIO
// with its decimal sizes (kB/MB/GB) - both tables are needed since the two fields use
// different bases for what looks like the same kind of value.
const BINARY_UNITS: Record<string, number> = {
  B: 1,
  KiB: 1024,
  MiB: 1024 ** 2,
  GiB: 1024 ** 3,
  TiB: 1024 ** 4,
};
const DECIMAL_UNITS: Record<string, number> = {
  B: 1,
  kB: 1000,
  KB: 1000,
  MB: 1000 ** 2,
  GB: 1000 ** 3,
  TB: 1000 ** 4,
};

/** Parses a Docker-formatted size string (e.g. `"15.76GB"`, `"208.9kB"`) into bytes. */
export function parseSize(raw: string | undefined): number {
  if (!raw) return 0;
  const match = raw.trim().match(/^([\d.]+)\s*([a-zA-Z]*)$/);
  if (!match) return 0;
  const value = parseFloat(match[1]);
  if (!Number.isFinite(value)) return 0;
  const unit = match[2];
  const multiplier = BINARY_UNITS[unit] ?? DECIMAL_UNITS[unit] ?? 1;
  return value * multiplier;
}

function parsePercent(raw: string): number {
  const value = parseFloat(raw.replace("%", ""));
  return Number.isFinite(value) ? value : 0;
}

/** Parses one `docker stats --format '{{json .}}'` line, or null if it isn't valid JSON. */
export function parseStatsLine(line: string): ContainerStatsPoint | null {
  let raw: RawDockerStats;
  try {
    raw = JSON.parse(line);
  } catch {
    return null;
  }

  const [memUsedRaw, memLimitRaw] = raw.MemUsage.split(" / ");
  const [netRxRaw, netTxRaw] = raw.NetIO.split(" / ");
  const [blockReadRaw, blockWriteRaw] = raw.BlockIO.split(" / ");

  return {
    cpuPercent: parsePercent(raw.CPUPerc),
    memUsedBytes: parseSize(memUsedRaw),
    memLimitBytes: parseSize(memLimitRaw),
    memPercent: parsePercent(raw.MemPerc),
    netRxBytes: parseSize(netRxRaw),
    netTxBytes: parseSize(netTxRaw),
    blockReadBytes: parseSize(blockReadRaw),
    blockWriteBytes: parseSize(blockWriteRaw),
    pids: parseInt(raw.PIDs, 10) || 0,
  };
}

export interface DiskUsage {
  /** The container's own writable layer, e.g. `"16.4kB"`. */
  writableSize: string;
  /** Writable layer + the (possibly shared) image, e.g. `"146MB"`; null if Docker didn't report one. */
  virtualSize: string | null;
}

/** Parses `docker ps -s`'s `Size` field, e.g. `"16.4kB (virtual 146MB)"`. */
export function parseDiskUsage(raw: string): DiskUsage {
  const match = raw.trim().match(/^(.+?)\s*\(virtual\s+(.+?)\)$/);
  if (!match) return { writableSize: raw.trim(), virtualSize: null };
  return { writableSize: match[1], virtualSize: match[2] };
}

/** Formats a core count for display, e.g. `4` -> `"4コア"`, `1.5` -> `"1.5コア"`. */
export function formatCores(cores: number): string {
  return `${Number.isInteger(cores) ? cores : cores.toFixed(1)}コア`;
}

const DISPLAY_UNITS = ["B", "KB", "MB", "GB", "TB"];

/** Formats a byte count for display, e.g. `1536` -> `"1.5 KB"`. */
export function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 B";
  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), DISPLAY_UNITS.length - 1);
  const value = bytes / 1024 ** exponent;
  return `${value < 10 ? value.toFixed(1) : Math.round(value)} ${DISPLAY_UNITS[exponent]}`;
}
