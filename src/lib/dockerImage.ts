interface ParsedImageRef {
  domain: string | null;
  /** Path segments after the domain, e.g. ["ec-cube", "ec-cube-php"]. */
  parts: string[];
  tag: string | null;
}

function parseImageRef(image: string): ParsedImageRef {
  const withoutDigest = image.split("@")[0];

  const firstSlash = withoutDigest.indexOf("/");
  const maybeDomain = firstSlash === -1 ? withoutDigest : withoutDigest.slice(0, firstSlash);
  const looksLikeDomain =
    firstSlash !== -1 &&
    (maybeDomain.includes(".") || maybeDomain.includes(":") || maybeDomain === "localhost");

  const domain = looksLikeDomain ? maybeDomain : null;
  const remainder = looksLikeDomain ? withoutDigest.slice(firstSlash + 1) : withoutDigest;

  // The domain (which may itself contain a ':port') has already been separated off
  // above, so any remaining colon in the final path segment belongs to a tag.
  const lastSlash = remainder.lastIndexOf("/");
  const lastSegment = lastSlash === -1 ? remainder : remainder.slice(lastSlash + 1);
  const colonInLastSegment = lastSegment.lastIndexOf(":");

  let repoPath = remainder;
  let tag: string | null = null;
  if (colonInLastSegment !== -1) {
    tag = lastSegment.slice(colonInLastSegment + 1);
    repoPath = remainder.slice(0, remainder.length - (lastSegment.length - colonInLastSegment));
  }

  return { domain, parts: repoPath.split("/").filter(Boolean), tag };
}

/**
 * Splits an image reference into the "linkable" repo portion (with any registry
 * domain) and its tag, so the UI can render just the repo as a link while leaving
 * `:tag` as plain text next to it.
 */
export function splitImageTag(image: string): { repo: string; tag: string | null } {
  const { tag } = parseImageRef(image);
  if (tag === null) {
    return { repo: image, tag: null };
  }
  return { repo: image.slice(0, image.length - (tag.length + 1)), tag };
}

/**
 * Best-effort mapping from a Docker image reference to its registry web page, for
 * linkifying the "Image" field in the container detail panel. Only registries with a
 * reliable, well-known URL convention are handled; everything else returns `null` and
 * is left as plain text rather than guessing a URL that might be wrong.
 */
export function imageRegistryUrl(image: string): string | null {
  const { domain, parts } = parseImageRef(image);

  if (!domain || domain === "docker.io" || domain === "index.docker.io") {
    if (parts.length === 1) {
      return `https://hub.docker.com/_/${parts[0]}`;
    }
    if (parts.length >= 2) {
      return `https://hub.docker.com/r/${parts[0]}/${parts.slice(1).join("/")}`;
    }
    return null;
  }

  if (domain === "ghcr.io" && parts.length >= 2) {
    const owner = parts[0];
    const packageName = parts.slice(1).join("/");
    const repo = parts[parts.length - 1];
    return `https://github.com/${owner}/${repo}/pkgs/container/${encodeURIComponent(packageName)}`;
  }

  if (domain === "quay.io" && parts.length >= 2) {
    return `https://quay.io/repository/${parts[0]}/${parts.slice(1).join("/")}`;
  }

  return null;
}
