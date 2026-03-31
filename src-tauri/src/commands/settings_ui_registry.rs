use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tracing::info;

use super::configuration_registry::{ConfigurationRegistry, ConfigurationScope};

/// Settings UI registry for enhanced settings management
pub struct SettingsUIRegistry {
    categories: Arc<RwLock<Vec<SettingsCategory>>>,
    ui_schemas: Arc<RwLock<HashMap<String, SettingUISchema>>>,
    configuration_registry: Arc<ConfigurationRegistry>,
    app_handle: AppHandle,
}

impl SettingsUIRegistry {
    pub fn new(app_handle: AppHandle, configuration_registry: Arc<ConfigurationRegistry>) -> Self {
        Self {
            categories: Arc::new(RwLock::new(Vec::new())),
            ui_schemas: Arc::new(RwLock::new(HashMap::new())),
            configuration_registry,
            app_handle,
        }
    }

    // ===== Category Management =====

    /// Register settings category
    pub async fn register_category(&self, category: SettingsCategory) -> Result<String, String> {
        let mut categories = self.categories.write().await;

        // Check if category already exists
        if categories.iter().any(|c| c.id == category.id) {
            return Err(format!("Category '{}' already registered", category.id));
        }

        let category_id = category.id.clone();
        categories.push(category);

        // Sort categories by order
        categories.sort_by_key(|c| c.order);

        info!("Registered category: {}", category_id);

        Ok(category_id)
    }

    /// Get all categories
    pub async fn get_categories(&self) -> Vec<SettingsCategory> {
        self.categories.read().await.clone()
    }

    /// Get category by ID
    pub async fn get_category(&self, category_id: &str) -> Result<SettingsCategory, String> {
        self.categories
            .read()
            .await
            .iter()
            .find(|c| c.id == category_id)
            .cloned()
            .ok_or_else(|| format!("Category '{}' not found", category_id))
    }

    // ===== UI Schema Management =====

    /// Register UI schema for a setting
    pub async fn register_ui_schema(
        &self, key: String, schema: SettingUISchema,
    ) -> Result<(), String> {
        let mut ui_schemas = self.ui_schemas.write().await;

        ui_schemas.insert(key.clone(), schema);

        info!("Registered UI schema: {}", key);

        Ok(())
    }

    /// Get UI schema for a setting
    pub async fn get_ui_schema(&self, key: &str) -> Option<SettingUISchema> {
        self.ui_schemas.read().await.get(key).cloned()
    }

    /// Get all UI schemas
    pub async fn get_all_ui_schemas(&self) -> HashMap<String, SettingUISchema> {
        self.ui_schemas.read().await.clone()
    }

    // ===== Settings Search =====

