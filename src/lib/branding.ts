// Kept in step with tauri.dev.conf.json by hand: that file gives the dev build its own
// productName, identifier and bundle icons, but none of it reaches the webview, so the
// title bar and About dialog would otherwise show the release name and mark while the
// taskbar and tray showed the dev ones.
//
// `import.meta.env.DEV` is statically replaced, so the unused branch and its asset are
// gone from a production build.

export const appName = import.meta.env.DEV ? "Dockl Dev" : "Dockl";

export const appIcon = import.meta.env.DEV ? "/app-icon-dev.png" : "/app-icon.png";
