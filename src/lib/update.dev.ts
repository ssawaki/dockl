// Dev only. The `tauri dev` build's updater endpoint is `routes/dev-update/latest.json`, which
// answers according to the `x-dockl-case` header; this is the other half, picking which. Kept
// free of top-level side effects so the import in `update.ts` disappears from production.

export const devCases = [
  { id: "none", label: "最新 (204)" },
  { id: "available", label: "利用可能" },
  { id: "notFound", label: "404" },
  { id: "badJson", label: "不正な JSON" },
  { id: "noPlatform", label: "プラットフォーム無し" },
  { id: "slow", label: "遅延 (4s)" },
] as const;

export type DevCase = (typeof devCases)[number]["id"];

let current: DevCase = "none";

export function setDevCase(id: DevCase) {
  current = id;
}

export function getDevCase() {
  return current;
}

export function devCheckOptions() {
  return { headers: { "x-dockl-case": current } };
}
