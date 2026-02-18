use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::error::AppError;

const WIDGET_WIDTH: f64 = 320.0;
const WIDGET_HEIGHT: f64 = 72.0;
const EDGE_PADDING: f64 = 20.0;
const MENU_BAR_HEIGHT: f64 = 30.0;
const DOCK_HEIGHT: f64 = 80.0;

/// Show the floating recording widget at the given screen position.
#[tauri::command]
pub async fn show_floating_widget(
    app: AppHandle,
    position: Option<String>,
) -> Result<(), AppError> {
    let (tx, rx) = std::sync::mpsc::channel();
    let app_for_main = app.clone();
    app.run_on_main_thread(move || {
        let result = show_floating_widget_impl(app_for_main, position);
        let _ = tx.send(result);
    })
    .map_err(|e| AppError::Audio(format!("Failed to schedule floating widget show on main thread: {e}")))?;

    rx.recv()
        .map_err(|e| AppError::Audio(format!("Failed to receive floating widget show result: {e}")))?
}

fn show_floating_widget_impl(app: AppHandle, position: Option<String>) -> Result<(), AppError> {
    // If already open, just bring it to front
    if let Some(window) = app.get_webview_window("floating") {
        window
            .show()
            .map_err(|e| AppError::Audio(format!("Failed to show floating widget: {e}")))?;
        return Ok(());
    }

    let mut builder =
        WebviewWindowBuilder::new(&app, "floating", WebviewUrl::App("index.html".into()))
            .title("")
            .decorations(false)
            .always_on_top(true)
            .focused(false)
            .resizable(false)
            .inner_size(WIDGET_WIDTH, WIDGET_HEIGHT)
            .skip_taskbar(true);

    // Try to position based on the given position string;
    // fall back to center() if monitor info is unavailable.
    let pos = position.as_deref().unwrap_or("bottom-right");
    if let Some(monitor) = app.primary_monitor().ok().flatten() {
        let screen = monitor.size();
        let scale = monitor.scale_factor();
        let sw = screen.width as f64 / scale;
        let sh = screen.height as f64 / scale;

        let (x, y) = match pos {
            "top-left" => (EDGE_PADDING, MENU_BAR_HEIGHT + EDGE_PADDING),
            "top-right" => (sw - WIDGET_WIDTH - EDGE_PADDING, MENU_BAR_HEIGHT + EDGE_PADDING),
            "bottom-left" => (EDGE_PADDING, sh - WIDGET_HEIGHT - DOCK_HEIGHT - EDGE_PADDING),
            // "bottom-right" and fallback
            _ => (
                sw - WIDGET_WIDTH - EDGE_PADDING,
                sh - WIDGET_HEIGHT - DOCK_HEIGHT - EDGE_PADDING,
            ),
        };

        builder = builder.position(x, y);
    } else {
        builder = builder.center();
    }

    let window = builder
        .build()
        .map_err(|e| AppError::Audio(format!("Failed to create floating widget: {e}")))?;

    // Make the window background transparent on macOS
    #[cfg(target_os = "macos")]
    #[allow(deprecated)]
    {
        use cocoa::appkit::{NSColor, NSWindow};
        use cocoa::base::nil;

        if let Ok(ns_window) = window.ns_window() {
            unsafe {
                let ns_win = ns_window as cocoa::base::id;
                let clear = NSColor::clearColor(nil);
                ns_win.setOpaque_(false);
                ns_win.setHasShadow_(false);
                ns_win.setBackgroundColor_(clear);
            }
        }
    }

    Ok(())
}

/// Hide the floating recording widget.
#[tauri::command]
pub async fn hide_floating_widget(app: AppHandle) -> Result<(), AppError> {
    let (tx, rx) = std::sync::mpsc::channel();
    let app_for_main = app.clone();
    app.run_on_main_thread(move || {
        let result = hide_floating_widget_impl(app_for_main);
        let _ = tx.send(result);
    })
    .map_err(|e| AppError::Audio(format!("Failed to schedule floating widget hide on main thread: {e}")))?;

    rx.recv()
        .map_err(|e| AppError::Audio(format!("Failed to receive floating widget hide result: {e}")))?
}

fn hide_floating_widget_impl(app: AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("floating") {
        window
            .close()
            .map_err(|e| AppError::Audio(format!("Failed to close floating widget: {e}")))?;
    }
    Ok(())
}
