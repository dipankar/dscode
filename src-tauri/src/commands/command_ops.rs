use tauri::State;
use crate::commands::{CommandRegistry, CommandInfo};

/// Get all registered commands
#[tauri::command]
pub async fn get_all_commands(
    registry: State<'_, CommandRegistry>,
) -> Result<Vec<CommandInfo>, String> {
    Ok(registry.get_all_commands())
}

/// Search commands by query
#[tauri::command]
pub async fn search_commands(
    query: String,
    registry: State<'_, CommandRegistry>,
) -> Result<Vec<CommandInfo>, String> {
    if query.trim().is_empty() {
        Ok(registry.get_all_commands())
    } else {
        Ok(registry.search_commands(&query))
    }
}

/// Get a specific command by ID
#[tauri::command]
pub async fn get_command(
    command_id: String,
    registry: State<'_, CommandRegistry>,
) -> Result<Option<CommandInfo>, String> {
    Ok(registry.get_command(&command_id))
}

/// Register a command (called from extension host via IPC)
#[tauri::command]
pub async fn register_command(
    command: CommandInfo,
    registry: State<'_, CommandRegistry>,
) -> Result<(), String> {
    registry.register_command(command)
}

/// Unregister a command (called from extension host via IPC)
#[tauri::command]
pub async fn unregister_command(
    command_id: String,
    owner: String,
    registry: State<'_, CommandRegistry>,
) -> Result<(), String> {
    registry.unregister_command(&command_id, &owner)
}
