use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tauri::AppHandle;
use tracing::{info, warn};

/// Debug configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfiguration {
    #[serde(rename = "type")]
    pub debug_type: String,
    pub name: String,
    pub request: String, // "launch" or "attach"
    pub program: Option<String>,
    pub args: Option<Vec<String>>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub console: Option<String>,
    #[serde(flatten)]
    pub additional_properties: HashMap<String, Value>,
}

/// Debug configuration provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfigurationProvider {
    pub id: String,
    pub owner: String,
    pub debug_type: String,
}

/// Debug adapter descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DebugAdapterDescriptor {
    #[serde(rename = "executable")]
    Executable {
        command: String,
        args: Vec<String>,
        options: Option<DebugAdapterExecutableOptions>,
    },
    #[serde(rename = "server")]
    Server { port: u16, host: Option<String> },
    #[serde(rename = "namedpipe")]
    NamedPipe { path: String },
}

/// Debug adapter executable options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugAdapterExecutableOptions {
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
}

/// Debug adapter descriptor factory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugAdapterDescriptorFactory {
    pub id: String,
    pub owner: String,
    pub debug_type: String,
}

/// Launch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfiguration {
    pub version: String,
    pub configurations: Vec<DebugConfiguration>,
}

/// Debug configuration registry
pub struct DebugConfigurationRegistry {
    configuration_providers: Arc<RwLock<Vec<DebugConfigurationProvider>>>,
    descriptor_factories: Arc<RwLock<Vec<DebugAdapterDescriptorFactory>>>,
    launch_configurations: Arc<RwLock<HashMap<String, LaunchConfiguration>>>,
    _app_handle: AppHandle,
}

impl DebugConfigurationRegistry {
    /// Create a new debug configuration registry
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            configuration_providers: Arc::new(RwLock::new(Vec::new())),
            descriptor_factories: Arc::new(RwLock::new(Vec::new())),
            launch_configurations: Arc::new(RwLock::new(HashMap::new())),
            _app_handle: app_handle,
        }
    }

    /// Register debug configuration provider
    pub fn register_debug_configuration_provider(
        &self, provider: DebugConfigurationProvider,
    ) -> Result<String, String> {
        let mut providers = self.configuration_providers.write().map_err(|e| e.to_string())?;

        let id = provider.id.clone();
        providers.push(provider.clone());

        info!(
            "Registered configuration provider: {} (type: {})",
            id, provider.debug_type
        );

        Ok(id)
    }

    /// Unregister debug configuration provider
    pub fn unregister_debug_configuration_provider(&self, provider_id: &str) -> Result<(), String> {
        let mut providers = self.configuration_providers.write().map_err(|e| e.to_string())?;

        let initial_len = providers.len();
        providers.retain(|p| p.id != provider_id);

        if providers.len() == initial_len {
            return Err(format!("Configuration provider not found: {}", provider_id));
        }

        info!("Unregistered configuration provider: {}", provider_id);

        Ok(())
    }

    /// Get debug configuration providers for type
    pub fn get_debug_configuration_providers(
        &self, debug_type: &str,
    ) -> Vec<DebugConfigurationProvider> {
        let providers = self
            .configuration_providers
            .read()
            .unwrap_or_else(|e| {
                warn!("debug_configuration_providers read lock poisoned, recovering: {}", e);
                e.into_inner()
            });
        providers.iter().filter(|p| p.debug_type == debug_type).cloned().collect()
    }

    /// Register debug adapter descriptor factory
    pub fn register_debug_adapter_descriptor_factory(
        &self, factory: DebugAdapterDescriptorFactory,
    ) -> Result<String, String> {
        let mut factories = self.descriptor_factories.write().map_err(|e| e.to_string())?;

        let id = factory.id.clone();
        factories.push(factory.clone());

        info!(
            "Registered adapter descriptor factory: {} (type: {})",
            id, factory.debug_type
        );

        Ok(id)
    }

    /// Unregister debug adapter descriptor factory
    pub fn unregister_debug_adapter_descriptor_factory(
        &self, factory_id: &str,
    ) -> Result<(), String> {
        let mut factories = self.descriptor_factories.write().map_err(|e| e.to_string())?;

        let initial_len = factories.len();
        factories.retain(|f| f.id != factory_id);

        if factories.len() == initial_len {
            return Err(format!("Adapter descriptor factory not found: {}", factory_id));
        }

        info!("Unregistered adapter descriptor factory: {}", factory_id);

        Ok(())
    }

    /// Get debug adapter descriptor factories for type
    pub fn get_debug_adapter_descriptor_factories(
        &self, debug_type: &str,
    ) -> Vec<DebugAdapterDescriptorFactory> {
        let factories = self
            .descriptor_factories
            .read()
            .unwrap_or_else(|e| {
                warn!("debug_descriptor_factories read lock poisoned, recovering: {}", e);
                e.into_inner()
            });
        factories.iter().filter(|f| f.debug_type == debug_type).cloned().collect()
    }

    /// Set launch configuration for workspace
    pub fn set_launch_configuration(
        &self, workspace_uri: String, configuration: LaunchConfiguration,
    ) -> Result<(), String> {
        let mut launch_configurations =
            self.launch_configurations.write().map_err(|e| e.to_string())?;

        launch_configurations.insert(workspace_uri.clone(), configuration);

        info!("Set launch configuration for workspace: {}", workspace_uri);

        Ok(())
    }

    /// Get launch configuration for workspace
    pub fn get_launch_configuration(
        &self, workspace_uri: &str,
    ) -> Result<LaunchConfiguration, String> {
        let launch_configurations = self.launch_configurations.read().map_err(|e| e.to_string())?;

        launch_configurations
            .get(workspace_uri)
            .cloned()
            .ok_or_else(|| format!("No launch configuration for workspace: {}", workspace_uri))
    }

    /// Get all launch configurations
    pub fn get_all_launch_configurations(&self) -> HashMap<String, LaunchConfiguration> {
        let launch_configurations = self.launch_configurations.read().unwrap_or_else(|e| {
            warn!("debug_launch_configurations read lock poisoned, recovering: {}", e);
            e.into_inner()
        });
        launch_configurations.clone()
    }

    /// Clear all debug configuration data for an owner
    pub fn clear_debug_configuration_data(&self, owner: &str) {
        // Clear configuration providers
        {
            let mut providers = self
                .configuration_providers
                .write()
                .unwrap_or_else(|e| {
                    warn!("debug_configuration_providers write lock poisoned, recovering: {}", e);
                    e.into_inner()
                });
            let before = providers.len();
            providers.retain(|p| p.owner != owner);
            let removed = before - providers.len();
            if removed > 0 {
                info!(
                    "Cleared {} configuration provider(s) for owner: {}",
                    removed, owner
                );
            }
        }

        // Clear descriptor factories
        {
            let mut factories = self
                .descriptor_factories
                .write()
                .unwrap_or_else(|e| {
                    warn!("debug_descriptor_factories write lock poisoned, recovering: {}", e);
                    e.into_inner()
                });
            let before = factories.len();
            factories.retain(|f| f.owner != owner);
            let removed = before - factories.len();
            if removed > 0 {
                info!(
                    "Cleared {} adapter descriptor factory(ies) for owner: {}",
                    removed, owner
                );
            }
        }
    }
}
