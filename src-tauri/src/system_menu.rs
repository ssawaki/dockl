//! The window's real Win32 system menu.
//!
//! `decorations: false` hides the frame, not the menu behind it: tao gives every
//! top-level window `WS_SYSMENU` unconditionally and only strips caption styles for
//! *child* windows (`to_window_styles`, tao's `platform_impl/windows/window_state.rs`).
//! So the menu has been there all along with nothing left to open it, and this is that
//! missing route rather than a lookalike — Move and Size included.

use tauri::WebviewWindow;

use crate::error::AppError;

/// Command id for the appended About entry. Below 0xF000, which Windows reserves for
/// `SC_*`.
#[cfg(windows)]
const IDM_ABOUT: u32 = 0x1000;

/// Pops the system menu at a point in the webview's own (logical) coordinates, and
/// reports whether About was chosen. Everything else is dispatched here as a
/// `WM_SYSCOMMAND`.
#[tauri::command]
pub async fn show_system_menu(
    window: WebviewWindow,
    x: f64,
    y: f64,
    about_label: String,
) -> Result<bool, AppError> {
    #[cfg(windows)]
    {
        // TrackPopupMenu is modal and has to run on the thread that owns the window, so
        // the work goes to the main thread and this one blocks on the answer.
        let (tx, rx) = std::sync::mpsc::channel();
        let win = window.clone();
        window
            .run_on_main_thread(move || {
                let _ = tx.send(win32::show(&win, x, y, &about_label));
            })
            .map_err(|e| AppError::CommandFailed(e.to_string()))?;

        rx.recv()
            .map_err(|e| AppError::CommandFailed(e.to_string()))?
    }

    #[cfg(not(windows))]
    {
        let _ = (window, x, y, about_label);
        Ok(false)
    }
}

#[cfg(windows)]
mod win32 {
    use super::{AppError, IDM_ABOUT};
    use tauri::WebviewWindow;
    use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
    use windows::Win32::Graphics::Gdi::ClientToScreen;
    use windows::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, EnableMenuItem, GetMenuItemCount, GetMenuItemID, GetSystemMenu, HMENU,
        MF_BYCOMMAND, MF_ENABLED, MF_GRAYED, MF_SEPARATOR, MF_STRING, PostMessageW, SC_MAXIMIZE,
        SC_MINIMIZE, SC_MOVE, SC_RESTORE, SC_SIZE, SetForegroundWindow, TPM_LEFTALIGN,
        TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, WM_NULL, WM_SYSCOMMAND,
    };
    use windows::core::PCWSTR;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn show(
        window: &WebviewWindow,
        x: f64,
        y: f64,
        about_label: &str,
    ) -> Result<bool, AppError> {
        let hwnd = HWND(
            window
                .hwnd()
                .map_err(|e| AppError::CommandFailed(e.to_string()))?
                .0 as _,
        );

        // Logical (CSS) pixels from the webview, which is what getBoundingClientRect gives
        // the frontend. ClientToScreen wants physical ones relative to the client area.
        let scale = window
            .scale_factor()
            .map_err(|e| AppError::CommandFailed(e.to_string()))?;
        let mut point = POINT {
            x: (x * scale).round() as i32,
            y: (y * scale).round() as i32,
        };

        unsafe {
            // `false` keeps whatever menu is already there; passing true would reset it and
            // throw away the About entry along with any state Windows has set.
            let menu: HMENU = GetSystemMenu(hwnd, false);

            // Appended once and left in place: the menu handle lives as long as the
            // window, so without the check a second open would stack another copy.
            if !has_about(menu) {
                // Bound to a local rather than passed inline: AppendMenuW copies the
                // string, but only while it's still there to copy, and a temporary would
                // already have been dropped by the time the call ran.
                let label = wide(about_label);
                let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
                let _ = AppendMenuW(menu, MF_STRING, IDM_ABOUT as usize, PCWSTR(label.as_ptr()));
            }

            set_item_states(window, menu);

            let _ = ClientToScreen(hwnd, &mut point);

            // Documented requirement for a popup menu on a window that isn't foreground:
            // without this pair it won't dismiss when the user clicks away.
            let _ = SetForegroundWindow(hwnd);

            let picked = TrackPopupMenu(
                menu,
                TPM_RETURNCMD | TPM_LEFTALIGN | TPM_RIGHTBUTTON,
                point.x,
                point.y,
                Some(0),
                hwnd,
                None,
            );

            let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));

            let cmd = picked.0 as u32;
            if cmd == 0 {
                return Ok(false); // dismissed without choosing anything
            }
            if cmd == IDM_ABOUT {
                return Ok(true);
            }

            // Posted rather than sent: TrackPopupMenu hasn't finished unwinding yet, and
            // SC_MOVE/SC_SIZE start their own modal loop the moment they're handled.
            let _ = PostMessageW(Some(hwnd), WM_SYSCOMMAND, WPARAM(cmd as usize), LPARAM(0));
            Ok(false)
        }
    }

    /// Greys out the entries that don't apply to the window's current state.
    ///
    /// Windows does this itself when the menu is reached the usual way, but that happens
    /// inside its own menu-tracking loop — calling `TrackPopupMenu` directly walks past
    /// it, and `WM_INITMENU` doesn't stand in for it (tried; the items stayed enabled
    /// while maximized). So the rules are spelled out here instead. They're the ones a
    /// real title bar follows: nothing to restore unless the window is maximized or
    /// minimized, and no moving or resizing a window that isn't in its normal state.
    fn set_item_states(window: &WebviewWindow, menu: HMENU) {
        let maximized = window.is_maximized().unwrap_or(false);
        let minimized = window.is_minimized().unwrap_or(false);
        let resizable = window.is_resizable().unwrap_or(true);
        let normal = !maximized && !minimized;

        for (id, enabled) in [
            (SC_RESTORE, maximized || minimized),
            (SC_MOVE, normal),
            (SC_SIZE, normal && resizable),
            (SC_MINIMIZE, !minimized),
            (SC_MAXIMIZE, !maximized && resizable),
        ] {
            let flag = if enabled { MF_ENABLED } else { MF_GRAYED };
            // Returns the previous state, or -1 when the item doesn't exist. Neither is
            // worth acting on: an id missing from this window's menu is one it never had.
            unsafe {
                let _ = EnableMenuItem(menu, id, MF_BYCOMMAND | flag);
            }
        }
    }

    /// Whether the About entry has already been appended to this window's system menu.
    ///
    /// It only ever goes on the end, so the last item is the only one worth looking at.
    fn has_about(menu: HMENU) -> bool {
        unsafe {
            let count = GetMenuItemCount(Some(menu));
            count > 0 && GetMenuItemID(menu, count - 1) == IDM_ABOUT
        }
    }
}
