use tauri::State;
use crate::commands::{StatusBarRegistry, StatusBarItem, StatusBarAlignment, StatusBarCommand};

/// Create a new status bar item
#[tauri::command]
pub async fn create_status_bar_item(
    owner: String,
    id: String,
    alignment: StatusBarAlignment,
    priority: Option<i32>,
    registry: State<'_, StatusBarRegistry>,
) -> Result<String, String> {
    registry.create_item(owner, id, alignment, priority)
}

/// Update a status bar item's properties
#[tauri::command]
pub async fn update_status_bar_item(
    key: String,
    text: Option<String>,
    tooltip: Option<String>,
    color: Option<String>,
    background_color: Option<String>,
    command: Option<StatusBarCommand>,
    registry: State<'_, StatusBarRegistry>,
) -> Result<(), String> {
    registry.update_item(key, text, tooltip, color, background_color, command)
}

/// Show a status bar item
#[tauri::command]
pub async fn show_status_bar_item(
    key: String,
    registry: State<'_, StatusBarRegistry>,
) -> Result<(), String> {
    registry.show_item(key)
}

/// Hide a status bar item
#[tauri::command]
pub async fn hide_status_bar_item(
    key: String,
    registry: State<'_, StatusBarRegistry>,
) -> Result<(), String> {
    registry.hide_item(key)
}

/// Dispose (remove) a status bar item
#[tauri::command]
pub async fn dispose_status_bar_item(
    key: String,
    registry: State<'_, StatusBarRegistry>,
) -> Result<(), String> {
    registry.dispose_item(key)
}

/// Get all visible status bar items
#[tauri::command]
pub async fn get_status_bar_items(
    registry: State<'_, StatusBarRegistry>,
) -> Result<Vec<StatusBarItem>, String> {
    Ok(registry.get_visible_items())
}

/// Clear all status bar items from a specific owner
#[tauri::command]
pub async fn clear_status_bar_items(
    owner: String,
    registry: State<'_, StatusBarRegistry>,
) -> Result<(), String> {
    registry.clear_owner_items(&owner)
}
