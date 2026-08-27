//! The tray icon and its right-click menu, in one of two styles the user picks in
//! Settings (`trayFlyoutEnabled` — see `tray_style`). Split into one module per style so
//! either can be read, changed, or (if this ever stops earning its keep) deleted without
//! touching the other:
//!
//! - [`native`]: a native Win32 menu, rebuilt from the current container list on demand.
//!   No window is ever created, so this has no per-open cost at all — the default.
//! - [`flyout`]: a small WebView-rendered popup (`src/routes/tray-menu`) with its own
//!   open animation, at the cost of ~150-300ms to spin up a WebView2 instance per open
//!   (rebuilt fresh each time — see its own doc comment for why it isn't kept alive
//!   hidden instead).
//!
//! This file only owns what's shared: the tray icon itself, the style switch, and
//! left-click (always "open the main window", regardless of style).

// `pub`, not `mod` — `tauri::generate_handler!` (in lib.rs) needs the actual definition
// path (`tray::flyout::tray_menu_open_main`) for its two `#[tauri::command]`s; the
// per-command items that macro expands to aren't carried across a `pub use` re-export.
pub mod flyout;
mod native;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{
    AppHandle, Listener, Manager,
    menu::Menu,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_store::StoreExt;

const TRAY_ID: &str = "main";
const TRAY_MENU_LABEL: &str = "tray-menu";

/// Matches `dockerEvents.svelte.ts`'s own debounce, and for the same reason: a single
/// `docker compose up`/`down` fires a dozen-plus `docker:event`s within milliseconds, and
/// without this each one would trigger its own `list_containers` round trip (a fresh
/// `wsl.exe` process per call under ShellOut mode) to rebuild the native menu.
const DOCKER_EVENT_DEBOUNCE: Duration = Duration::from_millis(400);

/// Bumped by every `apply_style` call and checked before a native menu build applies its
/// result. `apply_style` fires often and concurrently (startup, every `docker:event`,
/// post-connect, the Settings toggle) but its `Native` branch awaits `list_containers`
/// first — without this, a slow call finishing after a newer one (or after the user's
/// switched to `Flyout`) can re-attach a stale menu on top of a newer or absent one.
static MENU_GENERATION: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, PartialEq, Eq)]
enum TrayStyle {
    Native,
    Flyout,
}

/// Reads `trayFlyoutEnabled` from the settings store — `false` (native menu) by default.
/// This project's own priorities put robustness/performance ahead of visual polish (see
/// AGENTS.md), and a native menu has no window-creation cost at all, so it's the safer
/// default; the WebView flyout is there for whoever wants the richer look and doesn't mind
/// the per-open cost that comes with it.
fn tray_style(app: &AppHandle) -> TrayStyle {
    let flyout_enabled = app
        .store("settings.json")
        .ok()
        .and_then(|store| store.get("trayFlyoutEnabled"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    if flyout_enabled {
        TrayStyle::Flyout
    } else {
        TrayStyle::Native
    }
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Builds the persistent tray icon. Left-click always opens/focuses the main window.
/// Right-click's behavior depends on `tray_style`:
/// - Native: a menu stays attached via `tray.set_menu(...)` (see `apply_style`), so
///   Windows shows it itself. That popup and this `Right` arm are independent, though —
///   the click still reaches `on_tray_icon_event` even with a menu attached — so the
///   arm checks the style itself and does nothing while native is active, rather than
///   opening the flyout on top of the native popup that's already showing.
/// - Flyout: no menu is ever attached (`apply_style` clears it), so the `Right` arm below
///   is what actually runs, opening `flyout::show`'s WebView popup instead.
///
/// Switching styles mid-session (`tray_apply_style`, called from Settings) just calls
/// `apply_style` again — nothing here needs to change.
pub fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let flyout_state = flyout::FlyoutState::new();

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip(crate::product_name(app))
        // `tray-icon` defaults this to `true`; without it, a left-click on Windows shows
        // the attached native menu (via WM_LBUTTONUP) on top of `show_main_window` below.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| native::on_menu_event(app, event))
        .on_tray_icon_event(move |tray, event| {
            let app = tray.app_handle();
            // Feeds the plugin the tray icon's actual screen position/size, which
            // `flyout::show`'s `Position::TrayCenter` placement depends on.
            tauri_plugin_positioner::on_tray_event(app, &event);
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => show_main_window(app),
                TrayIconEvent::Click {
                    button: MouseButton::Right,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    if tray_style(app) != TrayStyle::Flyout || flyout_state.recently_closed() {
                        return;
                    }
                    flyout::show(app, &flyout_state);
                }
                _ => {}
            }
        })
        .build(app)?;

    apply_style(app);

    // Keeps the native menu's running-container list current without polling — the same
    // `docker:event` stream every container list in the app already refreshes from (see
    // `DockerEventManager`). `apply_style` is a no-op beyond one `set_menu(None)` call
    // while the flyout style is active. Debounced by `DOCKER_EVENT_DEBOUNCE`: aborts
    // whatever refresh is still pending and schedules a fresh one, so a burst of events
    // settles into a single rebuild instead of one per event.
    let pending_refresh: Arc<Mutex<Option<tauri::async_runtime::JoinHandle<()>>>> =
        Arc::new(Mutex::new(None));
    let app_handle = app.clone();
    app.listen("docker:event", move |_event| {
        let app_handle = app_handle.clone();
        let mut pending = pending_refresh.lock().unwrap();
        if let Some(handle) = pending.take() {
            handle.abort();
        }
        *pending = Some(tauri::async_runtime::spawn(async move {
            tokio::time::sleep(DOCKER_EVENT_DEBOUNCE).await;
            apply_style(&app_handle);
        }));
    });

    Ok(())
}

/// Applies the current `tray_style` to the tray icon: a live menu (rebuilt from the
/// current container list) for native, or none at all for flyout — see `build_tray`'s doc
/// comment for why clearing the menu is what lets the flyout's own click handling run.
/// Called once at startup, again whenever the container list might have changed
/// (`docker:event`), once more right after connecting (`setup_connect`, before any
/// `docker:event` has had a chance to fire), and whenever the user flips the Settings
/// toggle (`tray_apply_style`). A no-op if the tray hasn't been built yet.
pub fn apply_style(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let generation = MENU_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    match tray_style(app) {
        TrayStyle::Native => {
            // Attached synchronously first so a right-click landing before the async
            // rebuild below resolves — a real gap only at startup, since every later call
            // has an already-attached menu to leave in place until this one's ready —
            // still shows something instead of nothing.
            if let Ok(menu) = native::base_menu(app) {
                let _ = tray.set_menu(Some(menu));
            }
            let app = app.clone();
            let tray = tray.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(menu) = native::build_menu(&app).await {
                    // Discard this result if a newer `apply_style` call (a later
                    // `docker:event`, or a switch to `Flyout`) has already landed.
                    if MENU_GENERATION.load(Ordering::SeqCst) == generation {
                        let _ = tray.set_menu(Some(menu));
                    }
                }
            });
        }
        TrayStyle::Flyout => {
            let _ = tray.set_menu(None::<Menu<tauri::Wry>>);
        }
    }
}

/// Re-applies the current `tray_style` — called from Settings when the user flips the
/// flyout toggle, so the change takes effect on the very next click rather than needing an
/// app restart.
#[tauri::command]
pub fn tray_apply_style(app: AppHandle) {
    apply_style(&app);
}
