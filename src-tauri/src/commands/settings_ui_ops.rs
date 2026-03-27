use super::configuration_registry::ConfigurationScope;
use super::settings_ui_registry::*;
use serde_json::Value;
use tauri::State;

// ===== Category Management =====

/// Register settings category
#[tauri::command]
pub async fn register_settings_category(
    category: SettingsCategory, registry: State<'_, SettingsUIRegistry>,
) -> Result<String, String> {
    registry.register_category(category).await
}

/// Get all settings categories
#[tauri::command]
pub async fn get_settings_categories(
    registry: State<'_, SettingsUIRegistry>,
) -> Result<Vec<SettingsCategory>, String> {
    Ok(registry.get_categories().await)
}

/// Get settings category by ID
#[tauri::command]
pub async fn get_settings_category(
    category_id: String, registry: State<'_, SettingsUIRegistry>,
) -> Result<SettingsCategory, String> {
    registry.get_category(&category_id).await
}

// ===== UI Schema Management =====

/// Register UI schema for a setting
#[tauri::command]
pub async fn register_setting_ui_schema(
    key: String, schema: SettingUISchema, registry: State<'_, SettingsUIRegistry>,
) -> Result<(), String> {
    registry.register_ui_schema(key, schema).await
}

/// Get UI schema for a setting
#[tauri::command]
pub async fn get_setting_ui_schema(
    key: String, registry: State<'_, SettingsUIRegistry>,
) -> Result<Option<SettingUISchema>, String> {
    Ok(registry.get_ui_schema(&key).await)
}

/// Get all UI schemas
#[tauri::command]
pub async fn get_all_setting_ui_schemas(
    registry: State<'_, SettingsUIRegistry>,
) -> Result<std::collections::HashMap<String, SettingUISchema>, String> {
    Ok(registry.get_all_ui_schemas().await)
}

// ===== Settings Search =====

/// Search settings by query
#[tauri::command]
pub async fn search_settings(
    query: String, registry: State<'_, SettingsUIRegistry>,
) -> Result<Vec<SettingSearchResult>, String> {
    Ok(registry.search_settings(&query).await)
}

/// Get settings by category
#[tauri::command]
pub async fn get_settings_by_category(
    category_id: String, registry: State<'_, SettingsUIRegistry>,
) -> Result<Vec<SettingWithValue>, String> {
    Ok(registry.get_settings_by_category(&category_id).await)
}

// ===== Settings Value Management =====

/// Get setting with all scope values
#[tauri::command]
pub async fn get_setting_with_values(
    key: String, registry: State<'_, SettingsUIRegistry>,
) -> Result<SettingWithValue, String> {
    registry.get_setting_with_values(&key).await
}

/// Get effective setting value
#[tauri::command]
pub async fn get_effective_setting_value(
    key: String, scope: ConfigurationScope, registry: State<'_, SettingsUIRegistry>,
) -> Result<Value, String> {
    registry.get_effective_value(&key, scope).await
}

/// Update setting value
#[tauri::command]
pub async fn update_setting_value(
    key: String, value: Value, scope: ConfigurationScope, registry: State<'_, SettingsUIRegistry>,
) -> Result<(), String> {
    // Validate before updating
    registry.validate_setting(&key, &value).await?;
    registry.update_setting_value(&key, value, scope).await
}

/// Reset setting to default
#[tauri::command]
pub async fn reset_setting_to_default(
    key: String, scope: ConfigurationScope, registry: State<'_, SettingsUIRegistry>,
) -> Result<(), String> {
    registry.reset_setting_to_default(&key, scope).await
}

// ===== Validation =====

/// Validate setting value
#[tauri::command]
pub async fn validate_setting_value(
    key: String, value: Value, registry: State<'_, SettingsUIRegistry>,
) -> Result<(), String> {
    registry.validate_setting(&key, &value).await
}

// ===== Cleanup =====

/// Clear settings UI data for owner
#[tauri::command]
pub async fn clear_settings_ui_data(
    owner: String, registry: State<'_, SettingsUIRegistry>,
) -> Result<(), String> {
    registry.clear_settings_ui_data(&owner).await;
    Ok(())
}
