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

mod configuration;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::{RwLock, oneshot};
use tokio::time::{sleep, Duration};
use serde::{Serialize, Deserialize};
use serde_json::{Value, json, Map};
use uuid::Uuid;
use tauri::{AppHandle, Emitter, Manager};
use tauri::path::BaseDirectory;
use std::io::ErrorKind;
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use notify::{RecommendedWatcher, Watcher, RecursiveMode, Event, EventKind};
use notify::event::{CreateKind, ModifyKind, RemoveKind};
use ignore::WalkBuilder;

use crate::extension_host::{ExtensionHostManager, NngIpcManager, IncomingRequestHandler};
use crate::config::AppDirectories;
use crate::lsp::{LspServerPool, LspServerStrategy};
use crate::debug::DebugAdapterPool;
use configuration::ConfigurationStore;

/// Application session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub workspace_folders: Vec<PathBuf>,
    pub active_extensions: Vec<ExtensionInfo>,
    pub installed_extensions: Vec<ExtensionInfo>,
    pub available_commands: Vec<String>,
    pub status_bar_items: Vec<StatusBarItemState>,
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
    pub categories: Vec<String>,
    pub dependencies: Vec<String>,
    pub repository: Option<String>,
    pub activation_events: Vec<String>,
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarItemState {
    pub id: String,
    pub owner: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<StatusBarCommand>,
    pub alignment: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarCommand {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PositionPayload {
    line: usize,
    character: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RangePayload {
    start: PositionPayload,
    end: PositionPayload,
}

impl RangePayload {
    fn from_position(pos: PositionPayload) -> Self {
        Self {
            start: pos.clone(),
            end: pos,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct TextEditPayload {
    range: RangePayload,
    #[serde(rename = "newText")]
    new_text: String,
}

#[derive(Debug, Clone)]
struct StatusMessageEntry {
    text: String,
}

#[derive(Debug, Clone)]
struct OutputChannelEntry {
    name: String,
    lines: Vec<String>,
    visible: bool,
}

#[derive(Debug, Clone)]
struct StatusBarEntry {
    id: String,
    text: String,
    tooltip: Option<String>,
    color: Option<String>,
    command: Option<StatusBarCommand>,
    alignment: StatusBarAlignment,
    priority: Option<f64>,
    visible: bool,
    owner: String,
}

impl StatusBarEntry {
    fn new(id: String, owner: String) -> Self {
        Self {
            id,
            text: String::new(),
            tooltip: None,
            color: None,
            command: None,
            alignment: StatusBarAlignment::Left,
            priority: None,
            visible: false,
            owner,
        }
    }

    fn to_state(&self) -> StatusBarItemState {
        StatusBarItemState {
            id: self.id.clone(),
            owner: self.owner.clone(),
            text: self.text.clone(),
            tooltip: self.tooltip.clone(),
            color: self.color.clone(),
            command: self.command.clone(),
            alignment: self.alignment.as_str().to_string(),
            priority: self.priority,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatusBarAlignment {
    Left,
    Right,
}

impl StatusBarAlignment {
    fn from_i32(value: i32) -> Self {
        match value {
            2 => StatusBarAlignment::Right,
            _ => StatusBarAlignment::Left,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            StatusBarAlignment::Left => "left",
            StatusBarAlignment::Right => "right",
        }
    }

    fn sort_value(&self) -> i32 {
        match self {
            StatusBarAlignment::Left => 0,
            StatusBarAlignment::Right => 1,
        }
    }
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

    /// Available commands changed
    CommandsChanged { commands: Vec<String> },

    /// Status bar items changed
    StatusBarItems { items: Vec<StatusBarItemState> },

    /// Window message request
    WindowMessage { level: String, message: String, actions: Option<Vec<String>> },

    /// Window message with actionable items (awaiting user response)
    WindowActionRequest { id: String, level: String, message: String, actions: Vec<String> },

    /// Status bar transient message shown
    StatusBarMessageShown { id: String, text: String },

    /// Status bar transient message cleared
    StatusBarMessageCleared { id: String },

    /// Output channel registered
    OutputChannelRegistered { channel: String },

    /// Output channel received new content
    OutputChannelAppended { channel: String, value: String },

    /// Output channel cleared
    OutputChannelCleared { channel: String },

    /// Output channel disposed
    OutputChannelDisposed { channel: String },

    /// Output channel visibility toggled
    OutputChannelVisibility { channel: String, visible: bool },

    /// Tree view reveal request from extension host
    TreeViewReveal { view_id: String, element: Value, options: Value },

    /// Configuration changed
    ConfigurationChanged { section: Option<String>, key: Option<String> },

    /// Document content changed
    DocumentChanged { path: String, content: String },

    /// Editor decorations updated
    EditorDecorations { uri: String, key: String, decorations: Value },

    /// Quick pick selection required
    QuickPickRequest {
        id: String,
        items: Vec<Value>,
        can_pick_many: bool,
        place_holder: Option<String>,
        title: Option<String>,
        match_on_description: bool,
        match_on_detail: bool,
    },

    /// Input box value requested
    InputBoxRequest {
        id: String,
        prompt: Option<String>,
        place_holder: Option<String>,
        value: Option<String>,
        password: bool,
        value_selection: Option<(usize, usize)>,
    },
}

/// Central session manager
pub struct SessionManager {
    app_handle: AppHandle,
    state: Arc<RwLock<SessionState>>,
    extension_host: Arc<tokio::sync::Mutex<ExtensionHostManager>>,
    nng_manager: Arc<NngIpcManager>,
    lsp_pool: Arc<RwLock<LspServerPool>>,
    debug_pool: Arc<RwLock<DebugAdapterPool>>,
    app_dirs: AppDirectories,
    command_map: Arc<RwLock<HashMap<String, Vec<String>>>>,
    status_bar_items: Arc<RwLock<HashMap<String, StatusBarEntry>>>,
    pending_message_requests: Arc<RwLock<HashMap<String, oneshot::Sender<Option<String>>>>>,
    pending_quick_pick_requests: Arc<RwLock<HashMap<String, oneshot::Sender<Option<Value>>>>>,
    pending_input_requests: Arc<RwLock<HashMap<String, oneshot::Sender<Option<String>>>>>,
    status_messages: Arc<RwLock<HashMap<String, StatusMessageEntry>>>,
    output_channels: Arc<RwLock<HashMap<String, OutputChannelEntry>>>,
    active_output_channel: Arc<RwLock<Option<String>>>,
    configuration: Arc<RwLock<ConfigurationStore>>,
    document_versions: Arc<RwLock<HashMap<String, i32>>>,
    editor_decorations: Arc<RwLock<HashMap<String, HashMap<String, Value>>>>,
    decoration_types: Arc<RwLock<HashMap<String, Value>>>,
    extension_watchers: Arc<Mutex<HashMap<String, RecommendedWatcher>>>,
}

impl SessionManager {
    const CORE_COMMAND_OWNER: &'static str = "__core__";
    const OUTPUT_CHANNEL_MAX_LINES: usize = 2000;

    pub fn new(
        app_handle: AppHandle,
        app_dirs: AppDirectories,
    ) -> Self {
        let state = Arc::new(RwLock::new(SessionState {
            workspace_folders: Vec::new(),
            active_extensions: Vec::new(),
            installed_extensions: Vec::new(),
            available_commands: Vec::new(),
            status_bar_items: Vec::new(),
        }));

        let extension_host = Arc::new(tokio::sync::Mutex::new(
            ExtensionHostManager::new("main".to_string())
        ));

        let nng_manager = Arc::new(NngIpcManager::new());

        let lsp_pool = Arc::new(RwLock::new(LspServerPool::new(LspServerStrategy::OnePerLanguage)));
        let debug_pool = Arc::new(RwLock::new(DebugAdapterPool::new()));

        let configuration_store = match ConfigurationStore::default_in_dir(&app_dirs.storage_dir) {
            Ok(store) => store,
            Err(err) => {
                eprintln!("[SessionManager] Failed to load configuration: {}", err);
                ConfigurationStore::empty(app_dirs.storage_dir.join("settings.json"))
            }
        };

        Self {
            app_handle,
            state,
            extension_host,
            nng_manager,
            lsp_pool,
            debug_pool,
            app_dirs,
            command_map: Arc::new(RwLock::new(HashMap::new())),
            status_bar_items: Arc::new(RwLock::new(HashMap::new())),
            pending_message_requests: Arc::new(RwLock::new(HashMap::new())),
            pending_quick_pick_requests: Arc::new(RwLock::new(HashMap::new())),
            pending_input_requests: Arc::new(RwLock::new(HashMap::new())),
            status_messages: Arc::new(RwLock::new(HashMap::new())),
            output_channels: Arc::new(RwLock::new(HashMap::new())),
            active_output_channel: Arc::new(RwLock::new(None)),
            configuration: Arc::new(RwLock::new(configuration_store)),
            document_versions: Arc::new(RwLock::new(HashMap::new())),
            editor_decorations: Arc::new(RwLock::new(HashMap::new())),
            decoration_types: Arc::new(RwLock::new(HashMap::new())),
            extension_watchers: Arc::new(Mutex::new(HashMap::new())),
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
        let handler: IncomingRequestHandler = Arc::new(move |msg_type: String, payload: serde_json::Value| {
            let sm = session_manager.clone();
            Box::pin(async move { sm.handle_incoming_request(&msg_type, payload).await })
        });

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
            app_dirs: self.app_dirs.clone(),
            command_map: Arc::clone(&self.command_map),
            status_bar_items: Arc::clone(&self.status_bar_items),
            pending_message_requests: Arc::clone(&self.pending_message_requests),
            pending_quick_pick_requests: Arc::clone(&self.pending_quick_pick_requests),
            pending_input_requests: Arc::clone(&self.pending_input_requests),
            status_messages: Arc::clone(&self.status_messages),
            output_channels: Arc::clone(&self.output_channels),
            active_output_channel: Arc::clone(&self.active_output_channel),
            configuration: Arc::clone(&self.configuration),
            document_versions: Arc::clone(&self.document_versions),
            editor_decorations: Arc::clone(&self.editor_decorations),
            decoration_types: Arc::clone(&self.decoration_types),
            extension_watchers: Arc::clone(&self.extension_watchers),
        })
    }

    fn resolve_extension_host_entry(&self) -> Result<PathBuf, String> {
        let mut candidates: Vec<PathBuf> = Vec::new();
        let resolver = self.app_handle.path();

        if let Ok(path) = resolver.resolve("extension-host/dist/main.js", BaseDirectory::Resource) {
            candidates.push(path);
        }

        if let Ok(path) = resolver.resolve("extension-host/main.js", BaseDirectory::Resource) {
            candidates.push(path);
        }

        if let Some(parent_dir) = self.app_dirs.extensions_dir.parent() {
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
            return Ok(format!(r"ipc://\\.\pipe\dscode-{}-{}", kind, session_id));
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
                let extensions_dir = self.app_dirs.extensions_dir.canonicalize()
                    .unwrap_or_else(|_| self.app_dirs.extensions_dir.clone());
                println!("[SessionManager] Extensions directory: {:?}", extensions_dir);
                Ok(serde_json::json!(extensions_dir.to_string_lossy().to_string()))
            },
            "get-extension-storage" => {
                let extension_id = payload.get("extensionId")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing extensionId in request")?;

                let storage_root = self.app_dirs.storage_dir.join(extension_id);
                let global_dir = storage_root.join("global");
                let workspace_dir = storage_root.join("workspace");
                let logs_dir = self.app_dirs.logs_dir.join(extension_id);

                if let Err(e) = std::fs::create_dir_all(&global_dir) {
                    if e.kind() != std::io::ErrorKind::AlreadyExists {
                        return Err(format!("Failed to prepare global storage for {}: {}", extension_id, e));
                    }
                }
                if let Err(e) = std::fs::create_dir_all(&workspace_dir) {
                    if e.kind() != std::io::ErrorKind::AlreadyExists {
                        return Err(format!("Failed to prepare workspace storage for {}: {}", extension_id, e));
                    }
                }
                if let Err(e) = std::fs::create_dir_all(&logs_dir) {
                    if e.kind() != std::io::ErrorKind::AlreadyExists {
                        return Err(format!("Failed to prepare logs directory for {}: {}", extension_id, e));
                    }
                }

                Ok(serde_json::json!({
                    "global": global_dir.to_string_lossy().to_string(),
                    "workspace": workspace_dir.to_string_lossy().to_string(),
                    "logs": logs_dir.to_string_lossy().to_string(),
                }))
            },
            "registerTreeDataProvider" => {
                Ok(json!({"success": true}))
            },
            "updateStatusBarItem" => {
                let id = payload.get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing status bar item id".to_string())?;
                let owner = payload
                    .get("owner")
                    .and_then(|v| v.as_str())
                    .unwrap_or(Self::CORE_COMMAND_OWNER);

                self.upsert_status_bar_item(id, owner, &payload).await?;
                Ok(json!({"success": true}))
            },
            "hideStatusBarItem" => {
                let id = payload.get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing status bar item id".to_string())?;
                let owner = payload
                    .get("owner")
                    .and_then(|v| v.as_str())
                    .unwrap_or(Self::CORE_COMMAND_OWNER);

                self.hide_status_bar_item(owner, id).await?;
                Ok(json!({"success": true}))
            },
            "disposeStatusBarItem" => {
                let id = payload.get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing status bar item id".to_string())?;
                let owner = payload
                    .get("owner")
                    .and_then(|v| v.as_str())
                    .unwrap_or(Self::CORE_COMMAND_OWNER);

                self.dispose_status_bar_item(owner, id).await?;
                Ok(json!({"success": true}))
            },
            "command-registered" => {
                if let Some(command) = payload.get("command").and_then(|v| v.as_str()) {
                    let owner = payload
                        .get("owner")
                        .and_then(|v| v.as_str())
                        .unwrap_or(Self::CORE_COMMAND_OWNER);

                    self.add_command_owner(command, owner).await;
                    self.publish_command_list().await;

                    Ok(serde_json::json!({"success": true}))
                } else {
                    Err("Missing command name".to_string())
                }
            },
            "command-unregistered" => {
                if let Some(command) = payload.get("command").and_then(|v| v.as_str()) {
                    let owner = payload
                        .get("owner")
                        .and_then(|v| v.as_str())
                        .unwrap_or(Self::CORE_COMMAND_OWNER);

                    self.remove_command_owner(command, owner).await;
                    self.publish_command_list().await;

                    Ok(serde_json::json!({"success": true}))
                } else {
                    Err("Missing command name".to_string())
                }
            },
            "window-show-message" => {
                let message = payload.get("message").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let level = payload.get("type").and_then(|v| v.as_str()).unwrap_or("info").to_string();
                self.emit_event(SessionEvent::WindowMessage {
                    level,
                    message,
                    actions: None,
                });
                Ok(json!({"success": true}))
            },
            "window-show-message-with-actions" => {
                let message = payload.get("message").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let level = payload.get("type").and_then(|v| v.as_str()).unwrap_or("info").to_string();
                let actions_vec: Vec<String> = payload.get("actions")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|item| item.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                let has_actions = !actions_vec.is_empty();

                self.emit_event(SessionEvent::WindowMessage {
                    level: level.clone(),
                    message: message.clone(),
                    actions: if has_actions { Some(actions_vec.clone()) } else { None },
                });

                if !has_actions {
                    return Ok(json!({"action": null}));
                }

                let (tx, rx) = oneshot::channel();
                let request_id = Uuid::new_v4().to_string();
                let request_key = request_id.clone();
                {
                    let mut pending = self.pending_message_requests.write().await;
                    pending.insert(request_id.clone(), tx);
                }

                self.emit_event(SessionEvent::WindowActionRequest {
                    id: request_id,
                    level: level.clone(),
                    message: message.clone(),
                    actions: actions_vec.clone(),
                });

                let result = match rx.await {
                    Ok(action) => Ok(json!({"action": action})),
                    Err(_) => Ok(json!({"action": null})),
                };

                {
                    let mut pending = self.pending_message_requests.write().await;
                    pending.remove(&request_key);
                }

                result
            },
            "window-set-status-bar-message" => {
                let id = payload
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let text = payload
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let timeout_ms = payload
                    .get("timeout")
                    .and_then(|v| v.as_f64())
                    .map(|val| val.max(0.0).round() as u64);

                self.show_status_bar_message(&id, &text).await?;

                if let Some(ms) = timeout_ms {
                    if ms > 0 {
                        let session = self.clone_for_handler();
                        let id_clone = id.clone();
                        tokio::spawn(async move {
                            sleep(Duration::from_millis(ms)).await;
                            let _ = session.clear_status_bar_message(&id_clone).await;
                        });
                    }
                }

                Ok(json!({"success": true}))
            },
            "window-clear-status-bar-message" => {
                if let Some(id) = payload.get("id").and_then(|v| v.as_str()) {
                    self.clear_status_bar_message(id).await?;
                }
                Ok(json!({"success": true}))
            },
            "openTextDocument" => {
                let path = payload
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing document path".to_string())?;
                self.open_text_document(path).await
            },
            "saveDocument" => {
                let path = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing document uri".to_string())?;
                if let Some(content) = payload.get("content").and_then(|v| v.as_str()) {
                    let version = self.persist_document(path, content).await?;
                    Ok(json!({ "success": true, "version": version, "content": content }))
                } else {
                    Ok(json!({ "success": true }))
                }
            },
            "applyEdits" => {
                let path = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing document uri".to_string())?;
                let edits_value = payload
                    .get("edits")
                    .cloned()
                    .ok_or_else(|| "Missing edits payload".to_string())?;
                let mut edits: Vec<TextEditPayload> = serde_json::from_value(edits_value)
                    .map_err(|e| format!("Invalid edits payload: {}", e))?;
                let end_of_line = payload
                    .get("endOfLine")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);

                let original = fs::read_to_string(path)
                    .map_err(|e| format!("Failed to read document {}: {}", path, e))?;
                let updated = Self::apply_text_edits(&original, &mut edits, end_of_line)?;
                let version = self.persist_document(path, &updated).await?;

                Ok(json!({ "success": true, "version": version, "content": updated }))
            },
            "insertSnippet" => {
                let path = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing document uri".to_string())?;
                let snippet = payload
                    .get("snippet")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let locations = Self::parse_snippet_locations(payload.get("location"));

                let plain = Self::snippet_to_plain(snippet);
                let mut edits: Vec<TextEditPayload> = locations
                    .into_iter()
                    .map(|range| TextEditPayload {
                        range,
                        new_text: plain.clone(),
                    })
                    .collect();

                let original = fs::read_to_string(path)
                    .map_err(|e| format!("Failed to read document {}: {}", path, e))?;
                let updated = Self::apply_text_edits(&original, &mut edits, None)?;
                let version = self.persist_document(path, &updated).await?;

                Ok(json!({ "success": true, "version": version, "content": updated }))
            },
            "setDecorations" => {
                let uri = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing decoration uri".to_string())?;
                let key = payload
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing decoration key".to_string())?;
                let decorations = payload
                    .get("decorations")
                    .cloned()
                    .unwrap_or(Value::Array(Vec::new()));

                self.update_decorations(uri, key, decorations).await?;
                Ok(json!({"success": true}))
            },
            "disposeDecorationType" => {
                let key = payload
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing decoration key".to_string())?;
                {
                    let mut types = self.decoration_types.write().await;
                    types.remove(key);
                }
                self.dispose_decorations(key).await?;
                Ok(json!({"success": true}))
            },
            "registerDecorationType" => {
                let key = payload
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing decoration key".to_string())?;
                let options = payload.get("options").cloned().unwrap_or(Value::Object(Map::new()));
                {
                    let mut types = self.decoration_types.write().await;
                    types.insert(key.to_string(), options);
                }
                Ok(json!({"success": true}))
            },
            "workspace-register-watcher" => {
                let id = payload
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing watcher id".to_string())?;
                let pattern = payload
                    .get("globPattern")
                    .and_then(|v| v.as_str())
                    .unwrap_or("**/*");
                let ignore_create = payload
                    .get("ignoreCreateEvents")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let ignore_change = payload
                    .get("ignoreChangeEvents")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let ignore_delete = payload
                    .get("ignoreDeleteEvents")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                self.register_file_system_watcher(id, pattern, ignore_create, ignore_change, ignore_delete).await?;
                Ok(json!({"success": true}))
            },
            "workspace-unregister-watcher" => {
                if let Some(id) = payload.get("id").and_then(|v| v.as_str()) {
                    self.unregister_file_system_watcher(id);
                }
                Ok(json!({"success": true}))
            },
            "revealTreeItem" => {
                let view_id = payload
                    .get("viewId")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let element = payload.get("element").cloned().unwrap_or(Value::Null);
                let options = payload.get("options").cloned().unwrap_or(Value::Null);

                self.emit_event(SessionEvent::TreeViewReveal {
                    view_id,
                    element,
                    options,
                });

                Ok(json!({"success": true}))
            },
            "window-show-quick-pick" => {
                let items = payload
                    .get("items")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();

                let options = payload
                    .get("options")
                    .and_then(|v| v.as_object())
                    .cloned()
                    .unwrap_or_default();

                let can_pick_many = options
                    .get("canPickMany")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let place_holder = options
                    .get("placeHolder")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let title = options
                    .get("title")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let match_on_description = options
                    .get("matchOnDescription")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let match_on_detail = options
                    .get("matchOnDetail")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let (tx, rx) = oneshot::channel();
                let request_id = Uuid::new_v4().to_string();
                {
                    let mut pending = self.pending_quick_pick_requests.write().await;
                    pending.insert(request_id.clone(), tx);
                }

                self.emit_event(SessionEvent::QuickPickRequest {
                    id: request_id.clone(),
                    items,
                    can_pick_many,
                    place_holder,
                    title,
                    match_on_description,
                    match_on_detail,
                });

                let selection = match rx.await {
                    Ok(value) => value.unwrap_or(Value::Null),
                    Err(_) => Value::Null,
                };

                {
                    let mut pending = self.pending_quick_pick_requests.write().await;
                    pending.remove(&request_id);
                }

                Ok(json!({ "selected": selection }))
            },
            "window-show-input-box" => {
                let prompt = payload
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let place_holder = payload
                    .get("placeHolder")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let value = payload
                    .get("value")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let password = payload
                    .get("password")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let value_selection = payload
                    .get("valueSelection")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| {
                        if arr.len() == 2 {
                            let start = arr[0].as_u64().map(|v| v as usize)?;
                            let end = arr[1].as_u64().map(|v| v as usize)?;
                            Some((start, end))
                        } else {
                            None
                        }
                    });

                let (tx, rx) = oneshot::channel();
                let request_id = Uuid::new_v4().to_string();
                {
                    let mut pending = self.pending_input_requests.write().await;
                    pending.insert(request_id.clone(), tx);
                }

                self.emit_event(SessionEvent::InputBoxRequest {
                    id: request_id.clone(),
                    prompt,
                    place_holder,
                    value: value.clone(),
                    password,
                    value_selection,
                });

                let result = match rx.await {
                    Ok(val) => val,
                    Err(_) => None,
                };

                {
                    let mut pending = self.pending_input_requests.write().await;
                    pending.remove(&request_id);
                }

                Ok(json!({ "value": result }))
            },
            "output-channel-append" => {
                let channel = payload
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Extension Host")
                    .to_string();
                let value = payload
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

                self.append_output_channel(&channel, &value).await?;
                Ok(json!({"success": true}))
            },
            "output-channel-clear" => {
                if let Some(channel) = payload.get("channel").and_then(|v| v.as_str()) {
                    self.clear_output_channel(channel).await?;
                }
                Ok(json!({"success": true}))
            },
            "output-channel-show" => {
                if let Some(channel) = payload.get("channel").and_then(|v| v.as_str()) {
                    self.set_output_channel_visibility(channel, true).await?;
                }
                Ok(json!({"success": true}))
            },
            "output-channel-hide" => {
                if let Some(channel) = payload.get("channel").and_then(|v| v.as_str()) {
                    self.set_output_channel_visibility(channel, false).await?;
                }
                Ok(json!({"success": true}))
            },
            "output-channel-dispose" => {
                if let Some(channel) = payload.get("channel").and_then(|v| v.as_str()) {
                    self.dispose_output_channel(channel).await?;
                }
                Ok(json!({"success": true}))
            },
            "workspace-find-files" => {
                let include = payload
                    .get("include")
                    .and_then(|v| v.as_str())
                    .unwrap_or("**/*");
                let exclude = payload
                    .get("exclude")
                    .and_then(|v| v.as_str());
                let max_results = payload
                    .get("maxResults")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1000) as usize;

                let include_patterns = Self::split_patterns(include, "**/*");
                let exclude_patterns = Self::split_patterns_opt(exclude);

                let files = self
                    .find_workspace_files(&include_patterns, &exclude_patterns, max_results)
                    .await?;

                Ok(json!({ "files": files }))
            },
            "workspace-save-document" => {
                let path = payload
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing document path".to_string())?;
                let content = payload
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let version = self.persist_document(path, content).await?;
                Ok(json!({"success": true, "version": version}))
            },
            "workspace-get-configuration" => {
                let section = payload.get("section").and_then(|v| v.as_str());
                let store = self.configuration.read().await;
                let config = store.snapshot(section);
                Ok(json!({ "config": config }))
            },
            "workspace-update-configuration" => {
                let key = payload
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing configuration key".to_string())?;
                let section = payload.get("section").and_then(|v| v.as_str()).map(|s| s.to_string());
                let value = payload.get("value").cloned().unwrap_or(Value::Null);
                let mut store = self.configuration.write().await;
                let changed = store.update(section.as_deref(), key, value.clone())?;
                drop(store);

                if changed {
                    self.emit_event(SessionEvent::ConfigurationChanged {
                        section: section.clone(),
                        key: Some(key.to_string()),
                    });

                    let notify_payload = json!({
                        "section": section,
                        "key": key,
                        "value": value,
                    });

                    if let Err(err) = self
                        .nng_manager
                        .request("main", "configuration-changed", notify_payload)
                        .await
                    {
                        eprintln!("[SessionManager] Failed to forward configuration change: {}", err);
                    }
                }

                Ok(json!({"success": true}))
            },
            "executeCommandRequest" => {
                let command = payload
                    .get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

                println!(
                    "[SessionManager] Extension host requested core command '{}', but no handler is available",
                    command
                );

                Ok(json!({
                    "success": false,
                    "error": format!("Command '{}' is not implemented in the host environment", command),
                }))
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

        if !self.app_dirs.extensions_dir.exists() {
            std::fs::create_dir_all(&self.app_dirs.extensions_dir)
                .map_err(|e| format!("Failed to create extensions directory: {}", e))?;
        }

        let entries = std::fs::read_dir(&self.app_dirs.extensions_dir)
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

        {
            let mut state = self.state.write().await;
            state.installed_extensions = extensions.clone();
        }

        self.rebuild_command_index(&extensions).await;
        self.publish_command_list().await;

        println!("[SessionManager] Found {} installed extensions", extensions.len());

        self.emit_event(SessionEvent::ExtensionsChanged { extensions });

        Ok(())
    }

    /// Public helper to rescan extensions and emit change events.
    pub async fn scan_and_emit_extensions(&self) -> Result<(), String> {
        self.scan_extensions().await
    }

    async fn rebuild_command_index(&self, extensions: &[ExtensionInfo]) {
        let mut map = self.command_map.write().await;
        map.clear();
        for ext in extensions {
            for command in &ext.commands {
                let owners = map.entry(command.clone()).or_insert_with(Vec::new);
                if !owners.contains(&ext.id) {
                    owners.push(ext.id.clone());
                }
            }
        }
    }

    async fn publish_command_list(&self) {
        let commands = {
            let map = self.command_map.read().await;
            let mut commands: Vec<String> = map.keys().cloned().collect();
            commands.sort();
            commands
        };

        {
            let mut state = self.state.write().await;
            state.available_commands = commands.clone();
        }

        self.emit_event(SessionEvent::CommandsChanged { commands });
    }

    async fn show_status_bar_message(&self, id: &str, text: &str) -> Result<(), String> {
        {
            let mut map = self.status_messages.write().await;
            map.insert(id.to_string(), StatusMessageEntry { text: text.to_string() });
        }

        self.emit_event(SessionEvent::StatusBarMessageShown {
            id: id.to_string(),
            text: text.to_string(),
        });

        Ok(())
    }

    async fn clear_status_bar_message(&self, id: &str) -> Result<(), String> {
        let removed = {
            let mut map = self.status_messages.write().await;
            map.remove(id).is_some()
        };

        if removed {
            self.emit_event(SessionEvent::StatusBarMessageCleared {
                id: id.to_string(),
            });
        }

        Ok(())
    }

    async fn append_output_channel(&self, channel: &str, value: &str) -> Result<(), String> {
        let mut new_channel = false;
        {
            let mut map = self.output_channels.write().await;
            let entry = map.entry(channel.to_string()).or_insert_with(|| {
                new_channel = true;
                OutputChannelEntry {
                    name: channel.to_string(),
                    lines: Vec::new(),
                    visible: false,
                }
            });

            entry.lines.push(value.to_string());
            if entry.lines.len() > Self::OUTPUT_CHANNEL_MAX_LINES {
                let excess = entry.lines.len() - Self::OUTPUT_CHANNEL_MAX_LINES;
                entry.lines.drain(0..excess);
            }
        }

        if new_channel {
            self.emit_event(SessionEvent::OutputChannelRegistered {
                channel: channel.to_string(),
            });
        }

        self.emit_event(SessionEvent::OutputChannelAppended {
            channel: channel.to_string(),
            value: value.to_string(),
        });

        Ok(())
    }

    async fn clear_output_channel(&self, channel: &str) -> Result<(), String> {
        let cleared = {
            let mut map = self.output_channels.write().await;
            if let Some(entry) = map.get_mut(channel) {
                if !entry.lines.is_empty() {
                    entry.lines.clear();
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        if cleared {
            self.emit_event(SessionEvent::OutputChannelCleared {
                channel: channel.to_string(),
            });
        }

        Ok(())
    }

    async fn dispose_output_channel(&self, channel: &str) -> Result<(), String> {
        let mut next_channel: Option<String> = None;
        let removed = {
            let mut map = self.output_channels.write().await;
            if map.remove(channel).is_some() {
                next_channel = map.keys().next().cloned();
                true
            } else {
                false
            }
        };

        if removed {
            {
                let mut active = self.active_output_channel.write().await;
                if active.as_deref() == Some(channel) {
                    *active = next_channel.clone();
                }
            }

            self.emit_event(SessionEvent::OutputChannelDisposed {
                channel: channel.to_string(),
            });

            if let Some(next) = next_channel {
                self.emit_event(SessionEvent::OutputChannelVisibility {
                    channel: next,
                    visible: true,
                });
            }
        }

        Ok(())
    }

    async fn set_output_channel_visibility(&self, channel: &str, visible: bool) -> Result<(), String> {
        let mut new_channel = false;
        {
            let mut map = self.output_channels.write().await;
            let entry = map.entry(channel.to_string()).or_insert_with(|| {
                new_channel = true;
                OutputChannelEntry {
                    name: channel.to_string(),
                    lines: Vec::new(),
                    visible: false,
                }
            });
            entry.visible = visible;
        }

        if new_channel {
            self.emit_event(SessionEvent::OutputChannelRegistered {
                channel: channel.to_string(),
            });
        }

        if visible {
            let mut active = self.active_output_channel.write().await;
            *active = Some(channel.to_string());
        } else {
            let mut active = self.active_output_channel.write().await;
            if active.as_deref() == Some(channel) {
                *active = None;
            }
        }

        self.emit_event(SessionEvent::OutputChannelVisibility {
            channel: channel.to_string(),
            visible,
        });

        Ok(())
    }

    async fn open_text_document(&self, path: &str) -> Result<Value, String> {
        let content = fs::read_to_string(path).unwrap_or_default();
        let eol = if content.contains("\r\n") { 2 } else { 1 };
        let version = self.get_document_version(path).await;

        Ok(json!({
            "uri": path,
            "text": content,
            "languageId": Self::detect_language_id(path),
            "version": version,
            "isDirty": false,
            "eol": eol,
        }))
    }

    async fn persist_document(&self, path: &str, content: &str) -> Result<i32, String> {
        if let Some(parent) = Path::new(path).parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                return Err(format!("Failed to prepare document directory {:?}: {}", parent, err));
            }
        }

        fs::write(path, content)
            .map_err(|e| format!("Failed to save document {}: {}", path, e))?;

        let version = self.bump_document_version(path).await;
        self.emit_event(SessionEvent::DocumentChanged {
            path: path.to_string(),
            content: content.to_string(),
        });

        Ok(version)
    }

    async fn get_document_version(&self, path: &str) -> i32 {
        let mut map = self.document_versions.write().await;
        let entry = map.entry(path.to_string()).or_insert(1);
        *entry
    }

    async fn bump_document_version(&self, path: &str) -> i32 {
        let mut map = self.document_versions.write().await;
        let entry = map.entry(path.to_string()).or_insert(1);
        *entry += 1;
        *entry
    }

    fn detect_language_id(path: &str) -> &'static str {
        match Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "rs" => "rust",
            "ts" | "tsx" => "typescript",
            "js" | "jsx" => "javascript",
            "py" => "python",
            "java" => "java",
            "cs" => "csharp",
            "cpp" | "cxx" | "cc" | "h" | "hpp" => "cpp",
            "go" => "go",
            "rb" => "ruby",
            "swift" => "swift",
            "kt" => "kotlin",
            "php" => "php",
            "json" => "json",
            "yaml" | "yml" => "yaml",
            "toml" => "toml",
            "md" => "markdown",
            "html" | "htm" => "html",
            "css" | "scss" | "less" => "css",
            _ => "plaintext",
        }
    }

    fn snippet_to_plain(snippet: &str) -> String {
        let placeholder = Regex::new(r"\$\{(\d+):([^}]*)\}").unwrap();
        let tabstop = Regex::new(r"\$(\d+)").unwrap();
        let mut result = placeholder.replace_all(snippet, "$2").into_owned();
        result = tabstop.replace_all(&result, "").into_owned();
        result.replace("\\$", "$")
    }

    fn parse_snippet_locations(location: Option<&Value>) -> Vec<RangePayload> {
        fn push_location(target: &mut Vec<RangePayload>, value: &Value) {
            if let Ok(range) = serde_json::from_value::<RangePayload>(value.clone()) {
                target.push(range);
                return;
            }

            if let Ok(position) = serde_json::from_value::<PositionPayload>(value.clone()) {
                target.push(RangePayload::from_position(position));
            }
        }

        let mut ranges = Vec::new();
        match location {
            Some(Value::Array(items)) => {
                for item in items {
                    push_location(&mut ranges, item);
                }
            }
            Some(value) => push_location(&mut ranges, value),
            None => {}
        }

        if ranges.is_empty() {
            ranges.push(RangePayload::from_position(PositionPayload {
                line: 0,
                character: 0,
            }));
        }

        ranges
    }

    fn apply_text_edits(
        original: &str,
        edits: &mut [TextEditPayload],
        end_of_line: Option<i32>,
    ) -> Result<String, String> {
        let mut normalized = original.replace("\r\n", "\n");
        let original_crlf = original.contains("\r\n");

        edits.sort_by(|a, b| {
            b.range
                .start
                .line
                .cmp(&a.range.start.line)
                .then_with(|| b.range.start.character.cmp(&a.range.start.character))
        });

        for edit in edits.iter() {
            let line_offsets = Self::build_line_offsets(&normalized);
            let start = Self::offset_for_position(&normalized, &line_offsets, &edit.range.start)?;
            let end = Self::offset_for_position(&normalized, &line_offsets, &edit.range.end)?;
            let replacement = edit.new_text.replace("\r\n", "\n");
            normalized.replace_range(start..end, &replacement);
        }

        let newline = match end_of_line {
            Some(2) => "\r\n",
            Some(1) => "\n",
            _ => {
                if original_crlf {
                    "\r\n"
                } else {
                    "\n"
                }
            }
        };

        let result = if newline == "\n" {
            normalized
        } else {
            normalized.replace("\n", newline)
        };

        Ok(result)
    }

    fn build_line_offsets(text: &str) -> Vec<usize> {
        let mut offsets = vec![0];
        for (idx, ch) in text.char_indices() {
            if ch == '\n' {
                offsets.push(idx + 1);
            }
        }
        offsets
    }

    fn offset_for_position(
        text: &str,
        offsets: &[usize],
        position: &PositionPayload,
    ) -> Result<usize, String> {
        if offsets.is_empty() {
            return Ok(0);
        }

        let line_index = position.line.min(offsets.len() - 1);
        let line_start = offsets[line_index];
        let line_end = if line_index + 1 < offsets.len() {
            offsets[line_index + 1].saturating_sub(1)
        } else {
            text.len()
        };

        let line_text = &text[line_start..line_end];
        let total_chars = line_text.chars().count();
        let target_chars = position.character.min(total_chars);

        let mut byte_offset = line_start;
        let mut consumed = 0;
        for (idx, ch) in line_text.char_indices() {
            if consumed == target_chars {
                byte_offset = line_start + idx;
                break;
            }
            consumed += 1;
            byte_offset = line_start + idx + ch.len_utf8();
        }

        if target_chars == total_chars {
            byte_offset = line_end;
        }

        Ok(byte_offset)
    }

    async fn update_decorations(&self, uri: &str, key: &str, decorations: Value) -> Result<(), String> {
        let normalized = self.normalize_decorations(key, decorations).await?;

        {
            let mut map = self.editor_decorations.write().await;
            let entry = map.entry(uri.to_string()).or_insert_with(HashMap::new);
            if normalized.is_empty() {
                entry.remove(key);
                if entry.is_empty() {
                    map.remove(uri);
                }
            } else {
                entry.insert(key.to_string(), Value::Array(normalized.clone()));
            }
        }

        self.emit_event(SessionEvent::EditorDecorations {
            uri: uri.to_string(),
            key: key.to_string(),
            decorations: Value::Array(normalized),
        });

        Ok(())
    }

    async fn dispose_decorations(&self, key: &str) -> Result<(), String> {
        let mut affected = Vec::new();
        {
            let mut map = self.editor_decorations.write().await;
            for (uri, decorations) in map.iter_mut() {
                if decorations.remove(key).is_some() {
                    affected.push(uri.clone());
                }
            }
            map.retain(|_, decorations| !decorations.is_empty());
        }
        {
            let mut types = self.decoration_types.write().await;
            types.remove(key);
        }

        for uri in affected {
            self.emit_event(SessionEvent::EditorDecorations {
                uri: uri.clone(),
                key: key.to_string(),
                decorations: Value::Array(Vec::new()),
            });
        }

        Ok(())
    }

    async fn normalize_decorations(&self, key: &str, decorations: Value) -> Result<Vec<Value>, String> {
        let base_options = {
            let map = self.decoration_types.read().await;
            map.get(key).cloned().unwrap_or(Value::Object(Map::new()))
        };

        let mut normalized = Vec::new();
        let entries = match decorations {
            Value::Array(arr) => arr,
            Value::Null => Vec::new(),
            other => vec![other],
        };

        for entry in entries {
            let (range, hover, specific_options) = if let Ok(range) = serde_json::from_value::<RangePayload>(entry.clone()) {
                (range, None, None)
            } else if let Some(range_value) = entry.get("range") {
                let range = serde_json::from_value::<RangePayload>(range_value.clone())
                    .map_err(|e| format!("Invalid decoration range payload: {}", e))?;
                let hover = entry.get("hoverMessage").cloned();
                let specific = entry.get("renderOptions").or_else(|| entry.get("options")).cloned();
                (range, hover, specific)
            } else {
                continue;
            };

            let merged_options = Self::merge_decoration_options(&base_options, specific_options.as_ref());
            let mut map = Map::new();
            map.insert("range".into(), serde_json::to_value(&range).unwrap_or(Value::Null));
            map.insert("options".into(), merged_options);
            if let Some(hover_msg) = hover {
                map.insert("hoverMessage".into(), hover_msg);
            }
            normalized.push(Value::Object(map));
        }

        Ok(normalized)
    }

    fn merge_decoration_options(base: &Value, specific: Option<&Value>) -> Value {
        let mut merged = match base {
            Value::Object(obj) => obj.clone(),
            _ => Map::new(),
        };

        if let Some(Value::Object(spec)) = specific {
            for (key, value) in spec {
                merged.insert(key.clone(), value.clone());
            }
        }

        Value::Object(merged)
    }

    fn split_patterns(input: &str, fallback: &str) -> Vec<String> {
        let trimmed = input.trim();
        let mut patterns: Vec<String> = trimmed
            .split(',')
            .map(|segment| segment.trim())
            .filter(|segment| !segment.is_empty())
            .map(|segment| segment.to_string())
            .collect();

        if patterns.is_empty() {
            patterns.push(fallback.to_string());
        }

        patterns
    }

    fn split_patterns_opt(input: Option<&str>) -> Vec<String> {
        input
            .map(|value| {
                value
                    .split(',')
                    .map(|segment| segment.trim())
                    .filter(|segment| !segment.is_empty())
                    .map(|segment| segment.to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default()
    }

    fn build_glob_set(patterns: &[String]) -> Result<Option<GlobSet>, String> {
        if patterns.is_empty() {
            return Ok(None);
        }

        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            let glob = Glob::new(pattern)
                .map_err(|e| format!("Invalid glob pattern '{}': {}", pattern, e))?;
            builder.add(glob);
        }

        let set = builder
            .build()
            .map_err(|e| format!("Failed to build glob set: {}", e))?;
        Ok(Some(set))
    }

    fn matches_glob(set: &Option<GlobSet>, path: &Path) -> bool {
        match set {
            Some(globs) => globs.is_match(path),
            None => true,
        }
    }

    fn matches_glob_optional(set: &Option<GlobSet>, path: &Path) -> bool {
        match set {
            Some(globs) => globs.is_match(path),
            None => false,
        }
    }

    fn matches_glob_arc(set: &Option<Arc<GlobSet>>, path: &Path) -> bool {
        match set {
            Some(globs) => globs.is_match(path),
            None => true,
        }
    }

    pub async fn notify_editor_selection(
        &self,
        uri: &str,
        selection: Value,
        selections: Value,
    ) -> Result<(), String> {
        let payload = json!({
            "uri": uri,
            "selection": selection,
            "selections": selections,
        });
        self.nng_manager.request("main", "selectionChanged", payload).await?;
        Ok(())
    }

    pub async fn notify_editor_visible_ranges(
        &self,
        uri: &str,
        ranges: Value,
    ) -> Result<(), String> {
        let payload = json!({
            "uri": uri,
            "ranges": ranges,
        });
        self.nng_manager.request("main", "visibleRangesChanged", payload).await?;
        Ok(())
    }

    async fn find_workspace_files(
        &self,
        include_patterns: &[String],
        exclude_patterns: &[String],
        max_results: usize,
    ) -> Result<Vec<String>, String> {
        let include_glob = Self::build_glob_set(include_patterns)?;
        let exclude_glob = Self::build_glob_set(exclude_patterns)?;

        let roots = {
            let state = self.state.read().await;
            if state.workspace_folders.is_empty() {
                vec![env::current_dir().map_err(|e| format!("Failed to determine workspace directory: {}", e))?]
            } else {
                state.workspace_folders.clone()
            }
        };

        let mut results = Vec::new();

        for root in roots {
            let walker = WalkBuilder::new(&root)
                .hidden(false)
                .git_ignore(true)
                .git_exclude(true)
                .parents(true)
                .ignore(true)
                .build();

            for entry in walker {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(err) => {
                        eprintln!("[SessionManager] File search error: {}", err);
                        continue;
                    }
                };

                let file_type = match entry.file_type() {
                    Some(ty) => ty,
                    None => continue,
                };

                if !file_type.is_file() {
                    continue;
                }

                let relative = entry.path().strip_prefix(&root).unwrap_or(entry.path());

                if !Self::matches_glob(&include_glob, relative) {
                    continue;
                }

                if Self::matches_glob_optional(&exclude_glob, relative) {
                    continue;
                }

                results.push(entry.path().to_string_lossy().to_string());

                if results.len() >= max_results {
                    return Ok(results);
                }
            }
        }

        Ok(results)
    }

    async fn register_file_system_watcher(
        &self,
        watcher_id: &str,
        pattern: &str,
        ignore_create: bool,
        ignore_change: bool,
        ignore_delete: bool,
    ) -> Result<(), String> {
        let include_patterns = Self::split_patterns(pattern, pattern);
        let glob_set = Self::build_glob_set(&include_patterns)?;
        let matcher = glob_set.map(Arc::new);

        let roots = {
            let state = self.state.read().await;
            if state.workspace_folders.is_empty() {
                vec![env::current_dir().map_err(|e| format!("Failed to determine workspace directory: {}", e))?]
            } else {
                state.workspace_folders.clone()
            }
        };

        let session = self.clone_for_handler();
        let watcher_id_string = watcher_id.to_string();
        let matcher_clone = matcher.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    let event_type = match event.kind {
                        EventKind::Create(CreateKind::Any | CreateKind::File | CreateKind::Folder) => {
                            if ignore_create { None } else { Some("created") }
                        }
                        EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Any | ModifyKind::Metadata(_)) => {
                            if ignore_change { None } else { Some("changed") }
                        }
                        EventKind::Remove(RemoveKind::Any | RemoveKind::File | RemoveKind::Folder) => {
                            if ignore_delete { None } else { Some("deleted") }
                        }
                        _ => None,
                    };

                    if let Some(kind) = event_type {
                        for path in event.paths {
                            if !Self::matches_glob_arc(&matcher_clone, &path) {
                                continue;
                            }
                            let session_clone = session.clone();
                            let watcher_id = watcher_id_string.clone();
                            let path_string = path.to_string_lossy().to_string();
                            let kind_string = kind.to_string();
                            tokio::spawn(async move {
                                session_clone.emit_fs_event(&watcher_id, &kind_string, &path_string).await;
                            });
                        }
                    }
                }
                Err(err) => {
                    eprintln!("[SessionManager] File watcher error: {}", err);
                }
            }
        }).map_err(|e| format!("Failed to create file watcher: {}", e))?;

        for root in &roots {
            watcher.watch(root, RecursiveMode::Recursive)
                .map_err(|e| format!("Failed to watch {:?}: {}", root, e))?;
        }

        let mut map = self.extension_watchers.lock().unwrap();
        map.insert(watcher_id.to_string(), watcher);

        Ok(())
    }

    fn unregister_file_system_watcher(&self, watcher_id: &str) {
        let mut map = self.extension_watchers.lock().unwrap();
        map.remove(watcher_id);
    }

    async fn emit_fs_event(&self, watcher_id: &str, event: &str, path: &str) {
        let payload = json!({
            "id": watcher_id,
            "event": event,
            "path": path,
        });

        if let Err(err) = self.nng_manager.request("main", "fsWatcher:event", payload).await {
            eprintln!("[SessionManager] Failed to forward fs watcher event: {}", err);
        }
    }

    fn status_bar_key(owner: &str, id: &str) -> String {
        format!("{}::{}", owner, id)
    }

    async fn status_bar_items_snapshot(&self) -> Vec<StatusBarItemState> {
        let entries: Vec<StatusBarEntry> = {
            let map = self.status_bar_items.read().await;
            map.values()
                .filter(|entry| entry.visible)
                .cloned()
                .collect()
        };

        let mut entries = entries;
        entries.sort_by(|a, b| {
            let align_order = a.alignment.sort_value().cmp(&b.alignment.sort_value());
            if align_order != Ordering::Equal {
                return align_order;
            }

            let ap = a.priority.unwrap_or(0.0);
            let bp = b.priority.unwrap_or(0.0);
            bp.partial_cmp(&ap).unwrap_or(Ordering::Equal)
        });

        entries.into_iter().map(|entry| entry.to_state()).collect()
    }

    async fn publish_status_bar_items(&self) {
        let items = self.status_bar_items_snapshot().await;

        {
            let mut state = self.state.write().await;
            state.status_bar_items = items.clone();
        }

        self.emit_event(SessionEvent::StatusBarItems { items });
    }

    pub async fn resolve_window_message(&self, request_id: &str, action: Option<String>) -> Result<(), String> {
        let sender = {
            let mut pending = self.pending_message_requests.write().await;
            pending.remove(request_id)
        };

        if let Some(sender) = sender {
            let _ = sender.send(action);
        }

        Ok(())
    }

    pub async fn resolve_quick_pick(&self, request_id: &str, selection: Option<Value>) -> Result<(), String> {
        let sender = {
            let mut pending = self.pending_quick_pick_requests.write().await;
            pending.remove(request_id)
        };

        if let Some(sender) = sender {
            let _ = sender.send(selection);
        }

        Ok(())
    }

    pub async fn resolve_input_box(&self, request_id: &str, value: Option<String>) -> Result<(), String> {
        let sender = {
            let mut pending = self.pending_input_requests.write().await;
            pending.remove(request_id)
        };

        if let Some(sender) = sender {
            let _ = sender.send(value);
        }

        Ok(())
    }

    async fn upsert_status_bar_item(&self, id: &str, owner: &str, payload: &Value) -> Result<(), String> {
        {
            let mut map = self.status_bar_items.write().await;
            let key = Self::status_bar_key(owner, id);
            let entry = map
                .entry(key)
                .or_insert_with(|| StatusBarEntry::new(id.to_string(), owner.to_string()));

            entry.owner = owner.to_string();
            entry.id = id.to_string();

            if let Some(text) = payload.get("text").and_then(|v| v.as_str()) {
                entry.text = text.to_string();
            }

            if let Some(tooltip) = payload.get("tooltip") {
                entry.tooltip = match tooltip {
                    Value::String(text) => Some(text.clone()),
                    Value::Object(map) => map.get("value").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    _ => entry.tooltip.clone(),
                };
            }

            if let Some(color) = payload.get("color").and_then(|v| v.as_str()) {
                entry.color = Some(color.to_string());
            } else if payload.get("color").is_some() {
                entry.color = None;
            }

            if let Some(cmd_value) = payload.get("command") {
                entry.command = Self::parse_status_bar_command(cmd_value);
            }

            if let Some(alignment_value) = payload.get("alignment").and_then(|v| v.as_i64()) {
                entry.alignment = StatusBarAlignment::from_i32(alignment_value as i32);
            }

            if let Some(priority_value) = payload.get("priority").and_then(|v| v.as_f64()) {
                entry.priority = Some(priority_value);
            } else if payload.get("priority").is_some() {
                entry.priority = None;
            }

            entry.visible = payload
                .get("visible")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
        }

        self.publish_status_bar_items().await;
        Ok(())
    }

    async fn hide_status_bar_item(&self, owner: &str, id: &str) -> Result<(), String> {
        let key = Self::status_bar_key(owner, id);
        let changed = {
            let mut map = self.status_bar_items.write().await;
            if let Some(entry) = map.get_mut(&key) {
                if entry.visible {
                    entry.visible = false;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        if changed {
            self.publish_status_bar_items().await;
        }

        Ok(())
    }

    async fn dispose_status_bar_item(&self, owner: &str, id: &str) -> Result<(), String> {
        let key = Self::status_bar_key(owner, id);
        let removed = {
            let mut map = self.status_bar_items.write().await;
            map.remove(&key).is_some()
        };

        if removed {
            self.publish_status_bar_items().await;
        }

        Ok(())
    }

    fn parse_status_bar_command(value: &Value) -> Option<StatusBarCommand> {
        match value {
            Value::String(id) => Some(StatusBarCommand { id: id.clone(), arguments: None }),
            Value::Object(obj) => {
                let id = obj.get("command")?.as_str()?.to_string();
                let arguments = obj.get("arguments").and_then(|args| args.as_array().map(|arr| arr.clone()));
                Some(StatusBarCommand { id, arguments })
            }
            _ => None,
        }
    }

    async fn add_command_owner(&self, command: &str, owner: &str) {
        if owner.is_empty() {
            return;
        }
        let mut map = self.command_map.write().await;
        let owners = map.entry(command.to_string()).or_insert_with(Vec::new);
        if !owners.contains(&owner.to_string()) {
            owners.push(owner.to_string());
        }
    }

    async fn remove_command_owner(&self, command: &str, owner: &str) {
        let mut map = self.command_map.write().await;
        if let Some(owners) = map.get_mut(command) {
            owners.retain(|entry| entry != owner);
            if owners.is_empty() {
                map.remove(command);
            }
        }
    }

    async fn remove_extension_commands(&self, extension_id: &str) {
        let mut map = self.command_map.write().await;
        map.retain(|_, owners| {
            owners.retain(|owner| owner != extension_id);
            !owners.is_empty()
        });
    }

    async fn remove_extension_status_bar_items(&self, extension_id: &str) {
        let changed = {
            let mut map = self.status_bar_items.write().await;
            let before = map.len();
            map.retain(|_, entry| entry.owner != extension_id);
            before != map.len()
        };

        if changed {
            self.publish_status_bar_items().await;
        }
    }

    async fn is_extension_active(&self, extension_id: &str) -> bool {
        self.state.read().await.active_extensions.iter().any(|ext| ext.id == extension_id)
    }

    async fn owners_from_activation_events(&self, command: &str) -> Vec<String> {
        let pattern = format!("onCommand:{}", command);
        let state = self.state.read().await;
        state
            .installed_extensions
            .iter()
            .filter(|ext| ext.activation_events.iter().any(|evt| evt == &pattern))
            .map(|ext| ext.id.clone())
            .collect()
    }

    pub async fn ensure_command_ready(&self, command: &str) -> Result<(), String> {
        let owners_opt = {
            let map = self.command_map.read().await;
            map.get(command).cloned()
        };

        let mut owners = owners_opt.unwrap_or_default();

        if owners.is_empty() {
            owners = self.owners_from_activation_events(command).await;
        }

        if owners.is_empty() {
            return Ok(());
        }

        owners.sort();
        owners.dedup();

        for owner in owners {
            if owner == Self::CORE_COMMAND_OWNER {
                continue;
            }

            if !self.is_extension_active(&owner).await {
                self.load_extension(&owner).await?;
            }
        }

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

        let categories = manifest.get("categories")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let mut dependencies = manifest.get("extensionDependencies")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        if let Some(pack) = manifest.get("extensionPack").and_then(|v| v.as_array()) {
            for item in pack {
                if let Some(value) = item.as_str() {
                    dependencies.push(value.to_string());
                }
            }
        }

        dependencies.sort();
        dependencies.dedup();

        let activation_events = manifest.get("activationEvents")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let commands = manifest.get("contributes")
            .and_then(|v| v.get("commands"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("command").and_then(|c| c.as_str()).map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let repository = manifest.get("repository")
            .and_then(|value| {
                if let Some(url) = value.as_str() {
                    Some(url.to_string())
                } else if let Some(obj) = value.as_object() {
                    obj.get("url").and_then(|u| u.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            });

        Ok(ExtensionInfo {
            id: extension_id,
            name,
            version,
            publisher,
            description,
            enabled: true,
            active: false,
            categories,
            dependencies,
            repository,
            activation_events,
            commands,
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
        let extension_path = self.app_dirs.extensions_dir.join(extension_id);

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
        });
        self.remove_extension_status_bar_items(extension_id).await;

        Ok(())
    }

    /// Delete an extension
    pub async fn delete_extension(&self, extension_id: &str) -> Result<(), String> {
        println!("[SessionManager] Deleting extension: {}", extension_id);

        // Unload if active
        let _ = self.unload_extension(extension_id).await;

        // Ask the extension host to uninstall and clean up its internal state
        let payload = serde_json::json!({ "extensionId": extension_id });
        let response = self
            .nng_manager
            .request("main", "uninstall-extension", payload)
            .await?;

        let success = response.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        if !success {
            let error = response
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error returned from extension host");
            return Err(format!("Extension host failed to uninstall {}: {}", extension_id, error));
        }

        // Delete from filesystem
        let extension_path = self.app_dirs.extensions_dir.join(extension_id);
        if let Err(e) = remove_dir_if_exists(&extension_path) {
            eprintln!(
                "[SessionManager] Failed to delete extension directory {:?}: {}",
                extension_path, e
            );
        }

        let storage_dir = self.app_dirs.storage_dir.join(extension_id);
        if let Err(e) = remove_dir_if_exists(&storage_dir) {
            eprintln!(
                "[SessionManager] Failed to delete storage directory {:?}: {}",
                storage_dir, e
            );
        }

        let logs_dir = self.app_dirs.logs_dir.join(extension_id);
        if let Err(e) = remove_dir_if_exists(&logs_dir) {
            eprintln!(
                "[SessionManager] Failed to delete log directory {:?}: {}",
                logs_dir, e
            );
        }

        // Remove from state
        let mut state = self.state.write().await;
        state.installed_extensions.retain(|ext| ext.id != extension_id);
        state.active_extensions.retain(|ext| ext.id != extension_id);
        let extensions = state.installed_extensions.clone();
        drop(state);

        self.remove_extension_commands(extension_id).await;
        self.publish_command_list().await;
        self.remove_extension_status_bar_items(extension_id).await;

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

fn remove_dir_if_exists(path: &Path) -> Result<(), std::io::Error> {
    if !path.exists() {
        return Ok(());
    }

    match std::fs::remove_dir_all(path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}
