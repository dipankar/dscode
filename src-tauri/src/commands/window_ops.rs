use std::sync::Arc;
use tokio::sync::RwLock;

use tauri::{AppHandle, Manager, Window, State};

use crate::session::SessionManager;

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

/// Respond to a pending window message prompt
#[tauri::command]
pub async fn window_message_action(
    session: State<'_, Arc<RwLock<SessionManager>>>,
    request_id: String,
    action: Option<String>,
) -> Result<(), String> {
    let session = session.read().await;
    session.resolve_window_message(&request_id, action).await
}

/// Resolve a pending quick pick request
#[tauri::command]
pub async fn window_quick_pick_select(
    session: State<'_, Arc<RwLock<SessionManager>>>,
    request_id: String,
    selection: Option<serde_json::Value>,
) -> Result<(), String> {
    let session = session.read().await;
    session.resolve_quick_pick(&request_id, selection).await
}

/// Resolve a pending input box request
#[tauri::command]
pub async fn window_input_box_submit(
    session: State<'_, Arc<RwLock<SessionManager>>>,
    request_id: String,
    value: Option<String>,
) -> Result<(), String> {
    let session = session.read().await;
    session.resolve_input_box(&request_id, value).await
}
