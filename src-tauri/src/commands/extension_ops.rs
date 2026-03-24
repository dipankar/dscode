use crate::config::AppDirectories;
use crate::marketplace;
use crate::session::{ExtensionContribution, InstalledExtension, SessionManager};
use serde_json::Value;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

// Note: Extension host is started by SessionManager during app initialization.
// This command remains for compatibility with existing frontend calls.
#[tauri::command]
pub async fn start_extension_host() -> Result<(), String> {
    println!(
        "[ExtensionHost] start_extension_host called - extension host is managed by SessionManager"
    );
    Ok(())
}

#[tauri::command]
pub fn get_extensions_dir(app_dirs: State<'_, AppDirectories>) -> Result<String, String> {
    Ok(app_dirs
        .extensions_dir
        .to_str()
        .ok_or("Failed to convert extensions directory path to UTF-8 string")?
        .to_string())
}

#[tauri::command]
pub async fn install_extension(
    vsix_path: String,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<InstalledExtension, String> {
    let session = session.read().await;
    session.install_vsix_package(vsix_path).await
}

#[tauri::command]
pub async fn uninstall_extension(
    session: State<'_, Arc<RwLock<SessionManager>>>,
    extension_id: String,
) -> Result<(), String> {
    let session = session.read().await;
    session.delete_extension(&extension_id).await
}

#[tauri::command]
pub async fn list_extensions(
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<Vec<InstalledExtension>, String> {
    let session = session.read().await;
    session.list_extensions_detailed().await
}

#[tauri::command]
pub async fn get_extension_contributions(
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<Vec<ExtensionContribution>, String> {
    let session = session.read().await;
    session.get_extension_contributions_detailed().await
}

#[tauri::command]
pub async fn search_marketplace(
    query: String,
    page: Option<i32>,
    page_size: Option<i32>,
) -> Result<Vec<marketplace::MarketplaceExtension>, String> {
    marketplace::search_marketplace(query, page.unwrap_or(1), page_size.unwrap_or(50)).await
}

#[tauri::command]
pub async fn get_marketplace_extension(
    publisher: String,
    name: String,
) -> Result<marketplace::MarketplaceExtension, String> {
    marketplace::get_extension_details(publisher, name).await
}

#[tauri::command]
pub async fn install_from_marketplace(
    session: State<'_, Arc<RwLock<SessionManager>>>,
    publisher: String,
    name: String,
    version: String,
) -> Result<InstalledExtension, String> {
    let session = session.read().await;
    session
        .install_marketplace_extension(publisher, name, version)
        .await
}

#[tauri::command]
pub async fn extension_tree_get_children(
    session: State<'_, Arc<RwLock<SessionManager>>>,
    view_id: String,
    element: Option<Value>,
) -> Result<Vec<Value>, String> {
    let session = session.read().await;
    session.get_extension_tree_children(view_id, element).await
}

#[tauri::command]
pub async fn extension_tree_get_item(
    session: State<'_, Arc<RwLock<SessionManager>>>,
    view_id: String,
    element: Value,
) -> Result<Value, String> {
    let session = session.read().await;
    session.get_extension_tree_item(view_id, element).await
}

#[tauri::command]
pub async fn extension_execute_command(
    session: State<'_, Arc<RwLock<SessionManager>>>,
    command: String,
    args: Vec<Value>,
) -> Result<Value, String> {
    let session = session.read().await;
    session.execute_extension_command(command, args).await
}

#[tauri::command]
pub async fn extension_tree_notify_event(
    session: State<'_, Arc<RwLock<SessionManager>>>,
    view_id: String,
    event: String,
    element: Option<Value>,
    selection: Option<Vec<Value>>,
    visible: Option<bool>,
) -> Result<(), String> {
    let session = session.read().await;
    session
        .notify_extension_tree_event(view_id, event, element, selection, visible)
        .await
}
