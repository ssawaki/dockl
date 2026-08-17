mod appearance;
mod commands;
mod compose;
mod docker_bridge;
mod error;
mod pty_session;
mod state;
mod system_menu;
mod tray;
mod wsl;

use tauri::{Manager, WebviewWindowBuilder};
use tauri_plugin_store::StoreExt;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be registered first: focuses the existing window instead of letting a
        // second instance start when the user launches Dockl again.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            system_menu::show_system_menu,
            commands::list_containers,
            commands::container_action,
            commands::inspect_container,
            commands::setup_list_distros,
            commands::setup_connect,
            commands::setup_current_distro,
            commands::check_tcp_bridge,
            commands::connect_tcp_bridge,
            commands::connect_dial_stdio,
            commands::setup_distro_is_running,
            commands::compose_action,
            commands::list_images,
            commands::remove_image,
            commands::prune_images,
            commands::list_volumes,
            commands::remove_volume,
            commands::prune_volumes,
            commands::list_networks,
            commands::remove_network,
            commands::prune_networks,
            commands::app_commit_hash,
            commands::get_disk_usage,
            commands::prune_build_cache,
            commands::stream_logs,
            commands::stream_compose_logs,
            commands::stop_log_stream,
            commands::get_container_stats,
            commands::get_host_cpu_count,
            commands::get_container_disk_usage,
            commands::start_attach_session,
            commands::start_wsl_shell_session,
            commands::pty_write,
            commands::pty_resize,
            commands::pty_close,
            commands::set_window_material,
        ])
        .setup(|app| {
            // Built here instead of letting `tauri.conf.json`'s `windows` entry
            // auto-create it (it's marked `"create": false` there) so
            // `.enable_clipboard_access()` can be chained on — otherwise, since the
            // dev build serves the frontend from a real `http://localhost:<port>`
            // origin, WebView2 treats `navigator.clipboard.readText()` (used by the
            // terminal's paste handling) exactly like any other website would and
            // prompts for permission every time; this makes WebView2 auto-approve it,
            // matching what a user would always click "許可" to anyway.
            let window_config = app
                .config()
                .app
                .windows
                .iter()
                .find(|w| w.label == "main")
                .expect("main window must be declared in tauri.conf.json")
                .clone();
            let window = WebviewWindowBuilder::from_config(app.handle(), &window_config)?
                .enable_clipboard_access()
                .build()
                .expect("failed to build main window");

            #[cfg(target_os = "windows")]
            {
                // Mica/Acrylic require the window (and the page background) to be
                // transparent; see PLAN.md's Fluent/Mica design notes. Reads the same
                // persisted settings.json the frontend writes to (see Settings' 外観
                // section) so the right material is already showing before the
                // frontend's own init logic gets a chance to (re-)apply it — that later
                // call is also what corrects the `dark` guess below for "system" theme
                // mode, since only the frontend can resolve `prefers-color-scheme` this
                // early.
                let material = app
                    .store("settings.json")
                    .ok()
                    .and_then(|store| store.get("windowMaterial"))
                    .and_then(|value| value.as_str().map(str::to_string))
                    .unwrap_or_else(|| "mica".to_string());
                let dark = app
                    .store("settings.json")
                    .ok()
                    .and_then(|store| store.get("themeMode"))
                    .and_then(|value| value.as_str().map(str::to_string))
                    .map(|mode| mode == "dark")
                    .unwrap_or(false);

                if let Err(e) = appearance::apply_window_material(&window, &material, dark) {
                    eprintln!("[dockl] apply_window_material failed: {e}");
                }
            }

            window.show().expect("failed to show main window");

            tray::build_tray(&app.handle())?;

            let app_handle = app.handle().clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let tray_enabled = app_handle
                        .store("settings.json")
                        .ok()
                        .and_then(|store| store.get("trayEnabled"))
                        .and_then(|value| value.as_bool())
                        .unwrap_or(false);

                    if tray_enabled {
                        api.prevent_close();
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