    /// Search settings by query
    pub async fn search_settings(&self, query: &str) -> Vec<SettingSearchResult> {
        let ui_schemas = self.ui_schemas.read().await;
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for (key, schema) in ui_schemas.iter() {
            let mut score = 0;

            // Check key match
            if key.to_lowercase().contains(&query_lower) {
                score += 100;
            }

            // Check title match
            if schema.title.to_lowercase().contains(&query_lower) {
                score += 50;
            }

            // Check description match
            if let Some(desc) = &schema.description {
                if desc.to_lowercase().contains(&query_lower) {
                    score += 25;
                }
            }

            // Check category match
            if schema.category.to_lowercase().contains(&query_lower) {
                score += 10;
            }

            // Check tags match
            for tag in &schema.tags {
                if tag.to_lowercase().contains(&query_lower) {
                    score += 5;
                }
            }

            if score > 0 {
                results.push(SettingSearchResult {
                    key: key.clone(),
                    schema: schema.clone(),
                    score,
                });
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.cmp(&a.score));

        results
    }

    /// Filter settings by category
    pub async fn get_settings_by_category(&self, category_id: &str) -> Vec<SettingWithValue> {
        let ui_schemas = self.ui_schemas.read().await;
        let mut settings = Vec::new();

        for (key, schema) in ui_schemas.iter() {
            if schema.category == category_id {
                // Get current value
                let user_value = self
                    .configuration_registry
                    .get_configuration(key, ConfigurationScope::User)
                    .ok()
                    .flatten();

                let workspace_value = self
                    .configuration_registry
                    .get_configuration(key, ConfigurationScope::Workspace)
                    .ok()
                    .flatten();

                settings.push(SettingWithValue {
                    key: key.clone(),
                    schema: schema.clone(),
                    user_value,
                    workspace_value,
                });
            }
        }

        // Sort by order
        settings.sort_by_key(|s| s.schema.order);

        settings
    }

    // ===== Settings Value Management =====

    /// Get setting with all scope values
    pub async fn get_setting_with_values(&self, key: &str) -> Result<SettingWithValue, String> {
        let schema =
            self.get_ui_schema(key).await.ok_or_else(|| format!("Setting '{}' not found", key))?;

        let user_value = self
            .configuration_registry
            .get_configuration(key, ConfigurationScope::User)
            .ok()
            .flatten();

        let workspace_value = self
            .configuration_registry
            .get_configuration(key, ConfigurationScope::Workspace)
            .ok()
            .flatten();

        Ok(SettingWithValue { key: key.to_string(), schema, user_value, workspace_value })
    }

    /// Get effective setting value (with scope resolution)
    pub async fn get_effective_value(
        &self, key: &str, scope: ConfigurationScope,
    ) -> Result<Value, String> {
        self.configuration_registry
            .get_configuration_with_fallback(key, scope)
            .map(|v| v.unwrap_or(Value::Null))
    }

    /// Update setting value
    pub async fn update_setting_value(
        &self, key: &str, value: Value, scope: ConfigurationScope,
    ) -> Result<(), String> {
        self.configuration_registry.update_configuration(key.to_string(), value, scope)
    }

    /// Reset setting to default
    pub async fn reset_setting_to_default(
        &self, key: &str, scope: ConfigurationScope,
    ) -> Result<(), String> {
        // Get schema to find default value
        let schema =
            self.get_ui_schema(key).await.ok_or_else(|| format!("Setting '{}' not found", key))?;

        self.configuration_registry.update_configuration(
            key.to_string(),
            schema.default.clone(),
            scope,
        )
    }

    // ===== Validation =====

    /// Validate setting value
    pub async fn validate_setting(&self, key: &str, value: &Value) -> Result<(), String> {
        let schema =
            self.get_ui_schema(key).await.ok_or_else(|| format!("Setting '{}' not found", key))?;

        // Type validation
        match &schema.ui_type {
            SettingUIType::String { pattern, .. } => {
                if !value.is_string() {
                    return Err("Value must be a string".to_string());
                }
                if let Some(pattern_str) = pattern {
                    let value_str = value.as_str().ok_or("Value is not a string")?;
                    let regex = regex::Regex::new(pattern_str)
                        .map_err(|e| format!("Invalid pattern: {}", e))?;
                    if !regex.is_match(value_str) {
                        return Err(format!("Value does not match pattern: {}", pattern_str));
                    }
                }
            }
            SettingUIType::Number { min, max, .. } => {
                if !value.is_number() {
                    return Err("Value must be a number".to_string());
                }
                let num = value.as_f64().ok_or("Value is not a valid number")?;
                if let Some(min_val) = min {
                    if num < *min_val {
                        return Err(format!("Value must be >= {}", min_val));
                    }
                }
                if let Some(max_val) = max {
                    if num > *max_val {
                        return Err(format!("Value must be <= {}", max_val));
                    }
                }
            }
            SettingUIType::Boolean => {
                if !value.is_boolean() {
                    return Err("Value must be a boolean".to_string());
                }
            }
            SettingUIType::Enum { values, .. } => {
                if !value.is_string() {
                    return Err("Value must be a string".to_string());
                }
                let value_str = value.as_str().ok_or("Value is not a string")?;
                if !values.iter().any(|v| v.value == value_str) {
                    return Err("Value must be one of the allowed values".to_string());
                }
            }
            SettingUIType::Array { item_type, .. } => {
                if !value.is_array() {
                    return Err("Value must be an array".to_string());
                }
                // Validate each item
                if let Some(arr) = value.as_array() {
                    for item in arr {
                        // Basic type check for array items
                        match item_type.as_str() {
                            "string" => {
                                if !item.is_string() {
                                    return Err("Array items must be strings".to_string());
                                }
                            }
                            "number" => {
                                if !item.is_number() {
                                    return Err("Array items must be numbers".to_string());
                                }
                            }
                            "boolean" => {
                                if !item.is_boolean() {
                                    return Err("Array items must be booleans".to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            SettingUIType::Object => {
                if !value.is_object() {
                    return Err("Value must be an object".to_string());
                }
            }
        }

        Ok(())
    }

    // ===== Cleanup =====

    /// Clear settings UI data for owner
    pub async fn clear_settings_ui_data(&self, owner: &str) {
        let mut ui_schemas = self.ui_schemas.write().await;
        ui_schemas.retain(|key, _| !key.starts_with(&format!("{}.", owner)));

        let mut categories = self.categories.write().await;
        categories.retain(|c| c.owner != owner);

        info!("Cleared settings UI data for owner: {}", owner);
    }
}

// ===== Type Definitions =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsCategory {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub order: i32,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingUISchema {
    pub title: String,
    pub description: Option<String>,
    pub ui_type: SettingUIType,
    pub default: Value,
    pub category: String,
    pub order: i32,
    pub tags: Vec<String>,
    pub scope: SettingScope,
    pub deprecated: bool,
    pub deprecation_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SettingUIType {
    String {
        #[serde(skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        min_length: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_length: Option<usize>,
    },
    Number {
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        step: Option<f64>,
    },
    Boolean,
    Enum {
        values: Vec<EnumValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        allow_custom: Option<bool>,
    },
    Array {
        item_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        min_items: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_items: Option<usize>,
    },
    Object,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumValue {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettingScope {
    Application,
    Window,
    Resource,
    Language,
    Machine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingWithValue {
    pub key: String,
    pub schema: SettingUISchema,
    pub user_value: Option<Value>,
    pub workspace_value: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingSearchResult {
    pub key: String,
    pub schema: SettingUISchema,
    pub score: i32,
}
