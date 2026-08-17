import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";

import { appName } from "$lib/branding";

export interface BuildInfo {
  /** For display under a heading that already names the app. */
  label: string;
  /** For the clipboard, which arrives somewhere with no such heading. */
  copyValue: string;
}

/**
 * The version of the running binary, with the commit it was built from when there is one
 * (see `emit_git_hash` in build.rs).
 */
export async function buildInfo(): Promise<BuildInfo> {
  const [version, commit] = await Promise.all([
    getVersion(),
    invoke<string | null>("app_commit_hash"),
  ]);

  // Parenthesised rather than "0.1.1-abcdef0": that form is a valid semver prerelease
  // identifier, which would sort the build *below* the release it was cut from.
  const label = commit ? `${version} (${commit})` : version;

  return { label, copyValue: `${appName} ${label}` };
}
