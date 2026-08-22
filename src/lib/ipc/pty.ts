import { Channel, invoke } from "@tauri-apps/api/core";

export type PtyEvent = { event: "data"; data: string } | { event: "exit" };

function eventChannel(onEvent: (event: PtyEvent) => void): Channel<PtyEvent> {
  return new Channel(onEvent);
}

export function startAttachSession(
  containerId: string,
  cols: number,
  rows: number,
  onEvent: (event: PtyEvent) => void,
  shell?: string,
): Promise<string> {
  return invoke("start_attach_session", {
    containerId,
    shell,
    cols,
    rows,
    onEvent: eventChannel(onEvent),
  });
}

export function startWslShellSession(
  cols: number,
  rows: number,
  onEvent: (event: PtyEvent) => void,
): Promise<string> {
  return invoke("start_wsl_shell_session", { cols, rows, onEvent: eventChannel(onEvent) });
}

export function ptyWrite(sessionId: string, data: string): Promise<void> {
  return invoke("pty_write", { sessionId, data });
}

export function ptyResize(sessionId: string, cols: number, rows: number): Promise<void> {
  return invoke("pty_resize", { sessionId, cols, rows });
}

export function ptyClose(sessionId: string): Promise<void> {
  return invoke("pty_close", { sessionId });
}
