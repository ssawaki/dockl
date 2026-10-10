# AGENTS.md

## Overview

A Windows 11 desktop GUI for a Docker daemon running inside WSL2. Tauri v2 (Rust) + SvelteKit (Svelte 5). Windows-only. Nothing is installed into WSL2 and no ports are opened — see the connection modes below.

## Commands

```bash
npm run app          # run the dev build (`tauri dev` plus tauri.dev.conf.json)
npm run installer    # build the production installer
npm run check:all    # svelte-check and cargo check, in parallel
npm run lint         # prettier --check and eslint
npm run format       # prettier --write
```

There are no tests.

`npm run dev` and `npm run build` are the **frontend only** — Tauri invokes them as its `beforeDevCommand` / `beforeBuildCommand`. Neither runs or builds the app.

**`npm run installer` is not optional.** `tauri.conf.json` is the dev config (productName `Dockl Dev`, identifier `dev.dockl.desktop.dev`, `icons-dev/`), and `tauri.release.conf.json` overrides those three keys for production, plus the `publisher` / `homepage` that end up in the installer's metadata. A bare `tauri build` produces a Dev-branded installer — visible in the filename, but wrong.

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

## Releasing

```bash
npm version patch          # bumps package.json and tags v0.1.3
git push --follow-tags     # a plain `git push` leaves the tag behind
```

The tag is what fires `.github/workflows/release.yml`; nothing else builds an installer.
It attaches both the NSIS installer and the bare exe (renamed to carry the version) to a
**draft** Release, so publishing stays a deliberate act — the bundle is unsigned, and a
draft's assets aren't reachable until someone presses the button. That same gate is what
the in-app updater (`tauri-plugin-updater`) relies on: it polls
`releases/latest/download/latest.json`, which the workflow generates and uploads alongside
a `.sig` file (a separate Ed25519 signature, unrelated to code-signing), but which 404s
until the draft is published. Signing needs `TAURI_SIGNING_PRIVATE_KEY` /
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` as repo secrets — lose that key and existing installs
can no longer verify (and therefore install) any future update.

The same two variables are needed by a local `npm run installer`: `createUpdaterArtifacts`
makes the build fail at the signing step without them, after the installer itself is
already built. `TAURI_SIGNING_PRIVATE_KEY` takes the key's contents, not a path
(`..._PATH` is only for `tauri signer sign`).

`npm run app` doesn't touch the real endpoint: `tauri.dev.conf.json` points the updater at
`routes/dev-update`, a dev route that answers by header, and the Settings page picks which
response the next check gets. The override lives in that dev-only file rather than the other
way round so that forgetting it fails safe — a bare `tauri dev` just checks GitHub.

**winget is not automated.** After publishing the draft, a manifest PR to
`microsoft/winget-pkgs` for `Dockl.Dockl` is still a manual step, e.g.
`wingetcreate update Dockl.Dockl --version <v> --urls <setup.exe url> --submit`. Skipping it
doesn't break the in-app updater, but winget users stay on the old version.

`.github/workflows/ci.yml` runs the two type checks lefthook leaves out, on every push and
PR. It builds the frontend first because `cargo check` needs `frontendDist` to exist, and
`.gitattributes` forces LF because the Windows runners check out CRLF, which
`prettier --check` rejects.

## Conventions

- **i18n**: `src/lib/i18n/locales/{en,ja,ja-en}.ts` are flat key→string maps whose key sets must stay identical. Interpolation is `{name}`.
- **Comments**: only what can't be inferred from the code. Values restated in prose, and defences of choices nobody would question, get cut. The same standard applies to commit message bodies.
- Svelte 5 runes throughout. Don't assign state from an `$effect` (commit `a15cdbb` removed every instance of that shape).
- `use:` actions and store-based state are dated but deliberately left alone.
