import { invoke } from "@tauri-apps/api/core";

export type ComposeActionKind = "up" | "stop" | "down";

export function composeAction(
  project: string,
  configFiles: string[],
  action: ComposeActionKind,
): Promise<void> {
  return invoke("compose_action", { project, configFiles, action });
}
