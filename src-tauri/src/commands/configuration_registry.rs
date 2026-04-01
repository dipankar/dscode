use dscode_core::CoreError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter};
use tracing::{error, info, warn};

/// Configuration scope
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ConfigurationScope {
    User,
    Workspace,
    WorkspaceFolder,
}

/// Configuration schema for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationSchema {
    pub key: String,
    pub scope: ConfigurationScope,
    #[serde(rename = "type")]
    pub value_type: String,
    pub default: Value,
    pub description: String,
    pub enum_values: Option<Vec<Value>>,
}

/// Configuration contribution from extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationContribution {
    pub extension_id: String,
    pub title: String,
    pub properties: Vec<ConfigurationSchema>,
}

/// Configuration change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationChangeEvent {
    pub affected_keys: Vec<String>,
    pub scope: ConfigurationScope,
}

/// Configuration value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationValue {
    pub key: String,
    pub value: Value,
    pub scope: ConfigurationScope,
    pub default_value: Option<Value>,
}

/// Configuration registry
pub struct ConfigurationRegistry {
    schemas: Arc<RwLock<HashMap<String, ConfigurationSchema>>>,
    user_settings: Arc<RwLock<HashMap<String, Value>>>,
    workspace_settings: Arc<RwLock<HashMap<String, Value>>>,
    workspace_folder_settings: Arc<RwLock<HashMap<String, HashMap<String, Value>>>>,
    settings_path: Arc<RwLock<Option<PathBuf>>>,
    workspace_path: Arc<RwLock<Option<PathBuf>>>,
    app_handle: AppHandle,
}

