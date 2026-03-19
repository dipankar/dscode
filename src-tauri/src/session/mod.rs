/**
 * Session Manager
 *
 * Central coordinator for the entire application session.
 * Manages:
 * - Extension lifecycle (install, load, unload, delete)
 * - LSP/DAP server coordination
 * - Application state synchronization
 * - Event emission to UI
 */

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tauri::{AppHandle, Emitter, Manager};

use crate::extension_host::{ExtensionHostManager, NngIpcManager};
use crate::lsp::{LspServerPool, LspServerStrategy};
use crate::debug::DebugAdapterPool;

/// Application session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub workspace_folders: Vec<PathBuf>,
    pub active_extensions: Vec<ExtensionInfo>,
    pub installed_extensions: Vec<ExtensionInfo>,
}

/// Extension information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub active: bool,
}

/// Session events emitted to the UI
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum SessionEvent {
    /// Extension was installed
    ExtensionInstalled { extension_id: String },

    /// Extension was loaded/activated
    ExtensionLoaded { extension_id: String },

    /// Extension was unloaded/deactivated
    ExtensionUnloaded { extension_id: String },

    /// Extension was deleted
    ExtensionDeleted { extension_id: String },

    /// Extension list changed
    ExtensionsChanged { extensions: Vec<ExtensionInfo> },

    /// Workspace folder added
    WorkspaceFolderAdded { path: PathBuf },

    /// Workspace folder removed
    WorkspaceFolderRemoved { path: PathBuf },

    /// Session state changed
    StateChanged { state: SessionState },
}

/// Central session manager
pub struct SessionManager {
    app_handle: AppHandle,
    state: Arc<RwLock<SessionState>>,
    extension_host: Arc<tokio::sync::Mutex<ExtensionHostManager>>,
    nng_manager: Arc<NngIpcManager>,
    lsp_pool: Arc<RwLock<LspServerPool>>,
    debug_pool: Arc<RwLock<DebugAdapterPool>>,
    extensions_dir: PathBuf,
}

impl SessionManager {
    pub fn new(
        app_handle: AppHandle,
        extensions_dir: PathBuf,
    ) -> Self {
        let state = Arc::new(RwLock::new(SessionState {
            workspace_folders: Vec::new(),
            active_extensions: Vec::new(),
            installed_extensions: Vec::new(),
        })
    });

        let extension_host = Arc::new(tokio::sync::Mutex::new(
            ExtensionHostManager::new("main".to_string())
        ));

        let nng_manager = Arc::new(NngIpcManager::new());

        let lsp_pool = Arc::new(RwLock::new(LspServerPool::new(LspServerStrategy::OnePerLanguage)));
        let debug_pool = Arc::new(RwLock::new(DebugAdapterPool::new()));

