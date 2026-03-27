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
mod contributions;
mod documents;
mod extensions;
mod ipc;
mod ipc_providers;
mod workspace;

pub use contributions::ExtensionContributes;
pub use extensions::{ExtensionContribution, InstalledExtension};

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use notify::event::{CreateKind, ModifyKind, RemoveKind};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{oneshot, RwLock};
use tokio::time::Duration;

use crate::config::AppDirectories;
use crate::debug::DebugAdapterPool;
use crate::extension_host::path_validator::PathValidator;
use crate::extension_host::{ExtensionHostManager, IpcManager, SecretStorage};
use crate::lsp::{LspServerPool, LspServerStrategy};
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
    #[serde(default)]
    pub contributes: Option<ExtensionContributes>,
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
        Self { start: pos.clone(), end: pos }
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

    /// Execute command request from extension host
    ExecuteCommandRequest { id: String, command: String, args: Vec<Value> },

    /// Language feature provider registered
    ProviderRegistered {
        provider_type: String,
        provider_id: String,
        owner: String,
        selector: Value,
        trigger_characters: Option<Vec<String>>,
        metadata: Option<Value>,
    },

    /// Language configuration changed
    LanguageConfigurationChanged { language: String, configuration: Value },

    /// Diagnostics updated
    DiagnosticsUpdated { uri: String, diagnostics: Value },

    /// Diagnostics cleared
    DiagnosticsCleared { uri: String },
}

/// Central session manager
pub struct SessionManager {
    app_handle: AppHandle,
    state: Arc<RwLock<SessionState>>,
    extension_host: Arc<tokio::sync::Mutex<ExtensionHostManager>>,
    ipc_manager: Arc<IpcManager>,
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
    extension_watchers: Arc<tokio::sync::Mutex<HashMap<String, RecommendedWatcher>>>,
    workspace_configurations: Arc<RwLock<HashMap<String, crate::commands::WorkspaceConfiguration>>>,
    file_decoration_providers: Arc<RwLock<Vec<crate::commands::FileDecorationProvider>>>,
    file_decorations: Arc<RwLock<HashMap<String, Vec<crate::commands::FileDecoration>>>>,
    extension_host_ready: Arc<tokio::sync::Notify>,
    initialized: Arc<tokio::sync::OnceCell<()>>,
    secrets: Arc<SecretStorage>,
    path_validator: Arc<RwLock<PathValidator>>,
    outgoing_socket: Arc<RwLock<Option<String>>>,
    incoming_socket: Arc<RwLock<Option<String>>>,
    extension_host_entry: Arc<RwLock<Option<String>>>,
}

impl SessionManager {
    const CORE_COMMAND_OWNER: &'static str = "__core__";
    const OUTPUT_CHANNEL_MAX_LINES: usize = 2000;

    pub fn new(app_handle: AppHandle, app_dirs: AppDirectories) -> Self {
        let state = Arc::new(RwLock::new(SessionState {
            workspace_folders: Vec::new(),
            active_extensions: Vec::new(),
            installed_extensions: Vec::new(),
            available_commands: Vec::new(),
            status_bar_items: Vec::new(),
        }));

        let extension_host =
            Arc::new(tokio::sync::Mutex::new(ExtensionHostManager::new("main".to_string())));

        let ipc_manager = Arc::new(IpcManager::new());

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
            ipc_manager,
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
            extension_watchers: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            workspace_configurations: Arc::new(RwLock::new(HashMap::new())),
            file_decoration_providers: Arc::new(RwLock::new(Vec::new())),
            file_decorations: Arc::new(RwLock::new(HashMap::new())),
            extension_host_ready: Arc::new(tokio::sync::Notify::new()),
            initialized: Arc::new(tokio::sync::OnceCell::new()),
            secrets: Arc::new(SecretStorage::new()),
            path_validator: Arc::new(RwLock::new(PathValidator::new())),
            outgoing_socket: Arc::new(RwLock::new(None)),
            incoming_socket: Arc::new(RwLock::new(None)),
            extension_host_entry: Arc::new(RwLock::new(None)),
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
        if self.initialized.get().is_some() {
            println!("[SessionManager] Already initialized, skipping");
            return Ok(());
        }

        println!("[SessionManager] Initializing session...");

        println!("[SessionManager] Starting Extension Host...");
        self.start_extension_host().await?;

        println!("[SessionManager] Waiting for extension host ready signal...");

        let ready_wait = self.extension_host_ready.notified();
        match tokio::time::timeout(Duration::from_secs(10), ready_wait).await {
            Ok(_) => println!("[SessionManager] Extension host is ready"),
            Err(_) => {
                eprintln!("[SessionManager] Timeout waiting for extension host ready signal");
                return Err("Extension host failed to start within timeout".to_string());
            }
        }

        self.scan_extensions().await?;

        self.load_auto_start_extensions().await?;

        let state = self.state.read().await.clone();
        self.emit_event(SessionEvent::StateChanged { state });

        let _ = self.initialized.set(());

        println!("[SessionManager] Session initialized");
        Ok(())
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
            self.emit_event(SessionEvent::StatusBarMessageCleared { id: id.to_string() });
        }

