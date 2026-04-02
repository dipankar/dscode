use crate::session::{ExtensionInfo, SessionManager, SessionState};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Initialize the session
#[tauri::command]
pub async fn initialize_session(
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<(), String> {
    let session = session.read().await;
    session.initialize().await
}

/// Get current session state
#[tauri::command]
pub async fn get_session_state(
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<SessionState, String> {
    let session = session.read().await;
    Ok(session.get_state().await)
}

/// Get current session lifecycle
#[tauri::command]
pub async fn get_session_lifecycle(
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<String, String> {
    let session = session.read().await;
    let lifecycle = session.lifecycle().await;
    Ok(format!("{:?}", lifecycle))
}

/// Get list of installed extensions
#[tauri::command]
pub async fn get_installed_extensions(
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<Vec<ExtensionInfo>, String> {
    let session = session.read().await;
    Ok(session.get_installed_extensions().await)
}

/// Get list of active extensions
#[tauri::command]
pub async fn get_active_extensions(
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<Vec<ExtensionInfo>, String> {
    let session = session.read().await;
    Ok(session.get_active_extensions().await)
}

/// Load/activate an extension
#[tauri::command]
pub async fn session_load_extension(
    extension_id: String, session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<(), String> {
    let session = session.read().await;
    session.load_extension(&extension_id).await
}

/// Unload/deactivate an extension
#[tauri::command]
pub async fn session_unload_extension(
    extension_id: String, session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<(), String> {
    let session = session.read().await;
    session.unload_extension(&extension_id).await
}

/// Delete an extension
#[tauri::command]
pub async fn session_delete_extension(
    extension_id: String, session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<(), String> {
    let session = session.read().await;
    session.delete_extension(&extension_id).await
}

/// Add a workspace folder
#[tauri::command]
pub async fn add_workspace_folder(
    path: String, session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<(), String> {
    let session = session.read().await;
    session.add_workspace_folder(std::path::PathBuf::from(path)).await
}

/// Remove a workspace folder
#[tauri::command]
pub async fn remove_workspace_folder(
    path: String, session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<(), String> {
    let session = session.read().await;
    session.remove_workspace_folder(&std::path::PathBuf::from(path)).await
}

/// Notify selection change from editor
#[tauri::command]
pub async fn editor_selection_changed(
    session: State<'_, Arc<RwLock<SessionManager>>>, uri: String, selection: serde_json::Value,
    selections: serde_json::Value,
) -> Result<(), String> {
    let session = session.read().await;
    session.notify_editor_selection(&uri, selection, selections).await
}

/// Notify visible ranges change from editor
#[tauri::command]
pub async fn editor_visible_ranges_changed(
    session: State<'_, Arc<RwLock<SessionManager>>>, uri: String, ranges: serde_json::Value,
) -> Result<(), String> {
    let session = session.read().await;
    session.notify_editor_visible_ranges(&uri, ranges).await
}

#[tauri::command]
pub async fn reconnect_extension_host(
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<(), String> {
    let session = session.read().await;
    session.reconnect_extension_host().await
}
