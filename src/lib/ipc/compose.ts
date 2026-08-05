import { invoke } from "@tauri-apps/api/core";

export type ComposeActionKind = "up" | "stop" | "restart" | "down";

/** Resolves with `docker compose`'s own combined output (may be empty). */
export function composeAction(
  project: string,
  configFiles: string[],
  action: ComposeActionKind,
): Promise<string> {
  return invoke("compose_action", { project, configFiles, action });
}
