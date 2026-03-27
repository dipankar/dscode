use crate::commands::{MenuContext, MenuItem, MenuRegistry};
use serde_json::Value;
use tauri::State;

/// Get menu items for a specific location
#[tauri::command]
pub async fn get_menu_items(
    location: String, registry: State<'_, MenuRegistry>,
) -> Result<Vec<MenuItem>, String> {
    Ok(registry.get_menu_items(&location))
}

/// Get menu items for a location with context filtering
#[tauri::command]
pub async fn get_menu_items_filtered(
    location: String, context: MenuContext, registry: State<'_, MenuRegistry>,
) -> Result<Vec<MenuItem>, String> {
    Ok(registry.get_menu_items_filtered(&location, &context))
}

/// Register a menu item (called from extension host)
#[tauri::command]
pub async fn register_menu_item(
    item: MenuItem, registry: State<'_, MenuRegistry>,
) -> Result<(), String> {
    registry.register_menu_item(item)
}

/// Register multiple menu items
#[tauri::command]
pub async fn register_menu_items(
    items: Vec<MenuItem>, registry: State<'_, MenuRegistry>,
) -> Result<(), String> {
    registry.register_menu_items(items)
}

/// Get all menu locations
#[tauri::command]
pub async fn get_menu_locations(registry: State<'_, MenuRegistry>) -> Result<Vec<String>, String> {
    Ok(registry.get_locations())
}
