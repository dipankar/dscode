use crate::commands::{Keybinding, KeybindingRegistry};
use tauri::State;

/// Get all registered keybindings
#[tauri::command]
pub async fn get_all_keybindings(
    registry: State<'_, KeybindingRegistry>,
) -> Result<Vec<Keybinding>, String> {
    Ok(registry.get_all_keybindings())
}

/// Get keybindings for a specific key combination
#[tauri::command]
pub async fn get_keybindings_for_key(
    key: String, registry: State<'_, KeybindingRegistry>,
) -> Result<Vec<Keybinding>, String> {
    Ok(registry.get_keybindings_for_key(&key))
}

/// Get keybindings for a specific command
#[tauri::command]
pub async fn get_keybindings_for_command(
    command: String, registry: State<'_, KeybindingRegistry>,
) -> Result<Vec<Keybinding>, String> {
    Ok(registry.get_keybindings_for_command(&command))
}

/// Register a keybinding (called from extension host)
#[tauri::command]
pub async fn register_keybinding(
    keybinding: Keybinding, registry: State<'_, KeybindingRegistry>,
) -> Result<(), String> {
    registry.register_keybinding(keybinding)
}

/// Register multiple keybindings
#[tauri::command]
pub async fn register_keybindings(
    keybindings: Vec<Keybinding>, registry: State<'_, KeybindingRegistry>,
) -> Result<(), String> {
    registry.register_keybindings(keybindings)
}

/// Get current platform (windows, macos, linux)
#[tauri::command]
pub async fn get_platform(registry: State<'_, KeybindingRegistry>) -> Result<String, String> {
    Ok(registry.get_platform())
}
