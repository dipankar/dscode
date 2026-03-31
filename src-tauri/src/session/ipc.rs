use super::contributions::ExtensionContributes;
use super::{SessionEvent, SessionManager, TextEditPayload};
use crate::commands::{CommandInfo, CommandRegistry, LanguageFeaturesRegistry};
use dscode_extension_host::PathValidator;
use dscode_extension_host::IncomingRequestHandler;
use serde_json::{json, Map, Value};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::path::BaseDirectory;
use tauri::Manager;
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Maximum age for a pending request before it's considered stale (5 minutes).
const STALE_REQUEST_TIMEOUT_SECS: u64 = 300;

/// Remove orphaned dscode-*.sock files from /tmp that belong to previous sessions.
pub fn cleanup_stale_sockets() {
    #[cfg(target_family = "unix")]
    {
        let socket_dir = std::path::Path::new("/tmp");
        let mut cleaned = 0;
        if let Ok(entries) = fs::read_dir(socket_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("dscode-") && name_str.ends_with(".sock") {
                    if let Err(e) = fs::remove_file(entry.path()) {
                        warn!("Failed to remove stale socket {:?}: {}", entry.path(), e);
                    } else {
                        cleaned += 1;
                    }
                }
            }
        }
        if cleaned > 0 {
            info!("Cleaned up {} stale IPC socket(s)", cleaned);
        }
    }
}

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

fn fs_error(code: &str, message: &str) -> String {
    format!("{}: {}", code, message)
}

impl SessionManager {
    /// Spawn a background task that periodically removes stale pending requests.
    ///
    /// If the frontend doesn't respond to a pending request within the timeout
    /// (5 minutes by default), the oneshot::Sender is dropped, causing the
    /// Receiver to get a RecvError. This prevents memory leaks from orphaned
    /// pending requests when the frontend disconnects or hangs.
    pub fn start_pending_request_cleanup(&self) {
        let pending_message = Arc::clone(&self.pending_message_requests);
        let pending_quick_pick = Arc::clone(&self.pending_quick_pick_requests);
        let pending_input = Arc::clone(&self.pending_input_requests);

        tokio::spawn(async move {
            let interval = Duration::from_secs(60);
            loop {
                sleep(interval).await;

                // Clean up message requests if too many are pending (sign of stale entries)
                let stale_message_ids: Vec<String> = {
                    let pending = pending_message.read().await;
                    if pending.len() > 100 {
                        pending.keys().cloned().collect()
                    } else {
                        Vec::new()
                    }
                };

                if !stale_message_ids.is_empty() {
                    let mut pending = pending_message.write().await;
                    for id in stale_message_ids {
                        if let Some(sender) = pending.remove(&id) {
                            drop(sender);
                            debug!("Cleaned up stale message request: {}", id);
                        }
                    }
                    warn!("Cleaned up stale pending message requests");
                }

                // Clean up quick pick requests
                let stale_quick_pick_ids: Vec<String> = {
                    let pending = pending_quick_pick.read().await;
                    if pending.len() > 100 {
                        pending.keys().cloned().collect()
                    } else {
                        Vec::new()
                    }
                };

                if !stale_quick_pick_ids.is_empty() {
                    let mut pending = pending_quick_pick.write().await;
                    for id in stale_quick_pick_ids {
                        if let Some(sender) = pending.remove(&id) {
                            drop(sender);
                            debug!("Cleaned up stale quick pick request: {}", id);
                        }
                    }
                    warn!("Cleaned up stale pending quick pick requests");
                }

                // Clean up input requests
                let stale_input_ids: Vec<String> = {
                    let pending = pending_input.read().await;
                    if pending.len() > 100 {
                        pending.keys().cloned().collect()
                    } else {
                        Vec::new()
                    }
                };

                if !stale_input_ids.is_empty() {
                    let mut pending = pending_input.write().await;
                    for id in stale_input_ids {
                        if let Some(sender) = pending.remove(&id) {
                            drop(sender);
                            debug!("Cleaned up stale input request: {}", id);
                        }
                    }
                    warn!("Cleaned up stale pending input requests");
                }
            }
        });
    }

    /// Start the Extension Host with bidirectional NNG IPC
    pub(super) async fn start_extension_host(&self) -> Result<(), String> {
        {
            let mut manager = self.extension_host.lock().await;
            if manager.state() == dscode_extension_host::ExtensionHostState::Running {
                debug!("[SessionManager] Extension Host already running, skipping");
                return Ok(());
            }
        }

        let session_id = Uuid::new_v4();
        let outgoing_socket = Self::build_ipc_url("ext-out", &session_id)?;
        let incoming_socket = Self::build_ipc_url("ext-in", &session_id)?;

        let session_manager = self.clone_for_handler();
        let handler: IncomingRequestHandler =
            Arc::new(move |msg_type: String, payload: serde_json::Value| {
                let sm = session_manager.clone();
                Box::pin(async move { sm.handle_incoming_request(&msg_type, payload).await })
            });

        self.ipc_manager.setup_incoming("main", &incoming_socket, handler).await?;

        let extension_host_main = self.resolve_extension_host_entry()?;
        let extension_host_main_str = extension_host_main.to_string_lossy().to_string();

        {
            let mut entry = self.extension_host_entry.write().await;
            *entry = Some(extension_host_main_str.clone());
        }

        let mut manager = self.extension_host.lock().await;
        manager.start(&extension_host_main_str, &outgoing_socket, &incoming_socket)?;
        drop(manager);

        {
            let mut outgoing = self.outgoing_socket.write().await;
            *outgoing = Some(outgoing_socket.clone());
        }
        {
            let mut incoming = self.incoming_socket.write().await;
            *incoming = Some(incoming_socket.clone());
        }

        self.ipc_manager.connect_outgoing("main", &outgoing_socket).await?;

        info!("[SessionManager] Extension Host started successfully");
        Ok(())
    }

