use super::debug_configuration_registry::*;
use tauri::State;

/// Register debug configuration provider
#[tauri::command]
pub async fn register_debug_configuration_provider(
    provider: DebugConfigurationProvider, registry: State<'_, DebugConfigurationRegistry>,
) -> Result<String, String> {
    registry.register_debug_configuration_provider(provider)
}

/// Unregister debug configuration provider
#[tauri::command]
pub async fn unregister_debug_configuration_provider(
    provider_id: String, registry: State<'_, DebugConfigurationRegistry>,
) -> Result<(), String> {
    registry.unregister_debug_configuration_provider(&provider_id)
}

/// Get debug configuration providers for type
#[tauri::command]
pub async fn get_debug_configuration_providers(
    debug_type: String, registry: State<'_, DebugConfigurationRegistry>,
) -> Result<Vec<DebugConfigurationProvider>, String> {
    Ok(registry.get_debug_configuration_providers(&debug_type))
}

/// Register debug adapter descriptor factory
#[tauri::command]
pub async fn register_debug_adapter_descriptor_factory(
    factory: DebugAdapterDescriptorFactory, registry: State<'_, DebugConfigurationRegistry>,
) -> Result<String, String> {
    registry.register_debug_adapter_descriptor_factory(factory)
}

/// Unregister debug adapter descriptor factory
#[tauri::command]
pub async fn unregister_debug_adapter_descriptor_factory(
    factory_id: String, registry: State<'_, DebugConfigurationRegistry>,
) -> Result<(), String> {
    registry.unregister_debug_adapter_descriptor_factory(&factory_id)
}

/// Get debug adapter descriptor factories for type
#[tauri::command]
pub async fn get_debug_adapter_descriptor_factories(
    debug_type: String, registry: State<'_, DebugConfigurationRegistry>,
) -> Result<Vec<DebugAdapterDescriptorFactory>, String> {
    Ok(registry.get_debug_adapter_descriptor_factories(&debug_type))
}

/// Set launch configuration
#[tauri::command]
pub async fn set_launch_configuration(
    workspace_uri: String, configuration: LaunchConfiguration,
    registry: State<'_, DebugConfigurationRegistry>,
) -> Result<(), String> {
    registry.set_launch_configuration(workspace_uri, configuration)
}

/// Get launch configuration
#[tauri::command]
pub async fn get_launch_configuration(
    workspace_uri: String, registry: State<'_, DebugConfigurationRegistry>,
) -> Result<LaunchConfiguration, String> {
    registry.get_launch_configuration(&workspace_uri)
}

/// Get all launch configurations
#[tauri::command]
pub async fn get_all_launch_configurations(
    registry: State<'_, DebugConfigurationRegistry>,
) -> Result<std::collections::HashMap<String, LaunchConfiguration>, String> {
    Ok(registry.get_all_launch_configurations())
}

/// Clear debug configuration data for owner
#[tauri::command]
pub async fn clear_debug_configuration_data(
    owner: String, registry: State<'_, DebugConfigurationRegistry>,
) -> Result<(), String> {
    registry.clear_debug_configuration_data(&owner);
    Ok(())
}
