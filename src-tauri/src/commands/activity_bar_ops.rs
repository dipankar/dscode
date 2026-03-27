use crate::commands::{ActivityBarItem, ActivityBarRegistry};
use tauri::State;

/// Register a new activity bar item
#[tauri::command]
pub async fn register_activity_bar_item(
    owner: String, id: String, title: String, icon: Option<String>, icon_path: Option<String>,
    priority: Option<i32>, registry: State<'_, ActivityBarRegistry>,
) -> Result<String, String> {
    registry.register_item(owner, id, title, icon, icon_path, priority)
}

/// Update an activity bar item's badge
#[tauri::command]
pub async fn update_activity_bar_badge(
    key: String, badge_count: Option<i32>, badge_text: Option<String>,
    registry: State<'_, ActivityBarRegistry>,
) -> Result<(), String> {
    registry.update_badge(key, badge_count, badge_text)
}

/// Show an activity bar item
#[tauri::command]
pub async fn show_activity_bar_item(
    key: String, registry: State<'_, ActivityBarRegistry>,
) -> Result<(), String> {
    registry.show_item(key)
}

/// Hide an activity bar item
#[tauri::command]
pub async fn hide_activity_bar_item(
    key: String, registry: State<'_, ActivityBarRegistry>,
) -> Result<(), String> {
    registry.hide_item(key)
}

/// Dispose (remove) an activity bar item
#[tauri::command]
pub async fn dispose_activity_bar_item(
    key: String, registry: State<'_, ActivityBarRegistry>,
) -> Result<(), String> {
    registry.dispose_item(key)
}

/// Get all visible activity bar items
#[tauri::command]
pub async fn get_activity_bar_items(
    registry: State<'_, ActivityBarRegistry>,
) -> Result<Vec<ActivityBarItem>, String> {
    Ok(registry.get_visible_items())
}

/// Clear all activity bar items from a specific owner
#[tauri::command]
pub async fn clear_activity_bar_items(
    owner: String, registry: State<'_, ActivityBarRegistry>,
) -> Result<(), String> {
    registry.clear_owner_items(&owner)
}
