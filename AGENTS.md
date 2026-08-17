# AGENTS.md

## Overview

A Windows 11 desktop GUI for a Docker daemon running inside WSL2. Tauri v2 (Rust) + SvelteKit (Svelte 5). Windows-only. Nothing is installed into WSL2 and no ports are opened — see the connection modes below.

## Commands

```bash
npm run app          # run the dev build (same as npm run tauri dev)
npm run release      # build the production installer
npm run check:all    # svelte-check and cargo check, in parallel
npm run lint         # prettier --check and eslint
npm run format       # prettier --write
```

There are no tests.

`npm run dev` and `npm run build` are the **frontend only** — Tauri invokes them as its `beforeDevCommand` / `beforeBuildCommand`. Neither runs or builds the app.

**`npm run release` is not optional.** `tauri.conf.json` is the dev config (productName `Dockl Dev`, identifier `dev.dockl.desktop.dev`, `icons-dev/`), and `tauri.release.conf.json` overrides those three keys for production. A bare `tauri build` produces a Dev-branded installer — visible in the filename, but wrong.

lefthook runs prettier / eslint / rustfmt over staged files at commit time. Type checks are deliberately excluded for being slow, so run `check:all` by hand.

## Architecture

**Frontend → IPC → backend.** `src/lib/ipc/*.ts` are thin `invoke()` wrappers, one per domain, matching `src-tauri/src/commands/*_cmds.rs`. Commands reach the daemon through `AppState`, which holds an `Option<Arc<dyn DockerConnection>>` — `None` until a connection is established.

**`DockerConnection` (`docker_bridge/connection.rs`)** is the trait every backend implements, so commands never know which mode is live. Two implementations: `EngineApiConnection` (Docker Engine API, over dial-stdio or TCP) and `ShellOutConnection` (`wsl.exe -- docker ...`).

**Connection modes are decided in TypeScript, not Rust.** `src/lib/connection.ts` owns the authoritative list — `shell_out` / `dial_stdio` / `user_managed_tcp` — persists the choice to the store, and calls a different command per mode. The `ConnectionMode` enum in `docker_bridge/connection.rs` is a **vestige of an earlier plan**: it names a `ManagedBridge` that was never built, omits `dial_stdio` (the default), and is referenced only from comments. Don't treat it as current.

`dial_stdio` is the default and preferred mode. It speaks HTTP/1.1 directly over a child process's stdio (`wsl.exe ... docker system dial-stdio`), which is why `hyper` / `hyper-util` are direct dependencies — `reqwest` only dials TCP.

**Some things always shell out, whatever the mode** — `docker events` (`docker_bridge/events.rs`), log streaming (`logs.rs`), Compose actions (`compose/mod.rs`), PTY sessions (`pty_session/`). Each says so at the top of its file.

**Updates are event-driven, not polled.** The app subscribes to `docker events` and refreshes lists from that; the frontend half is `dockerEvents.svelte.ts`.

## Build-time details

**One version number, in package.json.** `tauri.conf.json`'s `version` points at `../package.json`, and `Cargo.toml` is pinned to a placeholder `0.0.0` (which pins `Cargo.lock` with it). Bump with `npm version <patch|minor|major>`.

**`build.rs` rerun triggers are load-bearing.** Without `../package.json`, a version bump leaves the old number in the exe's Windows resource while only the installer filename gets the new one. Without `.git/HEAD` and the ref it points at, the commit hash baked in for the About dialog goes stale. Adding anything else read at build time means adding its trigger too.

**`scripts/build-without-dev-routes.mjs`** moves `src/routes/dev-*` into `node_modules/.cache` for the duration of a build, because SvelteKit code-splits every route whether or not it's reachable. It restores them on the way in as well as on the way out — a killed build can't run its own cleanup.

## Conventions

- **i18n**: `src/lib/i18n/locales/{en,ja,ja-en}.ts` are flat key→string maps whose key sets must stay identical. Interpolation is `{name}`.
- **Comments**: only what can't be inferred from the code. Values restated in prose, and defences of choices nobody would question, get cut. The same standard applies to commit message bodies.
- Svelte 5 runes throughout. Don't assign state from an `$effect` (commit `a15cdbb` removed every instance of that shape).
- `use:` actions and store-based state are dated but deliberately left alone.
