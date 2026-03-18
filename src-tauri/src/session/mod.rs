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
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Emitter};

use crate::extension_host::{ExtensionHostPool, HostAllocationStrategy};
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
    extension_pool: Arc<ExtensionHostPool>,
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
        }));

        let extension_pool = Arc::new(ExtensionHostPool::new(
            HostAllocationStrategy::Shared { max_extensions_per_host: 10 }
        ));

        let lsp_pool = Arc::new(RwLock::new(LspServerPool::new(LspServerStrategy::OnePerLanguage)));
        let debug_pool = Arc::new(RwLock::new(DebugAdapterPool::new()));

        Self {
            app_handle,
            state,
            extension_pool,
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

    /// Initialize the session - scan and load extensions
    pub async fn initialize(&self) -> Result<(), String> {
        println!("[SessionManager] Initializing session...");

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
        });

        Ok(())
    }

    /// Internal extension loading
    async fn load_extension_internal(&self, extension_id: &str) -> Result<(), String> {
        let extension_path = self.extensions_dir.join(extension_id);

        if !extension_path.exists() {
            return Err(format!("Extension directory not found: {:?}", extension_path));
        }

        println!("[SessionManager] Loading extension: {}", extension_id);

        // Load into extension host pool
        self.extension_pool.load_extension(
            extension_id,
            extension_path.to_str().ok_or("Invalid path")?,
            None, // workspace
        ).await?;

        Ok(())
    }

    /// Unload an extension
    pub async fn unload_extension(&self, extension_id: &str) -> Result<(), String> {
        println!("[SessionManager] Unloading extension: {}", extension_id);

        // Unload from extension host pool
        self.extension_pool.unload_extension(extension_id).await?;

        // Update state and emit event
        self.mark_extension_active(extension_id, false).await;
        self.emit_event(SessionEvent::ExtensionUnloaded {
            extension_id: extension_id.to_string()
        });

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
        });
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

        // Shutdown extension host pool
        self.extension_pool.shutdown_all().await?;

        println!("[SessionManager] Session shutdown complete");
        Ok(())
    }

    /// Get reference to extension pool
    pub fn extension_pool(&self) -> &Arc<ExtensionHostPool> {
        &self.extension_pool
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
