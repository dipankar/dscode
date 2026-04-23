use super::configuration_registry::*;
use serde_json::Value;
use std::path::PathBuf;
use tauri::State;

/// Set settings file path
#[tauri::command]
pub async fn set_settings_path(
    path: String, registry: State<'_, ConfigurationRegistry>,
) -> Result<(), String> {
    registry.set_settings_path(PathBuf::from(path)).map_err(|e| e.to_string())
}

/// Set workspace path
#[tauri::command]
pub async fn set_workspace_path(
    path: String, registry: State<'_, ConfigurationRegistry>,
) -> Result<(), String> {
    registry.set_workspace_path(PathBuf::from(path)).map_err(|e| e.to_string())
}

/// Register configuration schema
#[tauri::command]
pub async fn register_configuration_schema(
    contribution: ConfigurationContribution, registry: State<'_, ConfigurationRegistry>,
) -> Result<(), String> {
    registry.register_configuration_schema(contribution).map_err(|e| e.to_string())
}

/// Get configuration value
#[tauri::command]
pub async fn get_configuration_value(
    key: String, scope: ConfigurationScope, registry: State<'_, ConfigurationRegistry>,
) -> Result<Option<Value>, String> {
    registry.get_configuration(&key, scope).map_err(|e| e.to_string())
}

/// Get configuration with fallback
#[tauri::command]
pub async fn get_configuration_with_fallback(
    key: String, scope: ConfigurationScope, registry: State<'_, ConfigurationRegistry>,
) -> Result<Option<Value>, String> {
    registry.get_configuration_with_fallback(&key, scope).map_err(|e| e.to_string())
}

/// Update configuration value
#[tauri::command]
pub async fn update_configuration_value(
    key: String, value: Value, scope: ConfigurationScope,
    registry: State<'_, ConfigurationRegistry>,
) -> Result<(), String> {
    registry.update_configuration(key, value, scope).map_err(|e| e.to_string())
}

/// Get all configuration keys
#[tauri::command]
pub async fn get_all_configuration_keys(
    scope: ConfigurationScope, registry: State<'_, ConfigurationRegistry>,
) -> Result<Vec<String>, String> {
    Ok(registry.get_all_keys(scope))
}

/// Check if configuration exists
#[tauri::command]
pub async fn has_configuration_key(
    key: String, scope: ConfigurationScope, registry: State<'_, ConfigurationRegistry>,
) -> Result<bool, String> {
    Ok(registry.has_configuration(&key, scope))
}

/// Clear configuration data for owner
#[tauri::command]
pub async fn clear_configuration_data(
    owner: String, registry: State<'_, ConfigurationRegistry>,
) -> Result<(), String> {
    registry.clear_configuration_data(&owner);
    Ok(())
}
