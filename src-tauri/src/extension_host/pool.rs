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
use super::nng_manager::NngIpcManager;
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
    pub outgoing_ipc_url: String,  // ExtHost listens, Tauri connects (REQ -> REP)
    pub incoming_ipc_url: String,  // Tauri listens, ExtHost connects (REP <- REQ)
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

    /// Mapping: command -> host_id (for faster routing)
    command_to_host: Arc<RwLock<HashMap<String, String>>>,

    /// Allocation strategy
    strategy: HostAllocationStrategy,

    /// Next host ID counter
    next_host_id: Arc<RwLock<usize>>,

    /// NNG IPC manager for bidirectional communication
    nng_manager: Arc<NngIpcManager>,
}

impl ExtensionHostPool {
    pub fn new(strategy: HostAllocationStrategy) -> Self {
        Self {
            hosts: Arc::new(RwLock::new(HashMap::new())),
            extension_to_host: Arc::new(RwLock::new(HashMap::new())),
            view_to_extension: Arc::new(RwLock::new(HashMap::new())),
            command_to_extension: Arc::new(RwLock::new(HashMap::new())),
            command_to_host: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            next_host_id: Arc::new(RwLock::new(1)),
            nng_manager: Arc::new(NngIpcManager::new()),
        }
    }

    /// Create a new extension host with bidirectional NNG IPC
    pub async fn create_host(&self, workspace: Option<String>) -> Result<String, String> {
        let host_id = {
            let mut counter = self.next_host_id.write().await;
            let id = format!("host-{}", *counter);
            *counter += 1;
            id
        };

        // Generate IPC URLs for bidirectional communication
        let outgoing_ipc_url = format!("ipc:///tmp/dscode-ext-out-{}.ipc", host_id);
        let incoming_ipc_url = format!("ipc:///tmp/dscode-ext-in-{}.ipc", host_id);

        let manager = Arc::new(tokio::sync::Mutex::new(ExtensionHostManager::new(host_id.clone())));

        let host_info = ExtensionHostInfo {
            id: host_id.clone(),
            outgoing_ipc_url,
            incoming_ipc_url,
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
                // Set up bidirectional NNG IPC
                // 1. Set up incoming connection (Tauri listens, ExtHost connects)
                let pool_self = self.clone_for_handler();
                let host_id_clone = host_id.clone();

                self.nng_manager.setup_incoming(
                    &host_id,
                    &host_info.incoming_ipc_url,
                    Arc::new(move |msg_type, payload| {
                        let pool = pool_self.clone();
                        let host_id = host_id_clone.clone();
                        let msg_type_owned = msg_type.to_string();

                        // Handle synchronously but spawn async work
                        tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current().block_on(async move {
                                pool.handle_incoming_request(&host_id, &msg_type_owned, payload).await
                            })
                        })
                    })
                ).await?;

                // 2. Start the extension host process with both IPC URLs
                manager.start_with_bidirectional_ipc(
                    extension_path,
                    &host_info.outgoing_ipc_url,
                    &host_info.incoming_ipc_url
                )?;

                // 3. Connect to outgoing endpoint (Tauri connects, ExtHost listens)
                // Wait a bit for extension host to start listening
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                self.nng_manager.connect_outgoing(&host_id, &host_info.outgoing_ipc_url).await?;
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

    /// Get outgoing IPC URL for a host (ExtHost listens, Tauri connects)
    pub async fn get_host_outgoing_ipc_url(&self, host_id: &str) -> Option<String> {
        self.hosts.read().await.get(host_id).map(|info| info.outgoing_ipc_url.clone())
    }