        Ok(())
    }

    async fn append_output_channel(&self, channel: &str, value: &str) -> Result<(), String> {
        let mut new_channel = false;
        {
            let mut map = self.output_channels.write().await;
            let entry = map.entry(channel.to_string()).or_insert_with(|| {
                new_channel = true;
                OutputChannelEntry { name: channel.to_string(), lines: Vec::new(), visible: false }
            });

            entry.lines.push(value.to_string());
            if entry.lines.len() > Self::OUTPUT_CHANNEL_MAX_LINES {
                let excess = entry.lines.len() - Self::OUTPUT_CHANNEL_MAX_LINES;
                entry.lines.drain(0..excess);
            }
        }

        if new_channel {
            self.emit_event(SessionEvent::OutputChannelRegistered { channel: channel.to_string() });
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
            self.emit_event(SessionEvent::OutputChannelCleared { channel: channel.to_string() });
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

            self.emit_event(SessionEvent::OutputChannelDisposed { channel: channel.to_string() });

            if let Some(next) = next_channel {
                self.emit_event(SessionEvent::OutputChannelVisibility {
                    channel: next,
                    visible: true,
                });
            }
        }

        Ok(())
    }

    async fn set_output_channel_visibility(
        &self, channel: &str, visible: bool,
    ) -> Result<(), String> {
        let mut new_channel = false;
        {
            let mut map = self.output_channels.write().await;
            let entry = map.entry(channel.to_string()).or_insert_with(|| {
                new_channel = true;
                OutputChannelEntry { name: channel.to_string(), lines: Vec::new(), visible: false }
            });
            entry.visible = visible;
        }

        if new_channel {
            self.emit_event(SessionEvent::OutputChannelRegistered { channel: channel.to_string() });
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

        let set = builder.build().map_err(|e| format!("Failed to build glob set: {}", e))?;
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

    async fn find_workspace_files(
        &self, include_patterns: &[String], exclude_patterns: &[String], max_results: usize,
    ) -> Result<Vec<String>, String> {
        let include_glob = Self::build_glob_set(include_patterns)?;
        let exclude_glob = Self::build_glob_set(exclude_patterns)?;

        let roots = {
            let state = self.state.read().await;
            if state.workspace_folders.is_empty() {
                vec![env::current_dir()
                    .map_err(|e| format!("Failed to determine workspace directory: {}", e))?]
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
        &self, watcher_id: &str, pattern: &str, ignore_create: bool, ignore_change: bool,
        ignore_delete: bool,
    ) -> Result<(), String> {
        let include_patterns = Self::split_patterns(pattern, pattern);
        let glob_set = Self::build_glob_set(&include_patterns)?;
        let matcher = glob_set.map(Arc::new);

        let roots = {
            let state = self.state.read().await;
            if state.workspace_folders.is_empty() {
                vec![env::current_dir()
                    .map_err(|e| format!("Failed to determine workspace directory: {}", e))?]
            } else {
                state.workspace_folders.clone()
            }
        };

        let session = self.clone_for_handler();
        let watcher_id_string = watcher_id.to_string();
        let matcher_clone = matcher.clone();

        let mut watcher =
            notify::recommended_watcher(move |res: Result<Event, notify::Error>| match res {
                Ok(event) => {
                    let event_type = match event.kind {
                        EventKind::Create(
                            CreateKind::Any | CreateKind::File | CreateKind::Folder,
                        ) => {
                            if ignore_create {
                                None
                            } else {
                                Some("created")
                            }
                        }
                        EventKind::Modify(
                            ModifyKind::Data(_) | ModifyKind::Any | ModifyKind::Metadata(_),
                        ) => {
                            if ignore_change {
                                None
                            } else {
                                Some("changed")
                            }
                        }
                        EventKind::Remove(
                            RemoveKind::Any | RemoveKind::File | RemoveKind::Folder,
                        ) => {
                            if ignore_delete {
                                None
                            } else {
                                Some("deleted")
                            }
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
                                session_clone
                                    .emit_fs_event(&watcher_id, &kind_string, &path_string)
                                    .await;
                            });
                        }
                    }
                }
                Err(err) => {
                    eprintln!("[SessionManager] File watcher error: {}", err);
                }
            })
            .map_err(|e| format!("Failed to create file watcher: {}", e))?;

        for root in &roots {
            watcher
                .watch(root, RecursiveMode::Recursive)
                .map_err(|e| format!("Failed to watch {:?}: {}", root, e))?;
        }

        let mut map = self.extension_watchers.lock().await;
        map.insert(watcher_id.to_string(), watcher);

        Ok(())
    }

    async fn unregister_file_system_watcher(&self, watcher_id: &str) {
        let mut map = self.extension_watchers.lock().await;
        map.remove(watcher_id);
    }

    async fn emit_fs_event(&self, watcher_id: &str, event: &str, path: &str) {
        let payload = json!({
            "id": watcher_id,
            "event": event,
            "path": path,
        });

        if let Err(err) = self.ipc_manager.request("main", "fsWatcher:event", payload).await {
            eprintln!("[SessionManager] Failed to forward fs watcher event: {}", err);
        }
    }

    fn status_bar_key(owner: &str, id: &str) -> String {
        format!("{}::{}", owner, id)
    }

    async fn status_bar_items_snapshot(&self) -> Vec<StatusBarItemState> {
        let entries: Vec<StatusBarEntry> = {
            let map = self.status_bar_items.read().await;
            map.values().filter(|entry| entry.visible).cloned().collect()
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

    pub async fn resolve_window_message(
        &self, request_id: &str, action: Option<String>,
    ) -> Result<(), String> {
        let sender = {
            let mut pending = self.pending_message_requests.write().await;
            pending.remove(request_id)
        };

        if let Some(sender) = sender {
            let _ = sender.send(action);
        }

        Ok(())
    }

    pub async fn resolve_quick_pick(
        &self, request_id: &str, selection: Option<Value>,
    ) -> Result<(), String> {
        let sender = {
            let mut pending = self.pending_quick_pick_requests.write().await;
            pending.remove(request_id)
        };

        if let Some(sender) = sender {
            let _ = sender.send(selection);
        }

        Ok(())
    }

    pub async fn resolve_input_box(
        &self, request_id: &str, value: Option<String>,
    ) -> Result<(), String> {
        let sender = {
            let mut pending = self.pending_input_requests.write().await;
            pending.remove(request_id)
        };

        if let Some(sender) = sender {
            let _ = sender.send(value);
        }

        Ok(())
    }

    pub async fn resolve_execute_command(
        &self, request_id: &str, result: Option<Value>,
    ) -> Result<(), String> {
        let sender = {
            let mut pending = self.pending_quick_pick_requests.write().await;
            pending.remove(request_id)
        };

        if let Some(sender) = sender {
            let _ = sender.send(result);
        }

        Ok(())
    }

    async fn upsert_status_bar_item(
        &self, id: &str, owner: &str, payload: &Value,
    ) -> Result<(), String> {
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
                    Value::Object(map) => {
                        map.get("value").and_then(|v| v.as_str()).map(|s| s.to_string())
                    }
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

            entry.visible = payload.get("visible").and_then(|v| v.as_bool()).unwrap_or(true);
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
                let arguments = obj.get("arguments").and_then(|args| args.as_array().cloned());
                Some(StatusBarCommand { id, arguments })
            }
            _ => None,
        }
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
    pub fn ipc_manager(&self) -> &Arc<IpcManager> {
        &self.ipc_manager
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
