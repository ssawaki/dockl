#!/usr/bin/env node
// `vite build` (invoked via `tauri build`'s beforeBuildCommand) must never ship dev-only
// pages in the compiled output. SvelteKit code-splits every route into its own chunk
// regardless of whether it's reachable, so gating access at runtime (e.g. checking
// import.meta.env.DEV inside the page) still leaves the chunk sitting in `build/`.
// Instead, this script physically moves every `src/routes/dev-*` folder out of the way
// before building and puts it back afterwards (even on failure), so vite never sees it.
//
// Putting them back is the part that needs care. `finally` covers a failed build but not
// a killed one, and on Windows a hard kill delivers no signal there'd be a chance to
// handle — so the routes can end up stranded in the stash with the working tree missing
// them, and nothing notices, because the next run finds no `dev-*` to stash and sails
// past. Hence a fixed stash path that `restore()` sweeps on the way in, rather than a
// fresh mkdtemp one nobody could find again.
import { execSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, renameSync, rmSync } from "node:fs";
import { join } from "node:path";

const routesDir = join(process.cwd(), "src", "routes");

// Under node_modules because it's already gitignored and is where build tooling is
// expected to keep scratch state. `npm ci` would wipe a stash sitting here, but only a
// stash left behind by a killed run, and only before the next build swept it up — and
// what it'd take out are routes git still has.
const stashRoot = join(process.cwd(), "node_modules", ".cache", "dockl-dev-routes");

function restore() {
  if (!existsSync(stashRoot)) return;

  for (const name of readdirSync(stashRoot)) {
    const target = join(routesDir, name);

    // Something already put this route back — a `git checkout` after a killed run is the
    // likely story. The working tree wins: overwriting it with the stashed copy would
    // throw away whatever that checkout restored.
    if (existsSync(target)) {
      console.warn(`[build] left stashed copy of ${name} at ${join(stashRoot, name)}`);
      continue;
    }

    renameSync(join(stashRoot, name), target);
    console.log(`[build] restored dev route: src/routes/${name}`);
  }

  if (readdirSync(stashRoot).length === 0) rmSync(stashRoot, { recursive: true, force: true });
}

// Whatever a previous run failed to put back.
restore();

const devDirs = readdirSync(routesDir, { withFileTypes: true })
  .filter((entry) => entry.isDirectory() && entry.name.startsWith("dev-"))
  .map((entry) => entry.name);

if (devDirs.length > 0) mkdirSync(stashRoot, { recursive: true });

for (const name of devDirs) {
  renameSync(join(routesDir, name), join(stashRoot, name));
  console.log(`[build] excluded dev route: src/routes/${name}`);
}

// Ctrl+C is the one interruption there's still a chance to act on. SIGTERM and SIGHUP go
// unfired on Windows, and no handler runs for a hard kill at all; `restore()` above is
// what covers those.
for (const signal of ["SIGINT", "SIGTERM", "SIGHUP", "SIGBREAK"]) {
  process.on(signal, () => {
    restore();
    process.exit(130);
  });
}

let exitCode = 0;
try {
  execSync("vite build", { stdio: "inherit" });
} catch (err) {
  exitCode = typeof err.status === "number" ? err.status : 1;
} finally {
  restore();
}

process.exit(exitCode);
