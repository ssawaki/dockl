import type { NetworkSummary } from "$lib/types";

/**
 * Docker creates these on every install and refuses to remove them. Recognised by name
 * because the Engine API doesn't flag them any other way.
 */
const BUILTIN_NETWORKS = new Set(["bridge", "host", "none"]);

export function isBuiltinNetwork(network: NetworkSummary): boolean {
  return BUILTIN_NETWORKS.has(network.name);
}
