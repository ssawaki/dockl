mod commands;
mod compose;
mod docker_bridge;
mod error;
mod pty_session;
mod state;
mod tray;
mod wsl;

use tauri::Manager;
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
            commands::list_containers,
            commands::container_action,
            commands::inspect_container,
            commands::setup_list_distros,
            commands::setup_connect,
            commands::setup_current_distro,
            commands::compose_action,
            commands::stream_logs,
            commands::stop_log_stream,
            commands::start_attach_session,
            commands::start_wsl_shell_session,
            commands::pty_write,
            commands::pty_resize,
            commands::pty_close,
        ])
        .setup(|app| {
            let window = app
                .get_webview_window("main")
                .expect("main window must exist");

            #[cfg(target_os = "windows")]
            {
                // Mica requires the window (and the page background) to be transparent;
                // see PLAN.md's Fluent/Mica design notes.
                if let Err(e) = window_vibrancy::apply_mica(&window, None) {
                    eprintln!("[dockl] apply_mica failed: {e}");
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
