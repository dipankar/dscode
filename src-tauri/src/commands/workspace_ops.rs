use crate::commands::{
    FileDecoration, FileDecorationProvider, FindFilesOptions, TextSearchOptions, TextSearchResult,
    WorkspaceConfiguration, WorkspaceFolder,
};
use crate::session::SessionManager;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Get all workspace folders
#[tauri::command]
pub async fn get_workspace_folders(
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<Vec<WorkspaceFolder>, String> {
    let session = session.read().await;
    Ok(session.get_workspace_folders_detailed().await)
}

/// Add a workspace folder
#[tauri::command]
pub async fn workspace_add_folder(
    folder: WorkspaceFolder,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<(), String> {
    let session = session.read().await;
    session.add_workspace_folder_from_uri(&folder.uri).await
}

/// Remove a workspace folder
#[tauri::command]
pub async fn workspace_remove_folder(
    uri: String,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<(), String> {
    let session = session.read().await;
    session.remove_workspace_folder_by_uri(&uri).await
}

/// Get workspace configuration
#[tauri::command]
pub async fn get_workspace_configuration(
    section: String,
    scope: Option<String>,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<Option<WorkspaceConfiguration>, String> {
    let session = session.read().await;
    Ok(session
        .get_workspace_configuration(&section, scope.as_deref())
        .await)
}

/// Update workspace configuration
#[tauri::command]
pub async fn update_workspace_configuration(
    section: String,
    scope: Option<String>,
    key: String,
    value: serde_json::Value,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<(), String> {
    let session = session.read().await;
    session
        .update_workspace_configuration(section, scope, key, value)
        .await
}

/// Register a file decoration provider
#[tauri::command]
pub async fn register_file_decoration_provider(
    provider: FileDecorationProvider,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<String, String> {
    let session = session.read().await;
    session.register_file_decoration_provider(provider).await
}

/// Update file decorations
#[tauri::command]
pub async fn update_file_decorations(
    provider_id: String,
    decorations: Vec<FileDecoration>,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<(), String> {
    let session = session.read().await;
    session
        .update_file_decorations(provider_id, decorations)
        .await
}

/// Get file decorations for a URI
#[tauri::command]
pub async fn get_file_decorations(
    uri: String,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<Vec<FileDecoration>, String> {
    let session = session.read().await;
    Ok(session.get_file_decorations(&uri).await)
}

/// Find files in workspace
#[tauri::command]
pub async fn workspace_find_files(
    options: FindFilesOptions,
    _folders: Option<Vec<WorkspaceFolder>>,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<Vec<String>, String> {
    let session = session.read().await;
    session.find_workspace_files_with_options(&options).await
}

/// Search for text in workspace files
#[tauri::command]
pub async fn workspace_find_text(
    options: TextSearchOptions,
    _folders: Option<Vec<WorkspaceFolder>>,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<Vec<TextSearchResult>, String> {
    let session = session.read().await;
    session.find_workspace_text(&options).await
}

/// Clear workspace data for an owner
#[tauri::command]
pub async fn clear_workspace_data(
    owner: String,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<(), String> {
    let session = session.read().await;
    session.clear_workspace_data(&owner).await
}