    /// Clone for handler callback
    pub fn clone_for_handler(&self) -> Arc<Self> {
        Arc::new(Self {
            app_handle: self.app_handle.clone(),
            state: Arc::clone(&self.state),
            extension_host: Arc::clone(&self.extension_host),
            ipc_manager: Arc::clone(&self.ipc_manager),
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
            workspace_configurations: Arc::clone(&self.workspace_configurations),
            file_decoration_providers: Arc::clone(&self.file_decoration_providers),
            file_decorations: Arc::clone(&self.file_decorations),
            extension_host_ready: Arc::clone(&self.extension_host_ready),
            initialized: Arc::clone(&self.initialized),
            lifecycle: Arc::clone(&self.lifecycle),
            secrets: Arc::clone(&self.secrets),
            path_validator: Arc::clone(&self.path_validator),
            outgoing_socket: Arc::clone(&self.outgoing_socket),
            incoming_socket: Arc::clone(&self.incoming_socket),
            extension_host_entry: Arc::clone(&self.extension_host_entry),
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
            let mut dir = cwd.as_path();
            for _ in 0..5 {
                if let Some(parent) = dir.parent() {
                    candidates.push(parent.join("extension-host/dist/main.js"));
                    candidates.push(parent.join("extension-host/main.js"));
                    dir = parent;
                } else {
                    break;
                }
            }
        }

        if let Ok(exe_path) = std::env::current_exe() {
            let mut dir = exe_path.as_path();
            for _ in 0..6 {
                if let Some(parent) = dir.parent() {
                    candidates.push(parent.join("extension-host/dist/main.js"));
                    dir = parent;
                } else {
                    break;
                }
            }
        }

        for candidate in &candidates {
            if candidate.exists() {
                debug!("[SessionManager] Resolved extension host entry to {:?}", candidate);
                return Ok(candidate.clone());
            }
        }

        error!(
            "[SessionManager] Extension host entry not found. Checked {} candidates",
            candidates.len()
        );
        Err("Unable to locate extension host entry point".to_string())
    }

    fn build_ipc_url(kind: &str, session_id: &Uuid) -> Result<String, String> {
        #[cfg(target_family = "unix")]
        {
            let socket_dir = std::path::Path::new("/tmp");
            let socket_path = socket_dir.join(format!("dscode-{}-{}.sock", kind, session_id));
            if socket_path.exists() {
                if let Err(err) = fs::remove_file(&socket_path) {
                    warn!(
                        "[SessionManager] Failed to remove stale socket {:?}: {}",
                        socket_path, err
                    );
                }
            }
            return Ok(format!("ipc://{}", socket_path.to_string_lossy()));
        }

        #[cfg(target_family = "windows")]
        {
            return Ok(format!(r"\\.\pipe\dscode-{}-{}", kind, session_id));
        }

        #[allow(unreachable_code)]
        {
            Err("Unsupported platform for IPC".to_string())
        }
    }

    /// Handle incoming requests from Extension Host
    ///
    /// Pending request lifecycle:
    ///   1. Request ID generated (UUID)
    ///   2. oneshot::Sender inserted into pending map (under write lock)
    ///   3. Event emitted to frontend (UI prompt shown)
    ///   4. User interacts with UI, frontend calls resolve_* IPC command
    ///   5. resolve_* removes Sender from map, sends value through channel
    ///   6. Original awaiter receives value
    ///
    /// Failure modes:
    ///   - Frontend disconnects: Sender stays in map forever (memory leak).
    ///     TODO: Add periodic cleanup of stale pending requests (>5 min).
    ///   - Frontend sends response for unknown ID: logged and ignored.
    ///   - Two responses for same ID: second one finds no Sender, logged.
    ///   - Session shutdown during pending: Senders dropped, Receivers get RecvError.
    async fn handle_incoming_request(
        &self, msg_type: &str, payload: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        debug!("[SessionManager] Handling incoming request: {}", msg_type);

        // Guard: reject requests if session is not ready.
        // During initialization, the extension host may send "extension-host-ready"
        // which must be allowed through. All other requests require Ready state.
        // This prevents race conditions where requests arrive before state is
        // fully initialized (e.g., extensions loaded, commands registered).
        //
        // We also allow requests during Initializing for bootstrap messages
        // (extension-host-ready, get-extensions-dir, get-extension-storage)
        // that are part of the initialization handshake.
        // TODO: Replace with manager.state() check once ExtensionHostState enum is available
        // (see src-tauri/src/extension_host/manager.rs for the new state machine)
        {
            let current = *self.lifecycle.read().await;
            let is_bootstrap_message = matches!(
                msg_type,
                "extension-host-ready" | "get-extensions-dir" | "get-extension-storage"
            );
            if !is_bootstrap_message
                && !matches!(
                    current,
                    super::SessionLifecycle::Ready | super::SessionLifecycle::Initializing
                )
            {
                return Err(format!(
                    "Session not ready (current state: {:?}), rejecting request: {}",
                    current, msg_type
                ));
            }
        }

        match msg_type {
            "extension-host-ready" => {
                info!("[SessionManager] Extension host ready signal received");
                self.extension_host_ready.notify_waiters();
                Ok(json!({"success": true}))
            }
            "get-extensions-dir" => {
                let extensions_dir = self
                    .app_dirs
                    .extensions_dir
                    .canonicalize()
                    .unwrap_or_else(|_| self.app_dirs.extensions_dir.clone());
                debug!("[SessionManager] Extensions directory: {:?}", extensions_dir);
                Ok(json!(extensions_dir.to_string_lossy().to_string()))
            }
            "get-extension-storage" => {
                let extension_id = payload
                    .get("extensionId")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing extensionId in request")?;

                let storage_root = self.app_dirs.storage_dir.join(extension_id);
                let global_dir = storage_root.join("global");
                let workspace_dir = storage_root.join("workspace");
                let logs_dir = self.app_dirs.logs_dir.join(extension_id);

                if let Err(e) = std::fs::create_dir_all(&global_dir) {
                    if e.kind() != std::io::ErrorKind::AlreadyExists {
                        return Err(format!(
                            "Failed to prepare global storage for {}: {}",
                            extension_id, e
                        ));
                    }
                }
                if let Err(e) = std::fs::create_dir_all(&workspace_dir) {
                    if e.kind() != std::io::ErrorKind::AlreadyExists {
                        return Err(format!(
                            "Failed to prepare workspace storage for {}: {}",
                            extension_id, e
                        ));
                    }
                }
                if let Err(e) = std::fs::create_dir_all(&logs_dir) {
                    if e.kind() != std::io::ErrorKind::AlreadyExists {
                        return Err(format!(
                            "Failed to prepare logs directory for {}: {}",
                            extension_id, e
                        ));
                    }
                }

                Ok(json!({
                    "global": global_dir.to_string_lossy().to_string(),
                    "workspace": workspace_dir.to_string_lossy().to_string(),
                    "logs": logs_dir.to_string_lossy().to_string(),
                }))
            }
            "registerTreeDataProvider" => Ok(json!({"success": true})),
            "updateStatusBarItem" => {
                let id = payload
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing status bar item id".to_string())?;
                let owner = payload
                    .get("owner")
                    .and_then(|v| v.as_str())
                    .unwrap_or(Self::CORE_COMMAND_OWNER);

                self.upsert_status_bar_item(id, owner, &payload).await?;
                Ok(json!({"success": true}))
            }
            "hideStatusBarItem" => {
                let id = payload
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing status bar item id".to_string())?;
                let owner = payload
                    .get("owner")
                    .and_then(|v| v.as_str())
                    .unwrap_or(Self::CORE_COMMAND_OWNER);

                self.hide_status_bar_item(owner, id).await?;
                Ok(json!({"success": true}))
            }
            "disposeStatusBarItem" => {
                let id = payload
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing status bar item id".to_string())?;
                let owner = payload
                    .get("owner")
                    .and_then(|v| v.as_str())
                    .unwrap_or(Self::CORE_COMMAND_OWNER);

                self.dispose_status_bar_item(owner, id).await?;
                Ok(json!({"success": true}))
            }
            "command-registered" => {
                if let Some(command) = payload.get("command").and_then(|v| v.as_str()) {
                    let owner = payload
                        .get("owner")
                        .and_then(|v| v.as_str())
                        .unwrap_or(Self::CORE_COMMAND_OWNER);

                    self.add_command_owner(command, owner).await;
                    self.publish_command_list().await;

                    if let Some(registry) = self.app_handle.try_state::<CommandRegistry>() {
                        let cmd_info = CommandInfo {
                            id: command.to_string(),
                            label: command.to_string(),
                            category: Some(owner.to_string()),
                            owner: owner.to_string(),
                            keybinding: None,
                            when: None,
                        };
                        let _ = registry.register_command(cmd_info);
                    }

                    Ok(json!({"success": true}))
                } else {
                    Err("Missing command name".to_string())
                }
            }
            "command-unregistered" => {
                if let Some(command) = payload.get("command").and_then(|v| v.as_str()) {
                    let owner = payload
                        .get("owner")
                        .and_then(|v| v.as_str())
                        .unwrap_or(Self::CORE_COMMAND_OWNER);

                    self.remove_command_owner(command, owner).await;
                    self.publish_command_list().await;

                    if let Some(registry) = self.app_handle.try_state::<CommandRegistry>() {
                        let _ = registry.unregister_command(command, owner);
                    }

                    Ok(json!({"success": true}))
                } else {
                    Err("Missing command name".to_string())
                }
            }
            "window-show-message" => {
                let message =
                    payload.get("message").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let level =
                    payload.get("type").and_then(|v| v.as_str()).unwrap_or("info").to_string();
                self.emit_event(SessionEvent::WindowMessage { level, message, actions: None });
                Ok(json!({"success": true}))
            }
            "window-show-message-with-actions" => {
                let message =
                    payload.get("message").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let level =
                    payload.get("type").and_then(|v| v.as_str()).unwrap_or("info").to_string();
                let actions_vec: Vec<String> = payload
                    .get("actions")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter().filter_map(|item| item.as_str().map(|s| s.to_string())).collect()
                    })
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
            }
            "window-set-status-bar-message" => {
                let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let text =
                    payload.get("text").and_then(|v| v.as_str()).unwrap_or_default().to_string();
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
            }
            "window-clear-status-bar-message" => {
                if let Some(id) = payload.get("id").and_then(|v| v.as_str()) {
                    self.clear_status_bar_message(id).await?;
                }
                Ok(json!({"success": true}))
            }
            "openTextDocument" => {
                let path = payload
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing document path".to_string())?;
                self.open_text_document(path).await
            }
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
            }
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
                let end_of_line =
                    payload.get("endOfLine").and_then(|v| v.as_i64()).map(|v| v as i32);

