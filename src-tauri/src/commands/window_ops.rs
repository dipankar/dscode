use tauri::{AppHandle, Manager, Window};

/// Hide window instead of closing it - enables instant "reopening"
#[tauri::command]
pub fn minimize_to_tray(window: Window) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())?;
    println!("[WindowPersistence] Window hidden");
    Ok(())
}

/// Show hidden window - instant since process is already running
#[tauri::command]
pub fn restore_from_tray(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        println!("[WindowPersistence] Window restored");
    }
    Ok(())
}

/// Check if window is visible
#[tauri::command]
pub fn is_window_visible(window: Window) -> Result<bool, String> {
    window.is_visible().map_err(|e| e.to_string())
}
