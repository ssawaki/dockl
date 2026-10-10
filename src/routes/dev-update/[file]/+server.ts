// Dev only, and removed from production builds with the rest of `dev-*` (see
// scripts/build-without-dev-routes.mjs). The `tauri dev` build's updater endpoint points
// here, so the real `check()` / download code runs against whatever this answers with.
import type { RequestHandler } from "./$types";

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export const GET: RequestHandler = async ({ params, request, url }) => {
  if (params.file === "setup.exe") {
    // The signature in the manifest below is junk, so the download is expected to end in
    // a verification error — after a pause long enough to see the downloading state.
    await sleep(3000);
    return new Response(new Uint8Array(1024), {
      headers: { "content-type": "application/octet-stream" },
    });
  }

  const release = (platforms: Record<string, unknown>) =>
    Response.json({ version: "9.9.9", notes: "dev", platforms });

  switch (request.headers.get("x-dockl-case")) {
    case "available":
      return release({
        "windows-x86_64": { url: `${url.origin}/dev-update/setup.exe`, signature: "junk" },
      });
    case "notFound":
      return new Response("not found", { status: 404 });
    case "badJson":
      return new Response("{ not json", { headers: { "content-type": "application/json" } });
    case "noPlatform":
      return release({});
    case "slow":
      await sleep(4000);
      return new Response(null, { status: 204 });
    default:
      return new Response(null, { status: 204 });
  }
};
