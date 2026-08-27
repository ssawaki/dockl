//! The WebView-rendered tray popup (`TrayStyle::Flyout` — see `super::tray_style`).
//!
//! Rebuilt fresh on every open and closed outright rather than kept alive hidden:
//! Tauri/WebView2 throttles a *hidden* window's JS runtime hard enough to silently break
//! event delivery (tauri-apps/tauri#3654, closed "not planned") — this is what broke
//! `listen("tray-menu:opened", ...)` back when the popup was instead parked at
//! (-10000, -10000) and kept alive.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_positioner::{Position, WindowExt};
use tauri_plugin_store::StoreExt;

use super::{TRAY_MENU_LABEL, show_main_window};

/// A click within this long after the popup losing focus is treated as the tail end of
/// that same gesture (the user clicking the tray icon to toggle it closed) rather than a
/// fresh request to open a new one — see the `recent` check in `show`.
const CLICK_MERGE_WINDOW: Duration = Duration::from_millis(400);

fn close(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(TRAY_MENU_LABEL) {
        let _ = window.close();
    }
}

/// Opens the main window and dismisses the tray popup — called from the popup's own
/// "Docklを開く" button and running-container rows (`src/routes/tray-menu`); the
/// container rows additionally `emit("tray:select-container", id)` themselves first (see
/// `+layout.svelte`), same as the "設定" row does with `tray:open-settings`.
#[tauri::command]
pub fn tray_menu_open_main(app: AppHandle) {
    show_main_window(&app);
    close(&app);
}

/// Quits the app — called from the popup's "終了" row.
#[tauri::command]
pub fn tray_menu_quit(app: AppHandle) {
    app.exit(0);
}

/// Set whenever the popup's own `Focused(false)` fires — including from
/// `tray_menu_open_main` itself (opening the main window steals focus from the popup
/// first) — so a same-instant right-click reaching `super::build_tray`'s handler can tell
/// "this is the tail end of something that just closed the popup" from a fresh request to
/// open one.
pub struct FlyoutState {
    last_close_at: Arc<Mutex<Option<Instant>>>,
}

impl FlyoutState {
    pub fn new() -> Self {
        Self {
            last_close_at: Arc::new(Mutex::new(None)),
        }
    }

    /// Whether a close was recorded less than `CLICK_MERGE_WINDOW` ago.
    pub fn recently_closed(&self) -> bool {
        self.last_close_at
            .lock()
            .unwrap()
            .is_some_and(|t| t.elapsed() < CLICK_MERGE_WINDOW)
    }
}

/// Builds a fresh popup window, positions it just above the tray icon, and shows it.
/// `move_window_constrained` (tauri-plugin-positioner) resolves that against whichever
/// monitor the tray icon is actually on and clamps it to that monitor's bounds, so this
/// doesn't need its own multi-monitor/taskbar-position handling.
///
/// `show()` has no fade of its own — tried driving one through Win32's `AnimateWindow`
/// instead, but `AW_BLEND` bypasses DWM composition and paints with the pre-Vista GDI
/// blend path, which looked glaringly out of place applied to a `transparent: true` window
/// on Windows 11 (a solid, low-color-depth flash rather than a smooth fade). Plain `show()`
/// it stays; `src/routes/tray-menu/+page.svelte` plays a CSS fade of its own instead.
///
/// If a window with this label somehow still exists (`WebviewWindow::close()` posts to the
/// event loop rather than closing synchronously, so a stale one right after a close is
/// possible in principle), this closes it and returns instead of also trying to `build()`
/// a second window under the same label, which would just fail — closing it is also the
/// right call behaviorally, since it means the user's click reached here before the
/// previous popup finished tearing down and reads the same as clicking to toggle it shut.
pub fn show(app: &AppHandle, state: &FlyoutState) {
    if app.get_webview_window(TRAY_MENU_LABEL).is_some() {
        close(app);
        return;
    }

    let Some(window_config) = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == TRAY_MENU_LABEL)
        .cloned()
    else {
        return;
    };

    // The popup has no `store:default` permission (`capabilities/tray-menu.json`) and
    // skips `initAppearance()` entirely (`+layout.svelte`'s `isTrayMenu` guard), so nothing
    // would otherwise set the `data-theme` attribute `theme.css` keys its dark palette off
    // of — it would always render light. Resolved here instead of granting that permission
    // back: an init script (runs before the page's own scripts) sets it directly, deciding
    // "system" the same way `resolveDark` does on the frontend, via `matchMedia` in the
    // popup's own page context.
    let theme_mode = app
        .store("settings.json")
        .ok()
        .and_then(|store| store.get("themeMode"))
        .and_then(|value| value.as_str().map(str::to_string))
        .filter(|mode| matches!(mode.as_str(), "light" | "dark"))
        .unwrap_or_else(|| "system".to_string());
    let theme_init_script = format!(
        r#"document.documentElement.dataset.theme =
            {theme_mode:?} === "dark" ||
            ({theme_mode:?} === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches)
            ? "dark" : "light";"#
    );

    let Ok(window) = WebviewWindowBuilder::from_config(app, &window_config)
        .map(|b| b.initialization_script(&theme_init_script))
        .and_then(|b| b.build())
    else {
        return;
    };

    let _ = window.set_always_on_top(true);
    let _ = window.move_window_constrained(Position::TrayCenter);
    let _ = window.show();
    let _ = window.set_focus();

    // Registered here (rather than reused across opens) since this window itself is
    // rebuilt on every open — recording into `last_close_at` is what lets `recently_closed`
    // tell "the user clicked the tray icon to close this" apart from "open a new one".
    let last_close_at = state.last_close_at.clone();
    window.on_window_event({
        let app = app.clone();
        move |event| {
            if let WindowEvent::Focused(false) = event {
                *last_close_at.lock().unwrap() = Some(Instant::now());
                close(&app);
            }
        }
    });
}