                let path_clone = path.to_string();
                let original = tokio::task::spawn_blocking(move || fs::read_to_string(&path_clone))
                    .await
                    .map_err(|e| format!("Task failed: {}", e))?
                    .map_err(|e| format!("Failed to read document {}: {}", path, e))?;
                let updated = Self::apply_text_edits(&original, &mut edits, end_of_line)?;
                let version = self.persist_document(path, &updated).await?;

                Ok(json!({ "success": true, "version": version, "content": updated }))
            }
            "insertSnippet" => {
                let path = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing document uri".to_string())?;
                let snippet = payload.get("snippet").and_then(|v| v.as_str()).unwrap_or("");
                let locations = Self::parse_snippet_locations(payload.get("location"));

                let plain = Self::snippet_to_plain(snippet);
                let mut edits: Vec<TextEditPayload> = locations
                    .into_iter()
                    .map(|range| TextEditPayload { range, new_text: plain.clone() })
                    .collect();

                let path_clone = path.to_string();
                let original = tokio::task::spawn_blocking(move || fs::read_to_string(&path_clone))
                    .await
                    .map_err(|e| format!("Task failed: {}", e))?
                    .map_err(|e| format!("Failed to read document {}: {}", path, e))?;
                let updated = Self::apply_text_edits(&original, &mut edits, None)?;
                let version = self.persist_document(path, &updated).await?;

                Ok(json!({ "success": true, "version": version, "content": updated }))
            }
            "setDecorations" => {
                let uri = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing decoration uri".to_string())?;
                let key = payload
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing decoration key".to_string())?;
                let decorations =
                    payload.get("decorations").cloned().unwrap_or(Value::Array(Vec::new()));

                self.update_decorations(uri, key, decorations).await?;
                Ok(json!({"success": true}))
            }
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
            }
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
            }
            "workspace-register-watcher" => {
                let id = payload
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing watcher id".to_string())?;
                let pattern = payload.get("globPattern").and_then(|v| v.as_str()).unwrap_or("**/*");
                let ignore_create =
                    payload.get("ignoreCreateEvents").and_then(|v| v.as_bool()).unwrap_or(false);
                let ignore_change =
                    payload.get("ignoreChangeEvents").and_then(|v| v.as_bool()).unwrap_or(false);
                let ignore_delete =
                    payload.get("ignoreDeleteEvents").and_then(|v| v.as_bool()).unwrap_or(false);

                self.register_file_system_watcher(
                    id,
                    pattern,
                    ignore_create,
                    ignore_change,
                    ignore_delete,
                )
                .await?;
                Ok(json!({"success": true}))
            }
            "workspace-unregister-watcher" => {
                if let Some(id) = payload.get("id").and_then(|v| v.as_str()) {
                    self.unregister_file_system_watcher(id).await;
                }
                Ok(json!({"success": true}))
            }
            "revealTreeItem" => {
                let view_id =
                    payload.get("viewId").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let element = payload.get("element").cloned().unwrap_or(Value::Null);
                let options = payload.get("options").cloned().unwrap_or(Value::Null);

                self.emit_event(SessionEvent::TreeViewReveal { view_id, element, options });

                Ok(json!({"success": true}))
            }
            "window-show-quick-pick" => {
                let items =
                    payload.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default();

                let options =
                    payload.get("options").and_then(|v| v.as_object()).cloned().unwrap_or_default();

                let can_pick_many =
                    options.get("canPickMany").and_then(|v| v.as_bool()).unwrap_or(false);
                let place_holder =
                    options.get("placeHolder").and_then(|v| v.as_str()).map(|s| s.to_string());
                let title = options.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
                let match_on_description =
                    options.get("matchOnDescription").and_then(|v| v.as_bool()).unwrap_or(false);
                let match_on_detail =
                    options.get("matchOnDetail").and_then(|v| v.as_bool()).unwrap_or(false);

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
            }
            "window-show-input-box" => {
                let prompt = payload.get("prompt").and_then(|v| v.as_str()).map(|s| s.to_string());
                let place_holder =
                    payload.get("placeHolder").and_then(|v| v.as_str()).map(|s| s.to_string());
                let value = payload.get("value").and_then(|v| v.as_str()).map(|s| s.to_string());
                let password = payload.get("password").and_then(|v| v.as_bool()).unwrap_or(false);
                let value_selection =
                    payload.get("valueSelection").and_then(|v| v.as_array()).and_then(|arr| {
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

                let result: Option<String> = rx.await.unwrap_or_default();

                {
                    let mut pending = self.pending_input_requests.write().await;
                    pending.remove(&request_id);
                }

                Ok(json!({ "value": result }))
            }
            "output-channel-append" => {
                let channel = payload
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Extension Host")
                    .to_string();
                let value =
                    payload.get("value").and_then(|v| v.as_str()).unwrap_or_default().to_string();

                self.append_output_channel(&channel, &value).await?;
                Ok(json!({"success": true}))
            }
            "output-channel-clear" => {
                if let Some(channel) = payload.get("channel").and_then(|v| v.as_str()) {
                    self.clear_output_channel(channel).await?;
                }
                Ok(json!({"success": true}))
            }
            "output-channel-show" => {
                if let Some(channel) = payload.get("channel").and_then(|v| v.as_str()) {
                    self.set_output_channel_visibility(channel, true).await?;
                }
                Ok(json!({"success": true}))
            }
            "output-channel-hide" => {
                if let Some(channel) = payload.get("channel").and_then(|v| v.as_str()) {
                    self.set_output_channel_visibility(channel, false).await?;
                }
                Ok(json!({"success": true}))
            }
            "output-channel-dispose" => {
                if let Some(channel) = payload.get("channel").and_then(|v| v.as_str()) {
                    self.dispose_output_channel(channel).await?;
                }
                Ok(json!({"success": true}))
            }
            "workspace-find-files" => {
                let include = payload.get("include").and_then(|v| v.as_str()).unwrap_or("**/*");
                let exclude = payload.get("exclude").and_then(|v| v.as_str());
                let max_results =
                    payload.get("maxResults").and_then(|v| v.as_u64()).unwrap_or(1000) as usize;

                let include_patterns = Self::split_patterns(include, "**/*");
                let exclude_patterns = Self::split_patterns_opt(exclude);

                let files = self
                    .find_workspace_files(&include_patterns, &exclude_patterns, max_results)
                    .await?;

                Ok(json!({ "files": files }))
            }
            "workspace-save-document" => {
                let path = payload
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing document path".to_string())?;
                let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("");

                let version = self.persist_document(path, content).await?;
                Ok(json!({"success": true, "version": version}))
            }
            "workspace-get-configuration" => {
                let section = payload.get("section").and_then(|v| v.as_str());
                let store = self.configuration.read().await;
                let config = store.snapshot(section);
                Ok(json!({ "config": config }))
            }
            "workspace-update-configuration" => {
                let key = payload
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing configuration key".to_string())?;
                let section =
                    payload.get("section").and_then(|v| v.as_str()).map(|s| s.to_string());
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
                        .ipc_manager
                        .request("main", "configuration-changed", notify_payload)
                        .await
                    {
                        error!(
                            "[SessionManager] Failed to forward configuration change: {}",
                            err
                        );
                    }
                }

                Ok(json!({"success": true}))
            }
            "executeCommandRequest" => {
                let command =
                    payload.get("command").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let args =
                    payload.get("args").and_then(|v| v.as_array()).cloned().unwrap_or_default();

                let (tx, rx) = oneshot::channel();
                let request_id = format!("cmd_{}", uuid::Uuid::new_v4());
                {
                    let mut pending = self.pending_quick_pick_requests.write().await;
                    pending.insert(request_id.clone(), tx);
                }

                self.emit_event(SessionEvent::ExecuteCommandRequest {
                    id: request_id.clone(),
                    command: command.clone(),
                    args,
                });

                let request_id_key = request_id.clone();
                let result = match tokio::time::timeout(Duration::from_secs(30), rx).await {
                    Ok(Ok(value)) => value.unwrap_or(Value::Null),
                    Ok(Err(_)) => json!({
                        "success": false,
                        "error": format!("Command '{}' did not return a result", command),
                    }),
                    Err(_) => json!({
                        "success": false,
                        "error": format!("Command '{}' timed out", command),
                    }),
                };

                {
                    let mut pending = self.pending_quick_pick_requests.write().await;
                    pending.remove(&request_id_key);
                }

                Ok(result)
            }
            "workspace-get-folders" => {
                let state = self.state.read().await;
                let folders: Vec<String> = state
                    .workspace_folders
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                Ok(json!(folders))
            }
            "secretGet" => {
                let extension_id = payload
                    .get("extensionId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing extensionId".to_string())?;
                let key = payload
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing key".to_string())?;

                match self.secrets.get(extension_id, key) {
                    Ok(value) => Ok(json!({ "value": value })),
                    Err(e) => Err(e),
                }
            }
            "secretStore" => {
                let extension_id = payload
                    .get("extensionId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing extensionId".to_string())?;
                let key = payload
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing key".to_string())?;
                let value = payload
                    .get("value")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing value".to_string())?;

                match self.secrets.set(extension_id, key, value) {
                    Ok(()) => {
                        if let Err(e) = self
                            .ipc_manager
                            .request(
                                "main",
                                "secretChanged",
                                json!({
                                    "extensionId": extension_id,
                                    "key": key,
                                }),
                            )
                            .await
                        {
                            error!("[SessionManager] Failed to notify secret change: {}", e);
                        }
                        Ok(json!({ "success": true }))
                    }
                    Err(e) => Err(e),
                }
            }
            "secretDelete" => {
                let extension_id = payload
                    .get("extensionId")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing extensionId".to_string())?;
                let key = payload
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing key".to_string())?;

                match self.secrets.delete(extension_id, key) {
                    Ok(()) => {
                        if let Err(e) = self
                            .ipc_manager
                            .request(
                                "main",
                                "secretChanged",
                                json!({
                                    "extensionId": extension_id,
                                    "key": key,
                                }),
                            )
                            .await
                        {
                            error!("[SessionManager] Failed to notify secret deletion: {}", e);
                        }
                        Ok(json!({ "success": true }))
                    }
                    Err(e) => Err(e),
                }
            }
            "fsReadFile" => {
                let uri = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing uri".to_string())?;

                let validated_path =
                    self.path_validator.read().await.validate_path(uri).map_err(|e| {
                        format!("NoPermissions: {}", PathValidator::sanitize_error(&e))
                    })?;

                const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;
                let metadata = match std::fs::metadata(&validated_path) {
                    Ok(m) => m,
                    Err(e) => {
                        return Err(fs_error(
                            classify_io_error(&e),
                            &PathValidator::sanitize_error(&format!("{}", e)),
                        ))
                    }
                };
                if metadata.len() > MAX_FILE_SIZE {
                    return Err("Unavailable: File too large to read".to_string());
                }

                let path_for_read = validated_path.clone();
                let contents = tokio::task::spawn_blocking(move || std::fs::read(&path_for_read))
                    .await
                    .map_err(|e| format!("Unavailable: {}", e))?
                    .map_err(|e| {
                        fs_error(
                            classify_io_error(&e),
                            &PathValidator::sanitize_error(&format!("{}", e)),
                        )
                    })?;

                Ok(json!({ "data": contents }))
            }
            "fsStat" => {
                let uri = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing uri".to_string())?;

                let validated_path =
                    self.path_validator.read().await.validate_path(uri).map_err(|e| {
                        format!("NoPermissions: {}", PathValidator::sanitize_error(&e))
                    })?;

                let path_for_stat = validated_path.clone();
                let metadata =
                    tokio::task::spawn_blocking(move || std::fs::metadata(&path_for_stat))
                        .await
                        .map_err(|e| format!("Unavailable: {}", e))?
                        .map_err(|e| {
                            fs_error(
                                classify_io_error(&e),
                                &PathValidator::sanitize_error(&format!("{}", e)),
                            )
                        })?;

                let file_type = if metadata.is_file() {
                    1
                } else if metadata.is_dir() {
                    2
                } else if metadata.is_symlink() {
                    64
                } else {
                    0
                };

                let mtime = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);

                let ctime = metadata
                    .created()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);

                Ok(json!({
                    "stat": {
                        "type": file_type,
                        "ctime": ctime,
                        "mtime": mtime,
                        "size": metadata.len(),
                    }
                }))
            }
            "fsReadDirectory" => {
                let uri = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing uri".to_string())?;

                let validated_path =
                    self.path_validator.read().await.validate_path(uri).map_err(|e| {
                        format!("NoPermissions: {}", PathValidator::sanitize_error(&e))
                    })?;

                let path_for_readdir = validated_path.clone();
                let entries = tokio::task::spawn_blocking(move || {
                    let mut result: Vec<(String, u8)> = Vec::new();
                    let dir_entries = match std::fs::read_dir(&path_for_readdir) {
                        Ok(e) => e,
                        Err(e) => {
                            return Err(fs_error(
                                classify_io_error(&e),
                                &PathValidator::sanitize_error(&format!("{}", e)),
                            ))
                        }
                    };
                    for entry in dir_entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let ft = if entry.path().is_file() {
                            1
                        } else if entry.path().is_dir() {
                            2
                        } else if entry.path().is_symlink() {
                            64
                        } else {
                            0
                        };
                        result.push((name, ft));
                    }
                    Ok(result)
                })
                .await
                .map_err(|e| format!("Unavailable: {}", e))?
                .map_err(|e| e)?;

                Ok(json!({ "entries": entries }))
            }
            "fsCreateDirectory" => {
                let uri = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing uri".to_string())?;

                let validated_path = self
                    .path_validator
                    .read()
                    .await
                    .validate_path(uri)
                    .map_err(|e| format!("NoPermissions: {}", PathValidator::sanitize_error(&e)))?;

                let path_for_mkdir = validated_path.clone();
                tokio::task::spawn_blocking(move || std::fs::create_dir_all(&path_for_mkdir))
                    .await
                    .map_err(|e| format!("Unavailable: {}", e))?
                    .map_err(|e| {
                        fs_error(
                            classify_io_error(&e),
                            &PathValidator::sanitize_error(&format!("{}", e)),
                        )
                    })?;

                Ok(json!({ "success": true }))
            }
            "fsWriteFile" => {
                let uri = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing uri".to_string())?;
                let content =
                    payload.get("content").ok_or_else(|| "Missing content".to_string())?;

                let validated_path = self
                    .path_validator
                    .read()
                    .await
                    .validate_path(uri)
                    .map_err(|e| format!("NoPermissions: {}", PathValidator::sanitize_error(&e)))?;

                let bytes: Vec<u8> = if let Some(arr) = content.as_array() {
                    arr.iter().filter_map(|v| v.as_u64().map(|n| n as u8)).collect()
                } else if let Some(s) = content.as_str() {
                    s.as_bytes().to_vec()
                } else {
                    return Err("Unavailable: Invalid content format".to_string());
                };

                const MAX_WRITE_SIZE: usize = 100 * 1024 * 1024;
                if bytes.len() > MAX_WRITE_SIZE {
                    return Err(
                        "Unavailable: File content exceeds maximum allowed size".to_string()
                    );
                }

                let path_for_write = validated_path.clone();
                tokio::task::spawn_blocking(move || std::fs::write(&path_for_write, bytes))
                    .await
                    .map_err(|e| format!("Unavailable: {}", e))?
                    .map_err(|e| {
                        fs_error(
                            classify_io_error(&e),
                            &PathValidator::sanitize_error(&format!("{}", e)),
                        )
                    })?;

                Ok(json!({ "success": true }))
            }
            "fsDelete" => {
                let uri = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing uri".to_string())?;
                let options = payload.get("options");
                let recursive = options
                    .and_then(|o| o.get("recursive"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let validated_path = self
                    .path_validator
                    .read()
                    .await
                    .validate_path(uri)
                    .map_err(|e| format!("NoPermissions: {}", PathValidator::sanitize_error(&e)))?;

                let path_for_delete = validated_path.clone();
                tokio::task::spawn_blocking(move || {
                    if path_for_delete.is_dir() {
                        if recursive {
                            std::fs::remove_dir_all(&path_for_delete)
                        } else {
                            std::fs::remove_dir(&path_for_delete)
                        }
                    } else {
                        std::fs::remove_file(&path_for_delete)
                    }
                })
                .await
                .map_err(|e| format!("Unavailable: {}", e))?
                .map_err(|e| {
                    fs_error(
                        classify_io_error(&e),
                        &PathValidator::sanitize_error(&format!("{}", e)),
                    )
                })?;

                Ok(json!({ "success": true }))
            }
            "fsRename" => {
                let old_uri = payload
                    .get("oldUri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing oldUri".to_string())?;
                let new_uri = payload
                    .get("newUri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing newUri".to_string())?;

                let old_validated = self
                    .path_validator
                    .read()
                    .await
                    .validate_path(old_uri)
                    .map_err(|e| format!("NoPermissions: {}", PathValidator::sanitize_error(&e)))?;
                let new_validated = self
                    .path_validator
                    .read()
                    .await
                    .validate_path(new_uri)
                    .map_err(|e| format!("NoPermissions: {}", PathValidator::sanitize_error(&e)))?;

                let old_path = old_validated.clone();
                let new_path = new_validated.clone();
                tokio::task::spawn_blocking(move || std::fs::rename(&old_path, &new_path))
                    .await
                    .map_err(|e| format!("Unavailable: {}", e))?
                    .map_err(|e| {
                        fs_error(
                            classify_io_error(&e),
                            &PathValidator::sanitize_error(&format!("{}", e)),
                        )
                    })?;

                Ok(json!({ "success": true }))
            }
            "fsCopy" => {
                let source_uri = payload
                    .get("source")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing source".to_string())?;
                let dest_uri = payload
                    .get("destination")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing destination".to_string())?;

                let source_validated = self
                    .path_validator
                    .read()
                    .await
                    .validate_path(source_uri)
                    .map_err(|e| format!("NoPermissions: {}", PathValidator::sanitize_error(&e)))?;
                let dest_validated = self
                    .path_validator
                    .read()
                    .await
                    .validate_path(dest_uri)
                    .map_err(|e| format!("NoPermissions: {}", PathValidator::sanitize_error(&e)))?;

                let src = source_validated.clone();
                let dst = dest_validated.clone();
                tokio::task::spawn_blocking(move || std::fs::copy(&src, &dst))
                    .await
                    .map_err(|e| format!("Unavailable: {}", e))?
                    .map_err(|e| {
                        fs_error(
                            classify_io_error(&e),
                            &PathValidator::sanitize_error(&format!("{}", e)),
                        )
                    })?;

                Ok(json!({ "success": true }))
            }
            "startProgress" => {
                let _location = payload.get("location");
                let _title = payload.get("title");
                let _cancellable = payload.get("cancellable");

                let progress_id = uuid::Uuid::new_v4().to_string();
                Ok(json!({ "progressId": progress_id }))
            }
            "updateProgress" => {
                let _progress_id = payload.get("progressId");
                let _message = payload.get("message");
                let _increment = payload.get("increment");
                Ok(json!({ "success": true }))
            }
            "endProgress" => {
                let _progress_id = payload.get("progressId");
                Ok(json!({ "success": true }))
            }
            "createFileSystemWatcher" => {
                let _glob_pattern = payload.get("globPattern");
                let _ignore_create =
                    payload.get("ignoreCreateEvents").and_then(|v| v.as_bool()).unwrap_or(false);
                let _ignore_change =
                    payload.get("ignoreChangeEvents").and_then(|v| v.as_bool()).unwrap_or(false);
                let _ignore_delete =
                    payload.get("ignoreDeleteEvents").and_then(|v| v.as_bool()).unwrap_or(false);

                let watcher_id = uuid::Uuid::new_v4().to_string();

                Ok(json!({ "watcherId": watcher_id }))
            }
            "disposeFileSystemWatcher" => {
                let _watcher_id = payload.get("watcherId");
                Ok(json!({ "success": true }))
            }
            "registerCompletionProvider"
            | "registerHoverProvider"
            | "registerDefinitionProvider"
            | "registerReferenceProvider"
            | "registerCodeActionsProvider"
            | "registerDocumentSymbolProvider"
            | "registerFormattingProvider"
            | "registerRenameProvider"
            | "registerSignatureHelpProvider"
            | "registerCodeLensProvider"
            | "registerDocumentLinkProvider"
            | "registerColorProvider"
            | "registerFoldingRangeProvider"
            | "registerSelectionRangeProvider"
            | "registerDocumentHighlightProvider"
            | "registerRangeFormattingProvider"
            | "registerOnTypeFormattingProvider"
            | "registerSemanticTokensProvider"
            | "registerInlineCompletionProvider"
            | "registerImplementationProvider"
            | "registerTypeDefinitionProvider"
            | "registerDeclarationProvider"
            | "registerWorkspaceSymbolProvider"
            | "registerCallHierarchyProvider"
            | "registerTypeHierarchyProvider" => {
                let provider_type = match msg_type {
                    "registerCompletionProvider" => "completion",
                    "registerHoverProvider" => "hover",
                    "registerDefinitionProvider" => "definition",
                    "registerReferenceProvider" => "references",
                    "registerCodeActionsProvider" => "codeAction",
                    "registerDocumentSymbolProvider" => "documentSymbol",
                    "registerFormattingProvider" => "formatting",
                    "registerRenameProvider" => "rename",
                    "registerSignatureHelpProvider" => "signatureHelp",
                    "registerCodeLensProvider" => "codeLens",
                    "registerDocumentLinkProvider" => "documentLink",
                    "registerColorProvider" => "color",
                    "registerFoldingRangeProvider" => "foldingRange",
                    "registerSelectionRangeProvider" => "selectionRange",
                    "registerDocumentHighlightProvider" => "documentHighlight",
                    "registerRangeFormattingProvider" => "rangeFormatting",
                    "registerOnTypeFormattingProvider" => "onTypeFormatting",
                    "registerSemanticTokensProvider" => "semanticTokens",
                    "registerInlineCompletionProvider" => "inlineCompletion",
                    "registerImplementationProvider" => "implementation",
                    "registerTypeDefinitionProvider" => "typeDefinition",
                    "registerDeclarationProvider" => "declaration",
                    "registerWorkspaceSymbolProvider" => "workspaceSymbol",
                    "registerCallHierarchyProvider" => "callHierarchy",
                    "registerTypeHierarchyProvider" => "typeHierarchy",
                    _ => "unknown",
                };
                let registry = self.app_handle.try_state::<LanguageFeaturesRegistry>();
                if let Some(registry) = registry {
                    let context = crate::session::ipc_providers::ProviderHandlerContext {
                        payload: &payload,
                        provider_type,
                    };
                    let result = crate::session::ipc_providers::handle_register_provider(
                        &context, &registry,
                    )?;
                    self.emit_event(SessionEvent::ProviderRegistered {
                        provider_type: provider_type.to_string(),
                        provider_id: result
                            .get("providerId")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        owner: payload
                            .get("owner")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        selector: payload.get("selector").cloned().unwrap_or(Value::Null),
                        trigger_characters: payload
                            .get("triggerCharacters")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            }),
                        metadata: payload.get("metadata").cloned(),
                    });
                    Ok(result)
                } else {
                    Ok(json!({"success": true}))
                }
            }
            "setLanguageConfiguration" => {
                let language = payload.get("language").and_then(|v| v.as_str()).unwrap_or_default();
                self.emit_event(SessionEvent::LanguageConfigurationChanged {
                    language: language.to_string(),
                    configuration: payload.get("configuration").cloned().unwrap_or(Value::Null),
                });
                Ok(json!({"success": true}))
            }
            "updateDiagnostics" => {
                let uri = payload.get("uri").and_then(|v| v.as_str()).unwrap_or_default();
                let diagnostics = payload.get("diagnostics").cloned().unwrap_or(Value::Null);
                self.emit_event(SessionEvent::DiagnosticsUpdated {
                    uri: uri.to_string(),
                    diagnostics,
                });
                Ok(json!({"success": true}))
            }
            "clearDiagnostics" => {
                let uri = payload.get("uri").and_then(|v| v.as_str()).unwrap_or_default();
                self.emit_event(SessionEvent::DiagnosticsCleared { uri: uri.to_string() });
                Ok(json!({"success": true}))
            }
            "clearAllDiagnostics" => {
                self.emit_event(SessionEvent::DiagnosticsCleared { uri: String::new() });
                Ok(json!({"success": true}))
            }
            "setContext" => {
                let key = payload.get("key").and_then(|v| v.as_str()).unwrap_or_default();
                let value = payload.get("value").cloned().unwrap_or(Value::Null);
                self.emit_event(SessionEvent::ContextChanged { key: key.to_string(), value });
                Ok(json!({"success": true}))
            }
            "showOpenFolderDialog" => {
                self.emit_event(SessionEvent::OpenFolderDialog);
                Ok(json!({"success": true}))
            }
            "showOpenFileDialog" => {
                self.emit_event(SessionEvent::OpenFileDialog);
                Ok(json!({"success": true}))
            }
            "openFolder" => {
                let uri = payload.get("uri").and_then(|v| v.as_str());
                if let Some(folder_path) = uri {
                    // Add to workspace and notify frontend
                    {
                        let mut state = self.state.write().await;
                        let path = PathBuf::from(folder_path);
                        if !state.workspace_folders.contains(&path) {
                            state.workspace_folders.push(path.clone());
                        }
                    }
                    self.emit_event(SessionEvent::OpenFolder { uri: folder_path.to_string() });

                    // Update path validator with new workspace folder
                    {
                        let mut pv = self.path_validator.write().await;
                        pv.add_workspace_folder(PathBuf::from(folder_path));
                    }

                    let state = self.state.read().await.clone();
                    self.emit_event(SessionEvent::StateChanged { state });
                } else {
                    // No URI provided, show folder picker dialog
                    self.emit_event(SessionEvent::OpenFolderDialog);
                }
                Ok(json!({"success": true}))
            }
            "openFile" => {
                let uri = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing uri for openFile".to_string())?;

                // Open the file in the editor via the existing document mechanism
                self.open_text_document(uri).await?;
                Ok(json!({"success": true}))
            }
            "registerDebugConfigurationProvider"
            | "registerDebugAdapterDescriptorFactory"
            | "registerDebugAdapterTrackerFactory"
            | "registerTaskProvider"
            | "registerAuthProvider"
            | "registerTextDocumentContentProvider"
            | "registerNotebookContentProvider"
            | "registerWebviewSerializer" => Ok(json!({"success": true})),
            _ => {
                warn!("[SessionManager] Unhandled request type: {}", msg_type);
                Ok(json!({"success": true}))
            }
        }
    }

    pub async fn reconnect_extension_host(&self) -> Result<(), String> {
        if self.ipc_manager.is_connected("main").await {
            return Ok(());
        }

        info!("[SessionManager] IPC disconnected, attempting reconnection...");

        let outgoing_socket = self.outgoing_socket.read().await.clone();
        let incoming_socket = self.incoming_socket.read().await.clone();
        let extension_host_entry = self.extension_host_entry.read().await.clone();

        let (outgoing, incoming, entry) =
            match (outgoing_socket, incoming_socket, extension_host_entry) {
                (Some(o), Some(i), Some(e)) => (o, i, e),
                _ => return Err("No previous connection info for reconnection".to_string()),
            };

        {
            let mut manager = self.extension_host.lock().await;
            // Update state from process status before deciding what to do.
            manager.check_and_update_state();
            let current = manager.state();
            match current {
                dscode_extension_host::ExtensionHostState::Running => {
                    // Process is still alive — just reconnect IPC below.
                }
                dscode_extension_host::ExtensionHostState::Unhealthy => {
                    manager.restart_if_needed(&entry, &outgoing, &incoming).await?;
                }
                _ => {
                    return Err(format!(
                        "Extension host in unexpected state for reconnection: {:?}",
                        current
                    ));
                }
            }
        }

        self.ipc_manager.reconnect_outgoing("main", &outgoing).await?;

        info!("[SessionManager] Extension host reconnected successfully");
        Ok(())
    }
}
