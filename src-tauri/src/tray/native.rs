//! The native right-click menu (`TrayStyle::Native` — see `super::tray_style`), rebuilt
//! from the current container list whenever it might have changed. Costs nothing beyond
//! the `list_containers` call itself — no window is ever created — which is the whole
//! reason this is the default style (see `super::tray_style`'s doc comment).

use tauri::{
    AppHandle, Emitter, Manager,
    menu::{IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

use crate::docker_bridge::types::ContainerSummary;
use crate::state::AppState;

use super::show_main_window;

const CONTAINER_ITEM_PREFIX: &str = "container:";

/// Builds the menu fresh from whichever containers are running right now. Errors (no
/// connection yet, a transient `list_containers` failure) fall back to an empty
/// container list rather than failing the whole menu — "開く"/"設定"/"終了" should stay
/// available regardless.
pub async fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let state = app.state::<AppState>();
    let running = match state.connection().await {
        Ok(connection) => connection
            .list_containers(false)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|c| c.state == "running")
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };
    menu_from(app, &running)
}

/// The same menu `build_menu` would produce for an empty container list — no async work,
/// so `apply_style` can attach it immediately as a stand-in for the moment before
/// `build_menu`'s `list_containers` call resolves (otherwise a right-click landing in that
/// window, which only exists right after startup, would show no menu at all).
pub fn base_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    menu_from(app, &[])
}

fn menu_from(app: &AppHandle, running: &[ContainerSummary]) -> tauri::Result<Menu<tauri::Wry>> {
    let show_item = MenuItem::with_id(
        app,
        "show",
        format!("{}を開く", crate::product_name(app)),
        true,
        None::<&str>,
    )?;
    let settings_item = MenuItem::with_id(app, "settings", "設定", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;

    let mut items: Vec<Box<dyn IsMenuItem<tauri::Wry>>> = vec![Box::new(show_item)];
    if !running.is_empty() {
        items.push(Box::new(PredefinedMenuItem::separator(app)?));
        items.push(Box::new(MenuItem::new(
            app,
            "起動中のコンテナ",
            false,
            None::<&str>,
        )?));
        for container in running {
            items.push(Box::new(MenuItem::with_id(
                app,
                format!("{CONTAINER_ITEM_PREFIX}{}", container.id),
                container.names.join(", "),
                true,
                None::<&str>,
            )?));
        }
    }
    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(settings_item));
    items.push(Box::new(quit_item));

    let refs: Vec<&dyn IsMenuItem<tauri::Wry>> = items.iter().map(|item| item.as_ref()).collect();
    Menu::with_items(app, &refs)
}

/// Dispatches a menu item click. Registered once on the tray icon regardless of the
/// current style (see `build_tray`) — while the flyout style is active this simply never
/// fires, since no native menu is ever attached for Windows to route a click through.
pub fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    let id = event.id.as_ref();
    if let Some(container_id) = id.strip_prefix(CONTAINER_ITEM_PREFIX) {
        show_main_window(app);
        let _ = app.emit("tray:select-container", container_id);
        return;
    }
    match id {
        "show" => show_main_window(app),
        "settings" => {
            show_main_window(app);
            let _ = app.emit("tray:open-settings", ());
        }
        "quit" => app.exit(0),
        _ => {}
    }
}
