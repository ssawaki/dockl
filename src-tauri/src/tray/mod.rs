use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::{
    AppHandle, Manager, WebviewWindowBuilder, WindowEvent,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_positioner::{Position, WindowExt};

const TRAY_ID: &str = "main";
const TRAY_MENU_LABEL: &str = "tray-menu";
/// A click within this long after either (a) a double-click or (b) the popup losing focus
/// is treated as the tail end of that same gesture rather than a fresh request to open the
/// popup — see the two `recent(...)` checks below for why each exists.
const CLICK_MERGE_WINDOW: Duration = Duration::from_millis(400);

/// Whether `at` (if any) was less than `CLICK_MERGE_WINDOW` ago.
fn recent(at: &Mutex<Option<Instant>>) -> bool {
    at.lock()
        .unwrap()
        .is_some_and(|t| t.elapsed() < CLICK_MERGE_WINDOW)
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Closes the popup outright rather than hiding it. It's rebuilt fresh on every open (see
/// `show_tray_menu`) instead of being kept alive off-screen: Tauri/WebView2 throttles a
/// *hidden* window's JS runtime hard enough to silently break event delivery
/// (tauri-apps/tauri#3654, closed "not planned") — this is what broke
/// `listen("tray-menu:opened", ...)` back when the popup was instead parked at
/// (-10000, -10000) and kept alive.
fn close_tray_menu(app: &AppHandle) {
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
    close_tray_menu(&app);
}

/// Quits the app — called from the popup's "終了" row.
#[tauri::command]
pub fn tray_menu_quit(app: AppHandle) {
    app.exit(0);
}

/// Builds the persistent tray icon. Both left- and right-click open a small
/// WebView-rendered popup (`tray-menu`, declared in `tauri.conf.json`) instead of a native
/// menu, immediately — no delay to wait out a possible second click, so a genuine
/// double-click briefly opens the popup before the `DoubleClick` handler below closes it
/// again and opens the main window instead. That flash is short enough (well under normal
/// double-click cadence) to read as instant rather than as the popup actually opening.
///
/// Windows delivers a double-click as an extra event on top of the two ordinary clicks,
/// not instead of them — both the first *and second* click's `Up` still fire as plain
/// `Click` events — so `last_double_click_at` is what stops that second `Up` from being
/// treated as a fresh click and reopening the popup right after `DoubleClick` just closed
/// it.
///
/// Clicking the tray icon while the popup is already open should toggle it closed, not
/// reopen it: that click first steals focus from the popup, which fires `Focused(false)`
/// and closes it, then reaches this handler as an ordinary `Click` — indistinguishable from
/// a fresh "open" request unless something records that the close *just* happened. That's
/// `last_close_at`, checked by both the left- and right-click arms below.
pub fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    // Set whenever the popup's own `Focused(false)` fires — including from
    // `tray_menu_open_main` itself (opening the main window steals focus from the popup
    // first) — so a same-instant tray click reaching the arms below can tell "this is the
    // tail end of something that just closed the popup" from a fresh request to open one.
    let last_close_at: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
    // Set when a double-click fires, so the ordinary `Up` click that Windows still sends
    // for the double-click's second press (see the doc comment above) doesn't get treated
    // as a fresh single click and reopen the popup right after `show_main_window` ran.
    let last_double_click_at: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip(crate::product_name(app))
        .on_tray_icon_event(move |tray, event| {
            let app = tray.app_handle();
            // Feeds the plugin the tray icon's actual screen position/size, which
            // `show_tray_menu`'s `Position::TrayCenter` placement depends on.
            tauri_plugin_positioner::on_tray_event(app, &event);
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left | MouseButton::Right,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    if recent(&last_double_click_at) || recent(&last_close_at) {
                        return;
                    }
                    show_tray_menu(app, &last_close_at);
                }
                TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                } => {
                    *last_double_click_at.lock().unwrap() = Some(Instant::now());
                    close_tray_menu(app);
                    show_main_window(app);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
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
fn show_tray_menu(app: &AppHandle, last_close_at: &Arc<Mutex<Option<Instant>>>) {
    if app.get_webview_window(TRAY_MENU_LABEL).is_some() {
        close_tray_menu(app);
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
    let Ok(window) = WebviewWindowBuilder::from_config(app, &window_config).and_then(|b| b.build())
    else {
        return;
    };

    let _ = window.set_always_on_top(true);
    let _ = window.move_window_constrained(Position::TrayCenter);
    let _ = window.show();
    let _ = window.set_focus();

    // Registered here (rather than reused across opens) since this window itself is
    // rebuilt on every open — recording into `last_close_at` is what lets `build_tray`'s
    // click handlers tell "the user clicked the tray icon to close this" apart from "open
    // a new one" (see its own doc comment).
    window.on_window_event({
        let app = app.clone();
        let last_close_at = last_close_at.clone();
        move |event| {
            if let WindowEvent::Focused(false) = event {
                *last_close_at.lock().unwrap() = Some(Instant::now());
                close_tray_menu(&app);
            }
        }
    });
}