        Self {
            app_handle,
            state,
            extension_host,
            nng_manager,
            lsp_pool,
            debug_pool,
            extensions_dir,
        }
    }

    /// Emit an event to the UI
    fn emit_event(&self, event: SessionEvent) {
        if let Err(e) = self.app_handle.emit("session-event", &event) {
            eprintln!("[SessionManager] Failed to emit event: {}", e);
        }
    }

    /// Initialize the session - start Extension Host and load extensions
    pub async fn initialize(&self) -> Result<(), String> {
        println!("[SessionManager] Initializing session...");

        // Start the single Extension Host with NNG IPC
        println!("[SessionManager] Starting Extension Host with NNG IPC...");
        self.start_extension_host().await?;

        // Scan for installed extensions
        self.scan_extensions().await?;

        // Load auto-start extensions
        self.load_auto_start_extensions().await?;

        // Emit initial state
        let state = self.state.read().await.clone();
        self.emit_event(SessionEvent::StateChanged { state });

        println!("[SessionManager] Session initialized");
        Ok(())
    }

    /// Start the Extension Host with bidirectional NNG IPC
    async fn start_extension_host(&self) -> Result<(), String> {
        let session_id = Uuid::new_v4();
        let outgoing_ipc_url = Self::build_ipc_url("ext-out", &session_id)?;
        let incoming_ipc_url = Self::build_ipc_url("ext-in", &session_id)?;

        let session_manager = self.clone_for_handler();
        let handler = Arc::new(move |msg_type: String, payload: serde_json::Value| {
            let sm = session_manager.clone();
            Box::pin(async move { sm.handle_incoming_request(&msg_type, payload).await })
        })
    };

        self
            .nng_manager
            .setup_incoming("main", &incoming_ipc_url, handler)
            .await?;

        let extension_host_main = self.resolve_extension_host_entry()?;
        let extension_host_main_str = extension_host_main.to_string_lossy().to_string();

        let mut manager = self.extension_host.lock().await;
        if manager.is_running() {
            println!("[SessionManager] Extension Host already running");
        } else {
            manager.start_with_nng(
                &extension_host_main_str,
                &outgoing_ipc_url,
                &incoming_ipc_url,
            )?;
        }
        drop(manager);

        const MAX_ATTEMPTS: u8 = 10;
        for attempt in 1..=MAX_ATTEMPTS {
            if self.nng_manager.is_connected("main").await {
                break;
            }

            match self.nng_manager.connect_outgoing("main", &outgoing_ipc_url).await {
                Ok(_) => break,
                Err(err) if attempt == MAX_ATTEMPTS => {
                    return Err(format!("Failed to connect to extension host: {}", err));
                }
                Err(err) => {
                    let backoff = Duration::from_millis((attempt as u64) * 200);
                    println!(
                        "[SessionManager] Outgoing connect attempt {} failed: {}. Retrying in {:?}",
                        attempt, err, backoff
                    );
                    sleep(backoff).await;
                }
            }
        }

        println!("[SessionManager] Extension Host started successfully");
        Ok(())
    }


    /// Clone for handler callback
    fn clone_for_handler(&self) -> Arc<Self> {
        Arc::new(Self {
            app_handle: self.app_handle.clone(),
            state: Arc::clone(&self.state),
            extension_host: Arc::clone(&self.extension_host),
            nng_manager: Arc::clone(&self.nng_manager),
            lsp_pool: Arc::clone(&self.lsp_pool),
            debug_pool: Arc::clone(&self.debug_pool),
            extensions_dir: self.extensions_dir.clone(),
        })
    }

    fn resolve_extension_host_entry(&self) -> Result<PathBuf, String> {
        let mut candidates: Vec<PathBuf> = Vec::new();
        let resolver = self.app_handle.path_resolver();

        if let Some(path) = resolver.resolve_resource("extension-host/dist/main.js") {
            candidates.push(path);
        }

        if let Some(path) = resolver.resolve_resource("extension-host/main.js") {
            candidates.push(path);
        }

        if let Some(parent_dir) = self.extensions_dir.parent() {
            candidates.push(parent_dir.join("extension-host/dist/main.js"));
            candidates.push(parent_dir.join("extension-host/main.js"));
        }

        if let Ok(cwd) = std::env::current_dir() {
            candidates.push(cwd.join("extension-host/dist/main.js"));
            candidates.push(cwd.join("extension-host/main.js"));
        }

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                candidates.push(parent.join("extension-host/dist/main.js"));
            }
        }

        for candidate in candidates {
            if candidate.exists() {
                println!("[SessionManager] Resolved extension host entry to {:?}", candidate);
                return Ok(candidate);
            }
        }

        Err("Unable to locate extension host entry point".to_string())
    }

    fn build_ipc_url(kind: &str, session_id: &Uuid) -> Result<String, String> {
        #[cfg(target_family = "unix")]
        {
            use std::fs;

            let socket_path = std::env::temp_dir().join(format!("dscode-{}-{}.sock", kind, session_id));
            if socket_path.exists() {
                if let Err(err) = fs::remove_file(&socket_path) {
                    eprintln!("[SessionManager] Failed to remove stale socket {:?}: {}", socket_path, err);
                }
            }

            return Ok(format!("ipc://{}", socket_path.to_string_lossy()));
        }

        #[cfg(target_family = "windows")]
        {
            return Ok(format!("ipc://\\.\pipe\dscode-{}-{}", kind, session_id));
        }

        #[allow(unreachable_code)]
        {
            Err("Unsupported platform for IPC".to_string())
        }
    }
    /// Handle incoming requests from Extension Host
    async fn handle_incoming_request(&self, msg_type: &str, payload: serde_json::Value) -> Result<serde_json::Value, String> {
        println!("[SessionManager] Handling incoming request: {}", msg_type);

        match msg_type {
            "get-extensions-dir" => {
                let extensions_dir = self.extensions_dir.canonicalize()
                    .unwrap_or_else(|_| self.extensions_dir.clone());
                println!("[SessionManager] Extensions directory: {:?}", extensions_dir);
                Ok(serde_json::json!(extensions_dir.to_string_lossy().to_string()))
            },
            "workspace-get-folders" => {
                let state = self.state.read().await;
                let folders: Vec<String> = state.workspace_folders.iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                Ok(serde_json::json!(folders))
            },
            _ => {
                println!("[SessionManager] Unhandled request type: {}", msg_type);
                Err(format!("Unhandled request type: {}", msg_type))
            }
        }
    }


    /// Scan extensions directory for installed extensions
    async fn scan_extensions(&self) -> Result<(), String> {
        println!("[SessionManager] Scanning extensions directory...");

        if !self.extensions_dir.exists() {
            std::fs::create_dir_all(&self.extensions_dir)
                .map_err(|e| format!("Failed to create extensions directory: {}", e))?;
        }

        let entries = std::fs::read_dir(&self.extensions_dir)
            .map_err(|e| format!("Failed to read extensions directory: {}", e))?;

        let mut extensions = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                match self.read_extension_manifest(&path) {
                    Ok(info) => extensions.push(info),
                    Err(e) => eprintln!("[SessionManager] Failed to read extension manifest at {:?}: {}", path, e),
                }
            }
        }

        let mut state = self.state.write().await;
        state.installed_extensions = extensions.clone();
        drop(state);

        println!("[SessionManager] Found {} installed extensions", extensions.len());

        self.emit_event(SessionEvent::ExtensionsChanged { extensions });

        Ok(())
    }

    /// Read extension manifest (package.json)
    fn read_extension_manifest(&self, extension_path: &PathBuf) -> Result<ExtensionInfo, String> {
        let manifest_path = extension_path.join("package.json");

        let content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;

        let manifest: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?;

        let name = manifest.get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'name' field")?
            .to_string();

        let version = manifest.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();

        let publisher = manifest.get("publisher")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let description = manifest.get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let extension_id = format!("{}.{}", publisher, name);

        Ok(ExtensionInfo {
            id: extension_id,
            name,
            version,
            publisher,
            description,
            enabled: true,
            active: false,
        })
    }
    }

    /// Load extensions that should auto-start
    async fn load_auto_start_extensions(&self) -> Result<(), String> {
        let state = self.state.read().await;
        let extensions: Vec<_> = state.installed_extensions
            .iter()
            .filter(|ext| ext.enabled)
            .cloned()
            .collect();
        drop(state);

        for ext_info in extensions {
            if let Err(e) = self.load_extension_internal(&ext_info.id).await {
                eprintln!("[SessionManager] Failed to load extension {}: {}", ext_info.id, e);
            }
        }

        Ok(())
    }

    /// Load an extension
    pub async fn load_extension(&self, extension_id: &str) -> Result<(), String> {
        self.load_extension_internal(extension_id).await?;

        // Update state and emit event
        self.mark_extension_active(extension_id, true).await;
        self.emit_event(SessionEvent::ExtensionLoaded {
            extension_id: extension_id.to_string()
        })
    };

        Ok(())
    }

    /// Internal extension loading
    async fn load_extension_internal(&self, extension_id: &str) -> Result<(), String> {
        let extension_path = self.extensions_dir.join(extension_id);

        if !extension_path.exists() {
            return Err(format!("Extension directory not found: {:?}", extension_path));
        }

        println!("[SessionManager] Loading extension: {}", extension_id);

        // Send activate-extension request to Extension Host via NNG
        let payload = serde_json::json!({ "extensionId": extension_id });
        self.nng_manager.request("main", "activate-extension", payload).await?;

        Ok(())
    }

    /// Unload an extension
    pub async fn unload_extension(&self, extension_id: &str) -> Result<(), String> {
        println!("[SessionManager] Unloading extension: {}", extension_id);

        // Send deactivate-extension request to Extension Host via NNG
        let payload = serde_json::json!({ "extensionId": extension_id });
        self.nng_manager.request("main", "deactivate-extension", payload).await?;

        // Update state and emit event
        self.mark_extension_active(extension_id, false).await;
        self.emit_event(SessionEvent::ExtensionUnloaded {
            extension_id: extension_id.to_string()
        })
    };

        Ok(())
    }

    /// Delete an extension
    pub async fn delete_extension(&self, extension_id: &str) -> Result<(), String> {
        println!("[SessionManager] Deleting extension: {}", extension_id);

        // Unload if active
        let _ = self.unload_extension(extension_id).await;

        // Delete from filesystem
        let extension_path = self.extensions_dir.join(extension_id);
        if extension_path.exists() {
            std::fs::remove_dir_all(&extension_path)
                .map_err(|e| format!("Failed to delete extension directory: {}", e))?;
        }

        // Remove from state
        let mut state = self.state.write().await;
        state.installed_extensions.retain(|ext| ext.id != extension_id);
        state.active_extensions.retain(|ext| ext.id != extension_id);
        let extensions = state.installed_extensions.clone();
        drop(state);

        // Emit events
        self.emit_event(SessionEvent::ExtensionDeleted {
            extension_id: extension_id.to_string()
        })
    };
        self.emit_event(SessionEvent::ExtensionsChanged { extensions });

        Ok(())
    }

    /// Mark extension as active/inactive in state
    async fn mark_extension_active(&self, extension_id: &str, active: bool) {
        let mut state = self.state.write().await;

        // Update in installed extensions list
        if let Some(ext) = state.installed_extensions.iter_mut().find(|e| e.id == extension_id) {
            ext.active = active;
        }

        // Update active extensions list
        if active {
            if let Some(ext) = state.installed_extensions.iter().find(|e| e.id == extension_id) {
                let ext_clone = ext.clone();
                if !state.active_extensions.iter().any(|e| e.id == extension_id) {
                    state.active_extensions.push(ext_clone);
                }
            }
        } else {
            state.active_extensions.retain(|e| e.id != extension_id);
        }

        let state_clone = state.clone();
        drop(state);

        // Emit state change
        self.emit_event(SessionEvent::StateChanged { state: state_clone });
    }

    /// Get current session state
    pub async fn get_state(&self) -> SessionState {
        self.state.read().await.clone()
    }

    /// Get list of installed extensions
    pub async fn get_installed_extensions(&self) -> Vec<ExtensionInfo> {
        self.state.read().await.installed_extensions.clone()
    }

    /// Get list of active extensions
    pub async fn get_active_extensions(&self) -> Vec<ExtensionInfo> {
        self.state.read().await.active_extensions.clone()
    }

    /// Add workspace folder
    pub async fn add_workspace_folder(&self, path: PathBuf) -> Result<(), String> {
        let mut state = self.state.write().await;

        if !state.workspace_folders.contains(&path) {
            state.workspace_folders.push(path.clone());
            drop(state);

            self.emit_event(SessionEvent::WorkspaceFolderAdded { path });
        }

        Ok(())
    }

    /// Remove workspace folder
    pub async fn remove_workspace_folder(&self, path: &PathBuf) -> Result<(), String> {
        let mut state = self.state.write().await;
        state.workspace_folders.retain(|p| p != path);
        drop(state);

        self.emit_event(SessionEvent::WorkspaceFolderRemoved { path: path.clone() });

        Ok(())
    }

    /// Shutdown session
    pub async fn shutdown(&self) -> Result<(), String> {
        println!("[SessionManager] Shutting down session...");

        // Unload all extensions
        let active = self.get_active_extensions().await;
        for ext in active {
            let _ = self.unload_extension(&ext.id).await;
        }

        // Shutdown Extension Host
        let mut manager = self.extension_host.lock().await;
        manager.shutdown();

        println!("[SessionManager] Session shutdown complete");
        Ok(())
    }

    /// Get reference to NNG manager
    pub fn nng_manager(&self) -> &Arc<NngIpcManager> {
        &self.nng_manager
    }

    /// Get reference to LSP pool
    pub fn lsp_pool(&self) -> &Arc<RwLock<LspServerPool>> {
        &self.lsp_pool
    }

    /// Get reference to debug pool
    pub fn debug_pool(&self) -> &Arc<RwLock<DebugAdapterPool>> {
        &self.debug_pool
    }
}
