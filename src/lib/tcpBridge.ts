/**
 * Traditional insecure-TCP Docker port. Shared by `TcpBridgeSetupDialog` (setup
 * instructions + verification), the Settings page's connection-mode switch, and
 * `connection.ts`'s startup reconnect — all of which need to agree on the same port.
 */
export const TCP_BRIDGE_PORT = 2375;
