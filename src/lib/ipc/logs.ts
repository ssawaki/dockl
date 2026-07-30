import { invoke } from "@tauri-apps/api/core";

export function streamLogs(id: string, tail = 200): Promise<string> {
  return invoke("stream_logs", { id, tail });
}

export function stopLogStream(streamId: string): Promise<void> {
  return invoke("stop_log_stream", { streamId });
}
