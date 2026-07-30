import { invoke } from "@tauri-apps/api/core";

export function startAttachSession(
  containerId: string,
  cols: number,
  rows: number,
  shell?: string,
): Promise<string> {
  return invoke("start_attach_session", { containerId, shell, cols, rows });
}

export function startWslShellSession(cols: number, rows: number): Promise<string> {
  return invoke("start_wsl_shell_session", { cols, rows });
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
