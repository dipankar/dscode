//! IPC dispatch from the extension host.
//!
//! This module handles incoming requests from the extension host process,
//! translating them into session actions and UI events.
//!
//! **This module requires the `tauri` feature flag.**

use crate::types::{SessionEvent, SessionLifecycle};
use crate::workspace::{FileDecoration, FileDecorationProvider};
use dscode_extension_host::{IpcManager, PathValidator, SecretStorage};
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;

#[allow(dead_code)]
fn classify_io_error(error: &io::Error) -> &'static str {
    match error.kind() {
        io::ErrorKind::NotFound => "EntryNotFound",
        io::ErrorKind::PermissionDenied => "NoPermissions",
        io::ErrorKind::AlreadyExists => "EntryExists",
        io::ErrorKind::IsADirectory => "EntryIsADirectory",
        io::ErrorKind::NotADirectory => "EntryNotADirectory",
        _ => "Unavailable",
    }
}

#[allow(dead_code)]
fn fs_error(code: &str, message: &str) -> String {
    format!("{}: {}", code, message)
}

/// Tauri-backed event emitter using `AppHandle::emit`.
#[allow(dead_code)]
pub struct TauriEventEmitter {
    app_handle: AppHandle,
}

impl TauriEventEmitter {
    #[allow(dead_code)]
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl crate::types::EventEmitter for TauriEventEmitter {
    fn emit_event(&self, event: &SessionEvent) {
        if let Err(e) = self.app_handle.emit("session-event", event) {
            tracing::warn!("Failed to emit event: {}", e);
        }
    }
}

/// Central session manager with Tauri integration.
///
/// This struct coordinates all session state and communicates with the
/// extension host, LSP/DAP servers, and the UI via Tauri events.
#[allow(dead_code)]
pub struct SessionManager {
    app_handle: AppHandle,
    state: Arc<tokio::sync::RwLock<crate::types::SessionState>>,
    extension_host: Arc<tokio::sync::Mutex<dscode_extension_host::ExtensionHostManager>>,
    ipc_manager: Arc<IpcManager>,
    lsp_pool: Arc<tokio::sync::RwLock<dscode_lsp::LspServerPool>>,
    debug_pool: Arc<tokio::sync::RwLock<dscode_dap::DebugAdapterPool>>,
    app_dirs: dscode_core::AppDirectories,
    command_map: Arc<tokio::sync::RwLock<HashMap<String, Vec<String>>>>,
    status_bar_items: Arc<tokio::sync::RwLock<HashMap<String, crate::types::StatusBarItemState>>>,
    pending_message_requests:
        Arc<tokio::sync::RwLock<HashMap<String, oneshot::Sender<Option<String>>>>>,
    pending_quick_pick_requests:
        Arc<tokio::sync::RwLock<HashMap<String, oneshot::Sender<Option<Value>>>>>,
    pending_input_requests:
        Arc<tokio::sync::RwLock<HashMap<String, oneshot::Sender<Option<String>>>>>,
    configuration: Arc<tokio::sync::RwLock<crate::configuration::ConfigurationStore>>,
    document_versions: Arc<tokio::sync::RwLock<HashMap<String, i32>>>,
    editor_decorations: Arc<tokio::sync::RwLock<HashMap<String, HashMap<String, Value>>>>,
    decoration_types: Arc<tokio::sync::RwLock<HashMap<String, Value>>>,
    extension_watchers: Arc<tokio::sync::Mutex<HashMap<String, notify::RecommendedWatcher>>>,
    workspace_configurations:
        Arc<tokio::sync::RwLock<HashMap<String, crate::workspace::WorkspaceConfiguration>>>,
    file_decoration_providers: Arc<tokio::sync::RwLock<Vec<FileDecorationProvider>>>,
    file_decorations: Arc<tokio::sync::RwLock<HashMap<String, Vec<FileDecoration>>>>,
    extension_host_ready: Arc<tokio::sync::Notify>,
    initialized: Arc<tokio::sync::OnceCell<()>>,
    lifecycle: Arc<tokio::sync::RwLock<SessionLifecycle>>,
    secrets: Arc<SecretStorage>,
    path_validator: Arc<tokio::sync::RwLock<PathValidator>>,
    outgoing_socket: Arc<tokio::sync::RwLock<Option<String>>>,
    incoming_socket: Arc<tokio::sync::RwLock<Option<String>>>,
    extension_host_entry: Arc<tokio::sync::RwLock<Option<String>>>,
}

#[allow(dead_code)]
impl SessionManager {
    const CORE_COMMAND_OWNER: &'static str = "__core__";
    const OUTPUT_CHANNEL_MAX_LINES: usize = 2000;

    /// Create a new session manager.
    pub fn new(
        app_handle: AppHandle,
        app_dirs: dscode_core::AppDirectories,
        path_validator: Arc<tokio::sync::RwLock<PathValidator>>,
    ) -> Self {
        use crate::types::SessionState;
        use dscode_extension_host::ExtensionHostManager;
        use dscode_lsp::LspServerStrategy;

        let state = Arc::new(tokio::sync::RwLock::new(SessionState {
            workspace_folders: Vec::new(),
            active_extensions: Vec::new(),
            installed_extensions: Vec::new(),
            available_commands: Vec::new(),
            status_bar_items: Vec::new(),
        }));

        let extension_host = Arc::new(tokio::sync::Mutex::new(ExtensionHostManager::new(
            "main".to_string(),
        )));

        let ipc_manager = Arc::new(IpcManager::new());

        let lsp_pool = Arc::new(tokio::sync::RwLock::new(dscode_lsp::LspServerPool::new(
            LspServerStrategy::OnePerLanguage,
        )));
        let debug_pool = Arc::new(tokio::sync::RwLock::new(dscode_dap::DebugAdapterPool::new()));

        let configuration_store =
            match crate::configuration::ConfigurationStore::default_in_dir(&app_dirs.storage_dir) {
                Ok(store) => store,
                Err(err) => {
                    tracing::warn!("Failed to load configuration: {}", err);
                    crate::configuration::ConfigurationStore::empty(
                        app_dirs.storage_dir.join("settings.json"),
                    )
                }
            };

        Self {
            app_handle,
            state,
            extension_host,
            ipc_manager,
            lsp_pool,
            debug_pool,
            app_dirs,
            command_map: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            status_bar_items: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            pending_message_requests: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            pending_quick_pick_requests: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            pending_input_requests: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            configuration: Arc::new(tokio::sync::RwLock::new(configuration_store)),
            document_versions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            editor_decorations: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            decoration_types: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            extension_watchers: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            workspace_configurations: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            file_decoration_providers: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            file_decorations: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            extension_host_ready: Arc::new(tokio::sync::Notify::new()),
            initialized: Arc::new(tokio::sync::OnceCell::new()),
            lifecycle: Arc::new(tokio::sync::RwLock::new(SessionLifecycle::Uninitialized)),
            secrets: Arc::new(SecretStorage::new()),
            path_validator,
            outgoing_socket: Arc::new(tokio::sync::RwLock::new(None)),
            incoming_socket: Arc::new(tokio::sync::RwLock::new(None)),
            extension_host_entry: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Emit an event to the UI via Tauri.
    #[allow(dead_code)]
    fn emit_event(&self, event: SessionEvent) {
        if let Err(e) = self.app_handle.emit("session-event", &event) {
            tracing::warn!("Failed to emit event: {}", e);
        }
    }
}