impl ConfigurationRegistry {
    /// Create a new configuration registry
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
            user_settings: Arc::new(RwLock::new(HashMap::new())),
            workspace_settings: Arc::new(RwLock::new(HashMap::new())),
            workspace_folder_settings: Arc::new(RwLock::new(HashMap::new())),
            settings_path: Arc::new(RwLock::new(None)),
            workspace_path: Arc::new(RwLock::new(None)),
            app_handle,
        }
    }

    /// Set settings file path
    pub fn set_settings_path(&self, path: PathBuf) -> Result<(), CoreError> {
        let mut settings_path = self.settings_path.write().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;
        *settings_path = Some(path.clone());

        // Load user settings
        self.load_user_settings()?;

        info!("Set settings path: {:?}", path);
        Ok(())
    }

    /// Set workspace path
    pub fn set_workspace_path(&self, path: PathBuf) -> Result<(), CoreError> {
        let mut workspace_path = self.workspace_path.write().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;
        *workspace_path = Some(path.clone());

        // Load workspace settings
        self.load_workspace_settings()?;

        info!("Set workspace path: {:?}", path);
        Ok(())
    }

    /// Register configuration schema
    pub fn register_configuration_schema(
        &self, contribution: ConfigurationContribution,
    ) -> Result<(), CoreError> {
        let mut schemas = self.schemas.write().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        let count = contribution.properties.len();
        let extension_id = contribution.extension_id.clone();

        for property in contribution.properties {
            schemas.insert(property.key.clone(), property);
        }

        info!("Registered {} schema(s) from: {}", count, extension_id);

        Ok(())
    }

    /// Get configuration value
    pub fn get_configuration(
        &self, key: &str, scope: ConfigurationScope,
    ) -> Result<Option<Value>, CoreError> {
        match scope {
            ConfigurationScope::User => {
                let user_settings = self.user_settings.read().map_err(|e| {
                    CoreError::Io(std::io::Error::other(e.to_string()))
                })?;
                Ok(user_settings.get(key).cloned())
            }
            ConfigurationScope::Workspace => {
                let workspace_settings = self.workspace_settings.read().map_err(|e| {
                    CoreError::Io(std::io::Error::other(e.to_string()))
                })?;
                Ok(workspace_settings.get(key).cloned())
            }
            ConfigurationScope::WorkspaceFolder => {
                // For now, return workspace settings
                let workspace_settings = self.workspace_settings.read().map_err(|e| {
                    CoreError::Io(std::io::Error::other(e.to_string()))
                })?;
                Ok(workspace_settings.get(key).cloned())
            }
        }
    }

    /// Get configuration with fallback
    pub fn get_configuration_with_fallback(
        &self, key: &str, scope: ConfigurationScope,
    ) -> Result<Option<Value>, CoreError> {
        // Try scope-specific first
        if let Some(value) = self.get_configuration(key, scope.clone())? {
            return Ok(Some(value));
        }

        // Fallback to workspace if scope is workspace folder
        if scope == ConfigurationScope::WorkspaceFolder {
            if let Some(value) = self.get_configuration(key, ConfigurationScope::Workspace)? {
                return Ok(Some(value));
            }
        }

        // Fallback to user settings
        if scope != ConfigurationScope::User {
            if let Some(value) = self.get_configuration(key, ConfigurationScope::User)? {
                return Ok(Some(value));
            }
        }

        // Finally, check schema for default value
        let schemas = self.schemas.read().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;
        if let Some(schema) = schemas.get(key) {
            return Ok(Some(schema.default.clone()));
        }

        Ok(None)
    }

    /// Update configuration value
    pub fn update_configuration(
        &self, key: String, value: Value, scope: ConfigurationScope,
    ) -> Result<(), CoreError> {
        // Validate against schema if exists
        let schemas = self.schemas.read().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;
        if let Some(schema) = schemas.get(&key) {
            self.validate_value(&value, schema)?;
        }
        drop(schemas);

        // Update in-memory settings
        match scope {
            ConfigurationScope::User => {
                let mut user_settings = self.user_settings.write().map_err(|e| {
                    CoreError::Io(std::io::Error::other(e.to_string()))
                })?;
                user_settings.insert(key.clone(), value.clone());
                drop(user_settings);

                // Save to file
                self.save_user_settings()?;
            }
            ConfigurationScope::Workspace => {
                let mut workspace_settings = self.workspace_settings.write().map_err(|e| {
                    CoreError::Io(std::io::Error::other(e.to_string()))
                })?;
                workspace_settings.insert(key.clone(), value.clone());
                drop(workspace_settings);

                // Save to file
                self.save_workspace_settings()?;
            }
            ConfigurationScope::WorkspaceFolder => {
                let mut workspace_settings = self.workspace_settings.write().map_err(|e| {
                    CoreError::Io(std::io::Error::other(e.to_string()))
                })?;
                workspace_settings.insert(key.clone(), value.clone());
                drop(workspace_settings);

                // Save to file (using workspace settings for now)
                self.save_workspace_settings()?;
            }
        }

        // Emit change event
        if let Err(e) = self.app_handle.emit(
            "configuration-changed",
            ConfigurationChangeEvent { affected_keys: vec![key], scope },
        ) {
            error!("Failed to emit change event: {}", e);
        }

        Ok(())
    }

    /// Validate value against schema
    fn validate_value(&self, value: &Value, schema: &ConfigurationSchema) -> Result<(), CoreError> {
        // Check type
        let value_type = match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        };

        if schema.value_type != "any" && schema.value_type != value_type {
            return Err(CoreError::Config(format!(
                "Invalid type for {}: expected {}, got {}",
                schema.key, schema.value_type, value_type
            )));
        }

        // Check enum values if specified
        if let Some(ref enum_values) = schema.enum_values {
            if !enum_values.contains(value) {
                return Err(CoreError::Config(format!(
                    "Invalid value for {}: must be one of {:?}",
                    schema.key, enum_values
                )));
            }
        }

        Ok(())
    }

    /// Load user settings from file
    fn load_user_settings(&self) -> Result<(), CoreError> {
        let settings_path = self.settings_path.read().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        if let Some(path) = settings_path.as_ref() {
            if path.exists() {
                let content = fs::read_to_string(path).map_err(CoreError::from)?;

                let settings: HashMap<String, Value> = serde_json::from_str(&content)
                    .map_err(|e| CoreError::Config(format!("Failed to parse user settings: {}", e)))?;

                let mut user_settings = self.user_settings.write().map_err(|e| {
                    CoreError::Io(std::io::Error::other(e.to_string()))
                })?;
                *user_settings = settings;

                info!("Loaded user settings from: {:?}", path);
            }
        }

        Ok(())
    }

    /// Load workspace settings from file
    fn load_workspace_settings(&self) -> Result<(), CoreError> {
        let workspace_path = self.workspace_path.read().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        if let Some(path) = workspace_path.as_ref() {
            let settings_file = path.join(".vscode").join("settings.json");

            if settings_file.exists() {
                let content = fs::read_to_string(&settings_file).map_err(CoreError::from)?;

                let settings: HashMap<String, Value> = serde_json::from_str(&content)
                    .map_err(|e| CoreError::Config(format!("Failed to parse workspace settings: {}", e)))?;

                let mut workspace_settings = self.workspace_settings.write().map_err(|e| {
                    CoreError::Io(std::io::Error::other(e.to_string()))
                })?;
                *workspace_settings = settings;

                info!("Loaded workspace settings from: {:?}", settings_file);
            }
        }

        Ok(())
    }

    /// Save user settings to file
    fn save_user_settings(&self) -> Result<(), CoreError> {
        let settings_path = self.settings_path.read().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        if let Some(path) = settings_path.as_ref() {
            let user_settings = self.user_settings.read().map_err(|e| {
                CoreError::Io(std::io::Error::other(e.to_string()))
            })?;

            let content = serde_json::to_string_pretty(&*user_settings)
                .map_err(|e| CoreError::Config(format!("Failed to serialize user settings: {}", e)))?;

            // Create parent directory if needed
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(CoreError::from)?;
            }

            fs::write(path, content).map_err(CoreError::from)?;

            info!("Saved user settings to: {:?}", path);
        }

        Ok(())
    }

    /// Save workspace settings to file
    fn save_workspace_settings(&self) -> Result<(), CoreError> {
        let workspace_path = self.workspace_path.read().map_err(|e| {
            CoreError::Io(std::io::Error::other(e.to_string()))
        })?;

        if let Some(path) = workspace_path.as_ref() {
            let settings_file = path.join(".vscode").join("settings.json");
            let workspace_settings = self.workspace_settings.read().map_err(|e| {
                CoreError::Io(std::io::Error::other(e.to_string()))
            })?;

            let content = serde_json::to_string_pretty(&*workspace_settings)
                .map_err(|e| CoreError::Config(format!("Failed to serialize workspace settings: {}", e)))?;

            // Create .vscode directory if needed
            let vscode_dir = path.join(".vscode");
            if !vscode_dir.exists() {
                fs::create_dir_all(&vscode_dir).map_err(CoreError::from)?;
            }

            fs::write(&settings_file, content).map_err(CoreError::from)?;

            info!("Saved workspace settings to: {:?}", settings_file);
        }

        Ok(())
    }

    /// Get all configuration keys for a scope
    pub fn get_all_keys(&self, scope: ConfigurationScope) -> Vec<String> {
        match scope {
            ConfigurationScope::User => {
                let settings = self.user_settings.read().unwrap_or_else(|e| {
                    warn!("user_settings read lock poisoned, recovering: {}", e);
                    e.into_inner()
                });
                settings.keys().cloned().collect()
            }
            ConfigurationScope::Workspace => {
                let settings = self.workspace_settings.read().unwrap_or_else(|e| {
                    warn!("workspace_settings read lock poisoned, recovering: {}", e);
                    e.into_inner()
                });
                settings.keys().cloned().collect()
            }
            ConfigurationScope::WorkspaceFolder => {
                let settings = self.workspace_settings.read().unwrap_or_else(|e| {
                    warn!("workspace_settings read lock poisoned, recovering: {}", e);
                    e.into_inner()
                });
                settings.keys().cloned().collect()
            }
        }
    }

    /// Check if configuration key exists
    pub fn has_configuration(&self, key: &str, scope: ConfigurationScope) -> bool {
        self.get_configuration(key, scope).ok().flatten().is_some()
    }

    /// Clear all configuration data for an owner
    pub fn clear_configuration_data(&self, owner: &str) {
        let mut schemas = self.schemas.write().unwrap_or_else(|e| {
            warn!("configuration_schemas write lock poisoned, recovering: {}", e);
            e.into_inner()
        });
        let before = schemas.len();
        schemas.retain(|_, schema| !schema.key.starts_with(&format!("{}.", owner)));
        let removed = before - schemas.len();

        if removed > 0 {
            info!("Cleared {} schema(s) for owner: {}", removed, owner);
        }
    }
}
