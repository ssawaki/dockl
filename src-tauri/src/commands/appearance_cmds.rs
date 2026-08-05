use tauri::{AppHandle, Manager};

use crate::appearance::apply_window_material;
use crate::error::AppError;

/// Switches the window's background material at runtime (Settings' 外観 section) — the
/// counterpart to the material applied once at startup in `lib.rs`'s `setup()`.
#[tauri::command]
pub async fn set_window_material(app: AppHandle, material: String, dark: bool) -> Result<(), AppError> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::CommandFailed("main window not found".into()))?;

    apply_window_material(&window, &material, dark).map_err(AppError::CommandFailed)
}
