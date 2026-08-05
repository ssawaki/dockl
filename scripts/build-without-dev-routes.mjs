#!/usr/bin/env node
// `vite build` (invoked via `tauri build`'s beforeBuildCommand) must never ship dev-only
// pages in the compiled output. SvelteKit code-splits every route into its own chunk
// regardless of whether it's reachable, so gating access at runtime (e.g. checking
// import.meta.env.DEV inside the page) still leaves the chunk sitting in `build/`.
// Instead, this script physically moves every `src/routes/dev-*` folder out of the way
// before building and puts it back afterwards (even on failure), so vite never sees it.
import { execSync } from "node:child_process";
import { mkdtempSync, readdirSync, renameSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const routesDir = join(process.cwd(), "src", "routes");

const devDirs = readdirSync(routesDir, { withFileTypes: true })
  .filter((entry) => entry.isDirectory() && entry.name.startsWith("dev-"))
  .map((entry) => entry.name);

const stashRoot = devDirs.length > 0 ? mkdtempSync(join(tmpdir(), "dockl-dev-routes-")) : null;

for (const name of devDirs) {
  renameSync(join(routesDir, name), join(stashRoot, name));
  console.log(`[build] excluded dev route: src/routes/${name}`);
}

let exitCode = 0;
try {
  execSync("vite build", { stdio: "inherit" });
} catch (err) {
  exitCode = typeof err.status === "number" ? err.status : 1;
} finally {
  for (const name of devDirs) {
    renameSync(join(stashRoot, name), join(routesDir, name));
  }
  if (stashRoot) rmSync(stashRoot, { recursive: true, force: true });
}

process.exit(exitCode);
