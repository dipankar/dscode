/**
 * Extension Host Pool Manager
 *
 * Manages multiple extension host processes with:
 * - Dynamic extension loading/unloading
 * - Request routing to correct host
 * - Host lifecycle management
 * - Load balancing
 */

use super::manager::ExtensionHostManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Strategy for assigning extensions to hosts
#[derive(Debug, Clone)]
pub enum HostAllocationStrategy {
    /// One host per extension (maximum isolation)
    OnePerExtension,
    /// One host per workspace
    OnePerWorkspace,
    /// Shared host with load balancing
    Shared { max_extensions_per_host: usize },
    /// Custom allocation function
    Custom,
}

/// Information about a running extension host
pub struct ExtensionHostInfo {
    pub id: String,
    pub ipc_url: String,
    pub manager: Arc<tokio::sync::Mutex<ExtensionHostManager>>,
    pub loaded_extensions: Vec<String>,
    pub workspace: Option<String>,
}

/// Routes requests to the correct extension host
pub struct ExtensionHostPool {
    /// All running extension hosts
    hosts: Arc<RwLock<HashMap<String, ExtensionHostInfo>>>,

    /// Mapping: extension_id -> host_id
    extension_to_host: Arc<RwLock<HashMap<String, String>>>,

    /// Mapping: view_id -> extension_id
    view_to_extension: Arc<RwLock<HashMap<String, String>>>,

    /// Mapping: command -> extension_id
    command_to_extension: Arc<RwLock<HashMap<String, String>>>,

    /// Allocation strategy
    strategy: HostAllocationStrategy,

    /// Next host ID counter
    next_host_id: Arc<RwLock<usize>>,
}

