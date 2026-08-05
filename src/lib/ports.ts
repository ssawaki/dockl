import type { PortForward } from "$lib/types";

/**
 * Addresses Docker binds to when a port is published without one being named. A single
 * `-p 8080:80` produces one binding per address family, so both of these show up for
 * what the user wrote as one mapping.
 */
const WILDCARD_HOST_IPS = new Set(["0.0.0.0", "::", ""]);

export interface DisplayPort {
  /** The address this is restricted to, or null when published on every address. */
  hostIp: string | null;
  hostPort: string;
  containerPort: string;
  protocol: string;
  /**
   * The host side as one value — `"8080"`, or `"127.0.0.1:8080"` when restricted. This is
   * both what's shown and what gets copied: an address without its port, or a port whose
   * address is a guess, isn't something you can paste anywhere useful.
   */
  address: string;
  /**
   * Where clicking should go, or null when there's nothing navigable.
   *
   * Null for anything that isn't TCP: HTTP/3 does run over UDP, but a UDP endpoint can't
   * be named in a URL — browsers reach HTTP/3 by connecting over TCP first and following
   * an `Alt-Svc` header or HTTPS DNS record. So a UDP row has no target to offer.
   *
   * Always `http://`, never `https://`, even for a container port of 443. Which scheme a
   * server speaks can't be known from a port number, and the two guesses fail very
   * differently: sending plain HTTP to a TLS port gets you the server's own explanation
   * ("You're speaking plain HTTP to an SSL-enabled server port"), while sending TLS to a
   * plain port gets an opaque protocol error. Guess toward the failure that tells you
   * what to do.
   */
  url: string | null;
}

/** IPv6 literals need brackets before a port can be appended. */
function formatHost(ip: string): string {
  return ip.includes(":") ? `[${ip}]` : ip;
}

/**
 * Collapses Docker's per-address-family bindings back into the one port forward the user
 * actually asked for. `-p 8080:80` arrives as two entries — `0.0.0.0` and `::` — that are
 * identical in every way that matters here, so they rendered as two indistinguishable
 * rows.
 *
 * Only the wildcard pair is collapsed. Two *named* addresses sharing a host port
 * (`-p 127.0.0.1:8080:80 -p 192.168.1.5:8080:80`) stay separate rows, because each is a
 * distinct place to connect to and folding them together would leave neither addressable.
 */
export function groupPortForwards(ports: PortForward[]): DisplayPort[] {
  const grouped = new Map<string, DisplayPort>();
  for (const p of ports) {
    const hostIp = WILDCARD_HOST_IPS.has(p.host_ip) ? null : p.host_ip;
    const key = `${hostIp ?? "*"}\t${p.host_port}\t${p.container_port}\t${p.protocol}`;
    if (grouped.has(key)) continue;

    const address = hostIp ? `${formatHost(hostIp)}:${p.host_port}` : p.host_port;
    // Falls back to an explicit loopback address rather than "localhost", which resolves
    // to `::1` first on Windows — and WSL2 publishes container ports onto the IPv4
    // loopback only, so the IPv6 attempt has nothing to connect to.
    const url =
      p.protocol === "tcp" ? `http://${formatHost(hostIp ?? "127.0.0.1")}:${p.host_port}` : null;

    grouped.set(key, {
      hostIp,
      hostPort: p.host_port,
      containerPort: p.container_port,
      protocol: p.protocol,
      address,
      url,
    });
  }
  return [...grouped.values()];
}