    /// Get incoming IPC URL for a host (Tauri listens, ExtHost connects)
    pub async fn get_host_incoming_ipc_url(&self, host_id: &str) -> Option<String> {
        self.hosts.read().await.get(host_id).map(|info| info.incoming_ipc_url.clone())
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

    /// Clone for handler (returns Arc-wrapped version for thread safety)
    fn clone_for_handler(&self) -> PoolHandlerClone {
        PoolHandlerClone {
            hosts: Arc::clone(&self.hosts),
            extension_to_host: Arc::clone(&self.extension_to_host),
            command_to_extension: Arc::clone(&self.command_to_extension),
            command_to_host: Arc::clone(&self.command_to_host),
            view_to_extension: Arc::clone(&self.view_to_extension),
            nng_manager: Arc::clone(&self.nng_manager),
        }
    }

    /// Execute a command (called from Tauri/UI)
    pub async fn execute_command(&self, command: &str, args: Vec<serde_json::Value>) -> Result<serde_json::Value, String> {
        // Route to the correct host
        if let Some(target_host_id) = self.command_to_host.read().await.get(command) {
            let target_host_id = target_host_id.clone();

            // Get the IPC connection
            if let Some(ipc) = self.nng_manager.get_outgoing(&target_host_id).await {
                // Send execute-command request to extension host
                let request_payload = serde_json::json!({
                    "command": command,
                    "args": args
                });

                match ipc.request("execute-command", request_payload).await {
                    Ok(response) => Ok(response),
                    Err(e) => Err(format!("Command execution failed: {}", e)),
                }
            } else {
                Err(format!("Host {} not connected", target_host_id))
            }
        } else {
            Err(format!("Command '{}' not registered", command))
        }
    }

    /// Get list of registered commands
    pub async fn list_commands(&self) -> Vec<String> {
        self.command_to_extension.read().await.keys().cloned().collect()
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

/// Cloneable handler for processing incoming requests
#[derive(Clone)]
struct PoolHandlerClone {
    hosts: Arc<RwLock<HashMap<String, ExtensionHostInfo>>>,
    extension_to_host: Arc<RwLock<HashMap<String, String>>>,
    command_to_extension: Arc<RwLock<HashMap<String, String>>>,
    command_to_host: Arc<RwLock<HashMap<String, String>>>,
    view_to_extension: Arc<RwLock<HashMap<String, String>>>,
    nng_manager: Arc<NngIpcManager>,
}

impl PoolHandlerClone {
    /// Handle incoming request from extension host
    async fn handle_incoming_request(
        &self,
        host_id: &str,
        msg_type: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        println!("[HostPool] Received {} from host {}", msg_type, host_id);

        match msg_type {
            // Command registration
            "command-registered" => {
                let command = payload.get("command")
                    .and_then(|c| c.as_str())
                    .ok_or("Missing command name")?;

                // Find which extension this host is running
                let extension_id = {
                    let hosts = self.hosts.read().await;
                    let host_info = hosts.get(host_id)
                        .ok_or(format!("Host {} not found", host_id))?;

                    // Get first extension from this host (assuming one extension per registration)
                    host_info.loaded_extensions.first()
                        .ok_or("No extension loaded in host")?
                        .clone()
                };

                // Register the command
                self.command_to_extension.write().await.insert(command.to_string(), extension_id.clone());
                self.command_to_host.write().await.insert(command.to_string(), host_id.to_string());

                println!("[HostPool] Registered command '{}' from extension '{}' on host '{}'",
                    command, extension_id, host_id);

                Ok(serde_json::json!({ "status": "ok" }))
            },

            // Command unregistration
            "command-unregistered" => {
                let command = payload.get("command")
                    .and_then(|c| c.as_str())
                    .ok_or("Missing command name")?;

                self.command_to_extension.write().await.remove(command);
                self.command_to_host.write().await.remove(command);

                println!("[HostPool] Unregistered command '{}'", command);

                Ok(serde_json::json!({ "status": "ok" }))
            },

            // Window API messages
            "window-show-message" => {
                let message = payload.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Message");
                let msg_type = payload.get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("info");

                println!("[HostPool] Window {} message: {}", msg_type, message);
                // TODO: Show actual UI dialog via Tauri
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "window-show-message-with-actions" => {
                let message = payload.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Message");
                let msg_type = payload.get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("info");

                println!("[HostPool] Window {} message with actions: {}", msg_type, message);
                // TODO: Show actual UI dialog with buttons via Tauri
                Ok(serde_json::json!({ "action": null }))
            },

            "window-show-input-box" => {
                let prompt = payload.get("prompt")
                    .and_then(|p| p.as_str())
                    .unwrap_or("");

                println!("[HostPool] Window input box: {}", prompt);
                // TODO: Show actual input dialog via Tauri
                Ok(serde_json::json!({ "value": null }))
            },

            "window-show-quick-pick" => {
                println!("[HostPool] Window quick pick");
                // TODO: Show actual quick pick via Tauri
                Ok(serde_json::json!({ "selected": null }))
            },

            // Workspace API
            "workspace-get-folders" => {
                println!("[HostPool] Workspace get folders");
                // TODO: Get actual workspace folders from Tauri state
                // For now, return empty array to allow extensions to load
                Ok(serde_json::json!([]))
            },

            "workspace-find-files" => {
                println!("[HostPool] Workspace find files");
                // TODO: Implement file searching
                Ok(serde_json::json!([]))
            },

            // Configuration API
            "workspace-get-configuration" => {
                let section = payload.get("section")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                println!("[HostPool] Get configuration: {}", section);
                // TODO: Implement actual configuration storage
                Ok(serde_json::json!({}))
            },

            "workspace-update-configuration" => {
                println!("[HostPool] Update configuration");
                // TODO: Implement configuration updates
                Ok(serde_json::json!({ "status": "ok" }))
            },

            // Execute command request (from extension to Tauri/UI)
            "execute-command-request" => {
                // Extensions can call executeCommand to run other commands
                // For now, we'll route it back through the command system
                let command = payload.get("command")
                    .and_then(|c| c.as_str())
                    .ok_or("Missing command name")?;

                let args = payload.get("args")
                    .and_then(|a| a.as_array())
                    .map(|a| a.clone())
                    .unwrap_or_default();

                // Route to the correct host
                if let Some(target_host_id) = self.command_to_host.read().await.get(command) {
                    let target_host_id = target_host_id.clone();

                    // Get the IPC connection
                    if let Some(ipc) = self.nng_manager.get_outgoing(&target_host_id).await {
                        // Send execute-command request to extension host
                        let request_payload = serde_json::json!({
                            "command": command,
                            "args": args
                        });

                        match ipc.request("execute-command", request_payload).await {
                            Ok(response) => Ok(response),
                            Err(e) => Err(format!("Command execution failed: {}", e)),
                        }
                    } else {
                        Err(format!("Host {} not connected", target_host_id))
                    }
                } else {
                    Err(format!("Command '{}' not found", command))
                }
            },

            // Webview API
            "createWebviewPanel" => {
                let panel_id = payload.get("panelId")
                    .and_then(|p| p.as_str())
                    .unwrap_or("unknown");
                let view_type = payload.get("viewType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let title = payload.get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("Webview");

                println!("[HostPool] Creating webview panel '{}' (type: {}, id: {})", title, view_type, panel_id);
                // TODO: Forward to Tauri UI to create actual webview
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "updateWebviewHtml" => {
                let panel_id = payload.get("panelId")
                    .and_then(|p| p.as_str())
                    .unwrap_or("unknown");
                let html = payload.get("html")
                    .and_then(|h| h.as_str())
                    .unwrap_or("");

                println!("[HostPool] Updating webview HTML for panel: {} (length: {} bytes)", panel_id, html.len());
                // TODO: Forward HTML to Tauri UI webview
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "updateWebviewTitle" => {
                let panel_id = payload.get("panelId")
                    .and_then(|p| p.as_str())
                    .unwrap_or("unknown");
                let title = payload.get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                println!("[HostPool] Updating webview title for panel {}: {}", panel_id, title);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "webviewPostMessage" => {
                let panel_id = payload.get("panelId")
                    .and_then(|p| p.as_str())
                    .unwrap_or("unknown");
                let message = payload.get("message");

                println!("[HostPool] Webview post message from panel {}: {:?}", panel_id, message);
                // TODO: Forward message to Tauri UI webview
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "revealWebview" => {
                let panel_id = payload.get("panelId")
                    .and_then(|p| p.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Revealing webview panel: {}", panel_id);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "disposeWebview" => {
                let panel_id = payload.get("panelId")
                    .and_then(|p| p.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Disposing webview panel: {}", panel_id);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "registerWebviewSerializer" => {
                let view_type = payload.get("viewType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                println!("[HostPool] Registering webview serializer for type: {}", view_type);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "unregisterWebviewSerializer" => {
                let view_type = payload.get("viewType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                println!("[HostPool] Unregistering webview serializer for type: {}", view_type);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            // Terminal API
            "createTerminal" => {
                let terminal_id = payload.get("terminalId")
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown");
                let options = payload.get("options");

                println!("[HostPool] Creating terminal: {} (options: {:?})", terminal_id, options);
                // TODO: Forward to Tauri UI to create actual terminal
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "terminalSendText" => {
                let terminal_id = payload.get("terminalId")
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown");
                let text = payload.get("text")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                let should_execute = payload.get("shouldExecute")
                    .and_then(|s| s.as_bool())
                    .unwrap_or(true);

                println!("[HostPool] Terminal {} send text: {} (execute: {})", terminal_id, text, should_execute);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "terminalShow" => {
                let terminal_id = payload.get("terminalId")
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Showing terminal: {}", terminal_id);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "terminalHide" => {
                let terminal_id = payload.get("terminalId")
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Hiding terminal: {}", terminal_id);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "terminalDispose" => {
                let terminal_id = payload.get("terminalId")
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Disposing terminal: {}", terminal_id);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "getTerminalProcessId" => {
                let terminal_id = payload.get("terminalId")
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Getting process ID for terminal: {}", terminal_id);
                // TODO: Return actual process ID
                Ok(serde_json::json!({ "processId": null }))
            },

            // TreeView API
            "registerTreeDataProvider" => {
                let view_id = payload.get("viewId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Registering tree data provider for view: {}", view_id);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "revealTreeItem" => {
                let view_id = payload.get("viewId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Revealing tree item in view: {}", view_id);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "disposeTreeView" => {
                let view_id = payload.get("viewId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Disposing tree view: {}", view_id);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            // StatusBar API
            "updateStatusBarItem" => {
                let item_id = payload.get("id")
                    .and_then(|i| i.as_str())
                    .unwrap_or("unknown");
                let text = payload.get("text")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                println!("[HostPool] Updating status bar item {}: {}", item_id, text);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "hideStatusBarItem" => {
                let item_id = payload.get("id")
                    .and_then(|i| i.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Hiding status bar item: {}", item_id);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "disposeStatusBarItem" => {
                let item_id = payload.get("id")
                    .and_then(|i| i.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Disposing status bar item: {}", item_id);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            // Language Provider registrations (LSP)
            msg_type if msg_type.starts_with("register") && msg_type.ends_with("Provider") => {
                println!("[HostPool] Registering language provider: {}", msg_type);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            // Debug API registrations (DAP)
            "registerDebugConfigurationProvider" => {
                let debug_type = payload.get("debugType")
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Registering debug configuration provider for: {}", debug_type);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "registerDebugAdapterDescriptorFactory" => {
                let debug_type = payload.get("debugType")
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Registering debug adapter descriptor factory for: {}", debug_type);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "registerDebugAdapterTrackerFactory" => {
                let debug_type = payload.get("debugType")
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Registering debug adapter tracker factory for: {}", debug_type);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "registerTaskProvider" => {
                let task_type = payload.get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Registering task provider for: {}", task_type);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            // Diagnostic collection
            "createDiagnosticCollection" => {
                let name = payload.get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Creating diagnostic collection: {}", name);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            "updateDiagnostics" => {
                let uri = payload.get("uri")
                    .and_then(|u| u.as_str())
                    .unwrap_or("unknown");

                println!("[HostPool] Updating diagnostics for: {}", uri);
                Ok(serde_json::json!({ "status": "ok" }))
            },

            _ => {
                println!("[HostPool] Unhandled message type: {}", msg_type);
                Ok(serde_json::json!({ "status": "ok" }))
            }
        }
    }
}
