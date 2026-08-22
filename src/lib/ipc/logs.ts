import { Channel, invoke } from "@tauri-apps/api/core";

export type LogStreamEvent =
  { event: "data"; data: string[] } | { event: "end" } | { event: "error"; data: string };

function eventChannel(onEvent: (event: LogStreamEvent) => void): Channel<LogStreamEvent> {
  return new Channel(onEvent);
}

export function streamLogs(
  id: string,
  onEvent: (event: LogStreamEvent) => void,
  tail = 200,
): Promise<string> {
  return invoke("stream_logs", { id, tail, onEvent: eventChannel(onEvent) });
}

export function streamComposeLogs(
  project: string,
  configFiles: string[],
  onEvent: (event: LogStreamEvent) => void,
  tail = 200,
): Promise<string> {
  return invoke("stream_compose_logs", {
    project,
    configFiles,
    tail,
    onEvent: eventChannel(onEvent),
  });
}

export function stopLogStream(streamId: string): Promise<void> {
  return invoke("stop_log_stream", { streamId });
}