impl ExtensionHostPool {
    pub fn new(strategy: HostAllocationStrategy) -> Self {
        Self {
            hosts: Arc::new(RwLock::new(HashMap::new())),
            extension_to_host: Arc::new(RwLock::new(HashMap::new())),
            view_to_extension: Arc::new(RwLock::new(HashMap::new())),
            command_to_extension: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            next_host_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Create a new extension host
    pub async fn create_host(&self, workspace: Option<String>) -> Result<String, String> {
        let host_id = {
            let mut counter = self.next_host_id.write().await;
            let id = format!("host-{}", *counter);
            *counter += 1;
            id
        };

        let ipc_url = format!("ipc:///tmp/dscode-ext-{}.ipc", host_id);
        let manager = Arc::new(tokio::sync::Mutex::new(ExtensionHostManager::new(host_id.clone())));

        let host_info = ExtensionHostInfo {
            id: host_id.clone(),
            ipc_url,
            manager,
            loaded_extensions: Vec::new(),
            workspace,
        };

        self.hosts.write().await.insert(host_id.clone(), host_info);

        println!("[HostPool] Created extension host: {}", host_id);
        Ok(host_id)
    }

    /// Get or create a host for an extension
    pub async fn get_host_for_extension(&self, extension_id: &str, workspace: Option<String>) -> Result<String, String> {
        // Check if extension is already loaded
        if let Some(host_id) = self.extension_to_host.read().await.get(extension_id) {
            return Ok(host_id.clone());
        }

        // Find or create a suitable host based on strategy
        let host_id = match &self.strategy {
            HostAllocationStrategy::OnePerExtension => {
                // Create a new dedicated host for this extension
                self.create_host(workspace).await?
            }
            HostAllocationStrategy::OnePerWorkspace => {
                // Find host for this workspace or create new one
                self.find_or_create_workspace_host(workspace).await?
            }
            HostAllocationStrategy::Shared { max_extensions_per_host } => {
                // Find host with capacity or create new one
                self.find_or_create_shared_host(*max_extensions_per_host, workspace).await?
            }
            HostAllocationStrategy::Custom => {
                // For now, default to shared
                self.find_or_create_shared_host(10, workspace).await?
            }
        };

        // Map extension to host
        self.extension_to_host.write().await.insert(extension_id.to_string(), host_id.clone());

        // Add to host's loaded extensions list
        if let Some(host) = self.hosts.write().await.get_mut(&host_id) {
            host.loaded_extensions.push(extension_id.to_string());
        }

        Ok(host_id)
    }

    /// Find workspace host or create one
    async fn find_or_create_workspace_host(&self, workspace: Option<String>) -> Result<String, String> {
        let hosts = self.hosts.read().await;

        // Find existing host for this workspace
        for (host_id, host_info) in hosts.iter() {
            if host_info.workspace == workspace {
                return Ok(host_id.clone());
            }
        }

        drop(hosts);

        // Create new host for workspace
        self.create_host(workspace).await
    }

    /// Find shared host with capacity or create one
    async fn find_or_create_shared_host(&self, max_extensions: usize, workspace: Option<String>) -> Result<String, String> {
        let hosts = self.hosts.read().await;

        // Find host with capacity
        for (host_id, host_info) in hosts.iter() {
            if host_info.loaded_extensions.len() < max_extensions {
                return Ok(host_id.clone());
            }
        }

        drop(hosts);

        // All hosts at capacity, create new one
        self.create_host(workspace).await
    }

    /// Load an extension into a host
    pub async fn load_extension(
        &self,
        extension_id: &str,
        extension_path: &str,
        workspace: Option<String>,
    ) -> Result<String, String> {
        // Get or create host for this extension
        let host_id = self.get_host_for_extension(extension_id, workspace).await?;

        // Get the host
        let hosts = self.hosts.read().await;
        let host_info = hosts.get(&host_id)
            .ok_or(format!("Host {} not found", host_id))?;

        // Start the host if not already running
        {
            let mut manager = host_info.manager.lock().await;
            if !manager.is_running() {
                manager.start_with_ipc(extension_path, &host_info.ipc_url)?;
            }
        }

        println!("[HostPool] Loaded extension {} into host {}", extension_id, host_id);
        Ok(host_id)
    }

    /// Unload an extension from its host
    pub async fn unload_extension(&self, extension_id: &str) -> Result<(), String> {
        // Find which host has this extension
        let host_id = self.extension_to_host.read().await.get(extension_id)
            .ok_or(format!("Extension {} not loaded", extension_id))?
            .clone();

        // Remove from mappings
        self.extension_to_host.write().await.remove(extension_id);

        // Remove from host's loaded extensions
        let mut hosts = self.hosts.write().await;
        if let Some(host) = hosts.get_mut(&host_id) {
            host.loaded_extensions.retain(|id| id != extension_id);

            // If host is now empty, consider shutting it down
            if host.loaded_extensions.is_empty() {
                println!("[HostPool] Host {} is now empty, shutting down", host_id);
                let mut manager = host.manager.lock().await;
                manager.stop()?;
                drop(manager);
                hosts.remove(&host_id);
            }
        }

        println!("[HostPool] Unloaded extension {}", extension_id);
        Ok(())
    }

    /// Register a view provided by an extension
    pub async fn register_view(&self, view_id: &str, extension_id: &str) {
        self.view_to_extension.write().await.insert(view_id.to_string(), extension_id.to_string());
    }

    /// Register a command provided by an extension
    pub async fn register_command(&self, command: &str, extension_id: &str) {
        self.command_to_extension.write().await.insert(command.to_string(), extension_id.to_string());
    }

    /// Route a view request to the correct host
    pub async fn route_view_request(&self, view_id: &str) -> Result<String, String> {
        // Find extension that owns this view
        let extension_id = self.view_to_extension.read().await.get(view_id)
            .ok_or(format!("No extension registered for view {}", view_id))?
            .clone();

        // Find host for that extension
        let host_id = self.extension_to_host.read().await.get(&extension_id)
            .ok_or(format!("Extension {} not loaded", extension_id))?
            .clone();

        Ok(host_id)
    }

    /// Route a command request to the correct host
    pub async fn route_command_request(&self, command: &str) -> Result<String, String> {
        // Find extension that owns this command
        let extension_id = self.command_to_extension.read().await.get(command)
            .ok_or(format!("No extension registered for command {}", command))?
            .clone();

        // Find host for that extension
        let host_id = self.extension_to_host.read().await.get(&extension_id)
            .ok_or(format!("Extension {} not loaded", extension_id))?
            .clone();

        Ok(host_id)
    }

    /// Get host info
    pub async fn get_host(&self, host_id: &str) -> Option<Arc<tokio::sync::Mutex<ExtensionHostManager>>> {
        self.hosts.read().await.get(host_id).map(|info| info.manager.clone())
    }

    /// Get IPC URL for a host
    pub async fn get_host_ipc_url(&self, host_id: &str) -> Option<String> {
        self.hosts.read().await.get(host_id).map(|info| info.ipc_url.clone())
    }

    /// List all running hosts
    pub async fn list_hosts(&self) -> Vec<String> {
        self.hosts.read().await.keys().cloned().collect()
    }

    /// Get statistics
    pub async fn get_stats(&self) -> HostPoolStats {
        let hosts = self.hosts.read().await;
        let total_hosts = hosts.len();
        let total_extensions = self.extension_to_host.read().await.len();
        let total_views = self.view_to_extension.read().await.len();
        let total_commands = self.command_to_extension.read().await.len();

        HostPoolStats {
            total_hosts,
            total_extensions,
            total_views,
            total_commands,
        }
    }

    /// Shutdown all hosts
    pub async fn shutdown_all(&self) -> Result<(), String> {
        let mut hosts = self.hosts.write().await;

        for (host_id, host_info) in hosts.iter() {
            println!("[HostPool] Shutting down host {}", host_id);
            let mut manager = host_info.manager.lock().await;
            manager.stop()?;
        }

        hosts.clear();
        self.extension_to_host.write().await.clear();
        self.view_to_extension.write().await.clear();
        self.command_to_extension.write().await.clear();

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct HostPoolStats {
    pub total_hosts: usize,
    pub total_extensions: usize,
    pub total_views: usize,
    pub total_commands: usize,
}
