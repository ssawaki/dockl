use tauri::WebviewWindow;

/// Applies the requested Windows background material to the window, clearing whichever
/// effect (if any) was previously active first.
///
/// `dark` selects the light/dark variant for materials that need an explicit hint:
/// - Mica auto-follows the OS when `None` is passed, but we always pass the app's own
///   *resolved* theme (which may be a manual override, not just the OS setting) instead,
///   so Mica never disagrees with the rest of the UI.
/// - Acrylic has no such auto mode at all — it's just a tinted blur, so the tint color
///   itself is what makes it read as "light" or "dark".
pub fn apply_window_material(window: &WebviewWindow, material: &str, dark: bool) -> Result<(), String> {
    // Best-effort: clearing an effect that was never applied returns an error we don't
    // care about (there's nothing to undo), and Mica/Acrylic are mutually exclusive so
    // only one of these ever actually had something to clear.
    let _ = window_vibrancy::clear_mica(window);
    let _ = window_vibrancy::clear_acrylic(window);

    match material {
        "mica" => window_vibrancy::apply_mica(window, Some(dark)).map_err(|e| e.to_string()),
        "acrylic" => {
            // Roughly matches Windows' own Fluent acrylic defaults for light/dark surfaces.
            // Alpha kept low (not ~200/255) so the blurred desktop/windows behind actually
            // read through the tint instead of washing out to a near-solid color.
            let tint = if dark { (30, 30, 30, 125) } else { (243, 243, 243, 125) };
            window_vibrancy::apply_acrylic(window, Some(tint)).map_err(|e| e.to_string())
        }
        // "solid" (or anything else): already cleared above, nothing further to apply —
        // the frontend paints an opaque CSS background for this mode instead.
        _ => Ok(()),
    }
}
