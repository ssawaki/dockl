// Canonical key set. `ja.ts` and `ja-en.ts` are typed against `MessageKey` (derived below),
// so removing/renaming a key here forces the other two catalogs to be updated too.
export const en = {
  // Sidebar navigation / page titles (PageHeader reuses these directly — same text either way).
  "nav.containers": "Containers",
  "nav.images": "Images",
  "nav.volumes": "Volumes",
  "nav.networks": "Networks",
  "nav.storage": "Storage",
  "nav.settings": "Settings",

  "app.connecting": "Connecting to WSL2...",
  "app.starting": "Starting WSL2...",
  "app.connectFailed": "Could not reach WSL2",
  "app.retry": "Retry",

  // Generic, reusable across dialogs/buttons.
  "common.cancel": "Cancel",
  "common.close": "Close",
  "common.loading": "Loading...",
  "common.refresh": "Refresh",
  "common.connecting": "Connecting...",
  "common.copy": "Copy",
  "common.copied": "Copied {value}",
  "common.unknown": "Unknown",
  "common.comingSoon": "Coming soon.",
  "common.yes": "Yes",
  "common.no": "No",
  "common.confirmIrreversible": "This action cannot be undone.",

  // Lowercase resource nouns, for interpolating into prune.* sentences ({resource}).
  "resource.images": "images",
  "resource.volumes": "volumes",
  "resource.networks": "networks",

  // The command's own name, not a description: these run `docker {resource} prune`, which
  // sweeps every unused resource of a kind at once, while the `action.remove` button on
  // each row is that one item's `rm`. Both said "Remove" on the same page — same conflation
  // `action.down` had with `action.remove`.
  "prune.button": "Prune unused {resource}",
  "prune.pending": "Pruning unused {resource}...",
  "prune.success": "Pruned unused {resource}",
  "prune.error": "Failed to prune: {error}",
  "confirmRemove.messageInUse":
    "Removes “{name}”. This will fail if another container is using it.",

  // Shared table column headers (identical wording across resource pages).
  "table.name": "Name",
  "table.composeProject": "Compose project",
  "table.driver": "Driver",
  "table.scope": "Scope",
  "table.size": "Size",

  "titlebar.wslDistro": "Connected WSL distro",
  "titlebar.openShell": "Open WSL shell",
  "titlebar.toggleSidebar": "Toggle sidebar",
  "titlebar.minimize": "Minimize",
  "titlebar.maximize": "Maximize",
  "titlebar.appMenu": "System menu",

  "about.title": "About Dockl",
  "about.version": "Version {version}",

  // Docker action verbs — kept in English in the ja-en catalog.
  "action.start": "Start",
  "action.stop": "Stop",
  "action.restart": "Restart",
  "action.remove": "Remove",
  "action.pause": "Pause",
  "action.unpause": "Resume",
  "action.up": "Start",
  // The command's own name, not a description: this runs `docker compose down`, which
  // removes the project's containers *and* its network, while `action.remove` next to it
  // is a single container's `rm -f`. Both said "Remove" on identical trash icons.
  "action.down": "Down",
  // The confirm button on the prune dialogs, which would otherwise fall back to
  // `action.remove` — the very word these labels exist to stay distinct from.
  "action.prune": "Prune",

  // Full-sentence toast templates per action/phase — {name} is a container or compose
  // project name, {error} is the caught error's String(err). Never compose these from
  // action labels + a shared sentence shell: word order differs too much between locales.
  "toast.start.pending": "Starting {name}...",
  "toast.start.success": "Started {name}",
  "toast.start.error": "Failed to start {name}: {error}",
  "toast.stop.pending": "Stopping {name}...",
  "toast.stop.success": "Stopped {name}",
  "toast.stop.error": "Failed to stop {name}: {error}",
  "toast.restart.pending": "Restarting {name}...",
  "toast.restart.success": "Restarted {name}",
  "toast.restart.error": "Failed to restart {name}: {error}",
  "toast.remove.pending": "Removing {name}...",
  "toast.remove.success": "Removed {name}",
  "toast.remove.error": "Failed to remove {name}: {error}",
  "toast.pause.pending": "Pausing {name}...",
  "toast.pause.success": "Paused {name}",
  "toast.pause.error": "Failed to pause {name}: {error}",
  "toast.unpause.pending": "Resuming {name}...",
  "toast.unpause.success": "Resumed {name}",
  "toast.unpause.error": "Failed to resume {name}: {error}",
  "toast.up.pending": "Starting {name}...",
  "toast.up.success": "Started {name}",
  "toast.up.error": "Failed to start {name}: {error}",
  // "Taking down" rather than "Removing": it matches `action.down`'s label while still
  // reading as a sentence, and keeps these distinguishable from `toast.remove.*` — the
  // two were word-for-word identical, so a toast gave no hint whether one container or a
  // whole Compose project had just gone away.
  "toast.down.pending": "Taking down {name}...",
  "toast.down.success": "Took down {name}",
  "toast.down.error": "Failed to take down {name}: {error}",

  // Translations for Rust-side failures, keyed by `AppError::code` — see
  // `src/lib/errors.ts`. Only the ones a user can act on are here; the rest keep their
  // original text, which is docker's or WSL's own output and not ours to reword.
  "errors.connectTimeout":
    "WSL2 stopped responding (no reply within {seconds}s). It may recover on its own — try again in a moment.",
  "errors.notConfigured": "Not connected to Docker yet. Complete setup from Settings.",
  "errors.wslUnavailable": "Could not reach WSL2: {detail}",
  "errors.noDistroFound": "No WSL2 distro with Docker was found.",

  "errors.noDistroSelected": "No connection target selected",
  "errors.connectionSwitchFailed": "Failed to switch connection method: {error}",
  "errors.notConnectedSetupRequired": "No saved connection info. Please complete initial setup.",
  "errors.copyFailed": "Failed to copy: {error}",

  "toastStack.copyError": "Copy error",
  "toastStack.showDetails": "Show details",

  "containers.loading": "Loading containers...",
  "containers.empty": "No containers found.",
  "containers.section.running": "Running",
  "containers.section.stopped": "Stopped",
  "containers.status.allRunning": "All running",
  "containers.status.partiallyRunning": "Partially running",
  "containers.showInExplorer": "Show in Explorer",
  "containers.expand": "Expand",
  "containers.collapse": "Collapse",
  "containers.expandAriaLabel": "Expand {name}",
  "containers.collapseAriaLabel": "Collapse {name}",
  "containers.confirmRemove.title": "Remove container",
  "containers.confirmRemove.message": "Stops “{name}”, then removes it.",
  "containers.confirmComposeDown.title": "Take down Compose project",
  "containers.confirmComposeDown.message":
    "Stops “{project}” and removes its containers and networks (volumes are kept).",
  "errors.explorerOpenFailed": "Failed to open in Explorer: {error}",

  "containers.tab.info": "Info",
  "containers.tab.stats": "Stats",
  "containers.tab.logs": "Logs",
  "containers.tab.terminal": "Terminal",
  "containers.detail.placeholder": "Select a container from the list on the left.",
  "containers.detail.terminalUnavailable": "Only running containers can connect to a terminal.",
  "containers.detail.currentStatus": "Current status: {status}",

  "stats.notRunningHint": "CPU, memory, and I/O stats are only available for running containers.",
  "stats.memory": "Memory",
  "stats.max": "MAX {percent}% ({cores})",
  "stats.blockIO": "Block I/O",
  "stats.blockIO.value": "Read {read} / Write {write}",
  "stats.networkIO": "Network I/O",
  "stats.loading": "Loading stats...",
  "stats.storageTotal": "Total (including image): {size}",

  "terminal.wrapOff": "No wrap",
  "terminal.wrapOn": "Wrap",
  "terminal.wrapOffAriaLabel": "Turn off wrapping",
  "terminal.wrapOnAriaLabel": "Turn on wrapping",
  "logs.ended": "Log output ended (the container stopped).",
  "terminal.sessionEnded": "Session ended",

  "terminal.search.placeholder": "Search",
  "terminal.search.ariaLabel": "Search within the terminal",
  "terminal.search.noMatches": "No matches",
  "terminal.search.previous": "Previous",
  "terminal.search.previousAriaLabel": "Find previous",
  "terminal.search.next": "Next",
  "terminal.search.nextAriaLabel": "Find next",
  "terminal.search.closeAriaLabel": "Close search",

  "wslShell.title": "WSL Shell",

  "tcpEndpoint.title": "TCP endpoint (Docker Engine API)",
  "tcpEndpoint.description1a":
    "The “TCP connection” mode talks to a Docker Engine API–compatible endpoint at ",
  "tcpEndpoint.description1b":
    " that you have exposed yourself — Podman's `podman system service`, for instance. Dockl does not open this port for you: the relay process mode reaches the same API at the same measured speed without one, so there is no reason for Dockl to create it.",
  "tcpEndpoint.warning":
    "If the endpoint you exposed has no authentication, any program on this Windows machine can drive the Docker API through it, which amounts to root inside WSL2 — Windows Firewall doesn't inspect loopback traffic, and a web page can be made to reach it too. Only use this on a machine you trust.",
  "tcpEndpoint.checkConnection": "Check connection",
  "tcpEndpoint.connectionOk": "Connection verified",
  "tcpEndpoint.connectionFailed": "Could not connect: {error}",
  "tcpEndpoint.teardown":
    "If an earlier version of Dockl set up this port for you, this removes that configuration and restores the Docker daemon to its original state (running containers restart):",

  "toastOutput.empty": "No output.",

  "images.confirmRemove.title": "Remove image",
  "images.table.image": "Image",
  "images.table.created": "Created",
  "images.table.containersInUse": "Containers in use",
  "images.loading": "Loading images...",
  "images.empty": "No images found.",
  "images.section.inUse": "In use",
  "images.section.unused": "Unused",
  "images.detail.placeholder": "Select an image from the list on the left.",
  "images.detail.id": "Image ID",
  "images.detail.tags": "Tags",
  "images.detail.noTags": "This image has no tags (dangling).",
  // Shown next to the one name the list has room for, so an image answering to several
  // names doesn't look like it only has one.
  "images.moreTags": "+{count}",
  "images.prune.message": "Prunes images not referenced by any container.",
  "images.prune.includeTagged": "Also prune tagged unused images",

  "volumes.confirmRemove.title": "Remove volume",
  "volumes.table.mountpoint": "Mount point",
  "volumes.loading": "Loading volumes...",
  "volumes.empty": "No volumes found.",
  "volumes.detail.placeholder": "Select a volume from the list on the left.",
  "volumes.prune.message":
    "Prunes volumes not referenced by any container. Their data will be lost.",
  "volumes.prune.includeNamed": "Also prune named unused volumes",

  "networks.confirmRemove.title": "Remove network",
  "networks.loading": "Loading networks...",
  "networks.empty": "No networks found.",
  "networks.builtinCannotRemove": "Built-in Docker networks cannot be removed",
  "networks.detail.placeholder": "Select a network from the list on the left.",
  "networks.detail.id": "Network ID",
  "networks.detail.internal": "Internal",
  "networks.prune.message": "Prunes networks not used by any container.",

  "storage.loading": "Loading disk usage...",
  "storage.totalLabel": "Total storage used by Docker",
  "storage.totalHint": "Approximate — space shared between categories may be double-counted.",
  "storage.table.kind": "Type",
  "storage.table.count": "Count",
  "storage.table.active": "Active",
  "storage.table.reclaimable": "Reclaimable",
  "storage.kind.localVolumes": "Local Volumes",
  "storage.kind.buildCache": "Build Cache",
  "storage.pruneBuildCache": "Prune build cache",
  "storage.pruneBuildCache.message": "Prunes build cache not used by any image.",
  "storage.pruneBuildCache.includeReusable":
    "Also prune cache that could be reused for future builds",
  "storage.pruneBuildCache.pending": "Pruning build cache...",
  "storage.pruneBuildCache.success": "Pruned build cache",

  "settings.appearance.heading": "Appearance",
  "settings.appearance.theme.label": "Theme",
  "settings.appearance.theme.system": "System",
  "settings.appearance.theme.light": "Light",
  "settings.appearance.theme.dark": "Dark",
  "settings.appearance.background.label": "Background",
  "settings.appearance.background.mica": "Mica",
  "settings.appearance.background.acrylic": "Acrylic",
  "settings.appearance.background.solid": "Solid color",
  "settings.appearance.sidebarHoverExpand": "Expand the sidebar on hover",
  "settings.appearance.error": "Failed to display appearance settings ({error})",

  "settings.language.heading": "Language",
  "settings.language.ja": "日本語",
  "settings.language.jaEn": "日本語 + English",
  "settings.language.en": "English",

  "settings.connection.heading": "Connection",
  "settings.connection.description":
    "How Dockl talks to Docker inside WSL2 (used for listing and operating on containers, images, volumes, and networks).",
  "settings.connection.recommended": "Recommended",
  "settings.connection.dialStdio": "Relay process (dial-stdio)",
  "settings.connection.dialStdio.desc":
    "Reaches the Docker Engine API through a relay process Dockl starts itself. It makes no changes to WSL2 at all, so there is nothing to set up or undo.",
  "settings.connection.shellOut": "No bridge (CLI shell-out)",
  "settings.connection.shellOut.desc":
    "Runs the docker CLI for every operation. The simplest option, but each call spawns a wsl.exe process, which makes it slower. Dockl also falls back to this automatically whenever the relay process is unavailable.",
  "settings.connection.tcp": "TCP connection (Docker Engine API)",
  "settings.connection.tcp.desc":
    "Connects to a Docker Engine API–compatible endpoint you have exposed over TCP yourself, such as Podman's. Dockl does not open the port — it is no faster than the relay process, so there is nothing to gain unless the relay process is unavailable to you.",
  "settings.connection.switching": "Switching...",
  "settings.connection.current": "Current connection:",
  "settings.connection.notConnected": "Not connected",
  "settings.connection.reconnect": "Reconnect",
  "settings.connection.tcpSetupLabel": "TCP endpoint (Docker Engine API):",
  "settings.connection.setupButton": "Details...",

  "settings.tray.heading": "System tray",
  "settings.tray.toggle": "Keep running in the system tray when the window is closed",

  "settings.autostart.heading": "Startup",
  "settings.autostart.toggle": "Launch Dockl automatically when you sign in to Windows",

  "setup.welcome": "Welcome",
  "setup.lead": "Select the WSL2 distro running Docker.",
  "setup.detecting": "Detecting WSL distros...",
  "setup.noneFound": "No WSL2 distros were found.",
  "setup.noneFoundHint": "Install WSL2 and set up Docker, then try again.",
  "setup.running": "Running",
  "setup.stopped": "Stopped",
  "setup.default": "Default",
  "setup.connect": "Connect",
} as const satisfies Record<string, string>;

export type MessageKey = keyof typeof en;
