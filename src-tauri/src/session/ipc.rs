use super::{SessionEvent, SessionManager, TextEditPayload};
use crate::commands::{CommandInfo, CommandRegistry};
use crate::extension_host::IncomingRequestHandler;
use crate::extension_host::path_validator::PathValidator;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::path::BaseDirectory;
use tauri::Manager;
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

impl SessionManager {
    /// Start the Extension Host with bidirectional NNG IPC
    pub(super) async fn start_extension_host(&self) -> Result<(), String> {
        {
            let mut manager = self.extension_host.lock().await;
            if manager.is_running() {
                println!("[SessionManager] Extension Host already running, skipping");
                return Ok(());
            }
        }

        let session_id = Uuid::new_v4();
        let outgoing_ipc_url = Self::build_ipc_url("ext-out", &session_id)?;
        let incoming_ipc_url = Self::build_ipc_url("ext-in", &session_id)?;

        let session_manager = self.clone_for_handler();
        let handler: IncomingRequestHandler =
            Arc::new(move |msg_type: String, payload: serde_json::Value| {
                let sm = session_manager.clone();
                Box::pin(async move { sm.handle_incoming_request(&msg_type, payload).await })
            });

        self.nng_manager
            .setup_incoming("main", &incoming_ipc_url, handler)
            .await?;

        let extension_host_main = self.resolve_extension_host_entry()?;
        let extension_host_main_str = extension_host_main.to_string_lossy().to_string();

        let mut manager = self.extension_host.lock().await;
        manager.start_with_nng(
            &extension_host_main_str,
            &outgoing_ipc_url,
            &incoming_ipc_url,
        )?;
        drop(manager);

        const MAX_ATTEMPTS: u8 = 10;
        for attempt in 1..=MAX_ATTEMPTS {
            if self.nng_manager.is_connected("main").await {
                break;
            }

            match self
                .nng_manager
                .connect_outgoing("main", &outgoing_ipc_url)
                .await
            {
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
    pub(super) fn clone_for_handler(&self) -> Arc<Self> {
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
            workspace_configurations: Arc::clone(&self.workspace_configurations),
            file_decoration_providers: Arc::clone(&self.file_decoration_providers),
            file_decorations: Arc::clone(&self.file_decorations),
            extension_host_ready: Arc::clone(&self.extension_host_ready),
            extension_host_ready_flag: Arc::clone(&self.extension_host_ready_flag),
            initialized: Arc::clone(&self.initialized),
            secrets: Arc::clone(&self.secrets),
            path_validator: Arc::clone(&self.path_validator),
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
                println!(
                    "[SessionManager] Resolved extension host entry to {:?}",
                    candidate
                );
                return Ok(candidate.clone());
            }
        }

        eprintln!(
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
                    eprintln!(
                        "[SessionManager] Failed to remove stale socket {:?}: {}",
                        socket_path, err
                    );
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
    async fn handle_incoming_request(
        &self,
        msg_type: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        println!("[SessionManager] Handling incoming request: {}", msg_type);

        match msg_type {
            "extension-host-ready" => {
                println!("[SessionManager] Extension host ready signal received");
                *self.extension_host_ready_flag.write().await = true;
                self.extension_host_ready.notify_waiters();
                Ok(json!({"success": true}))
            }
            "get-extensions-dir" => {
                let extensions_dir = self
                    .app_dirs
                    .extensions_dir
                    .canonicalize()
                    .unwrap_or_else(|_| self.app_dirs.extensions_dir.clone());
                println!(
                    "[SessionManager] Extensions directory: {:?}",
                    extensions_dir
                );
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
                let message = payload
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let level = payload
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("info")
                    .to_string();
                self.emit_event(SessionEvent::WindowMessage {
                    level,
                    message,
                    actions: None,
                });
                Ok(json!({"success": true}))
            }
            "window-show-message-with-actions" => {
                let message = payload
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let level = payload
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("info")
                    .to_string();
                let actions_vec: Vec<String> = payload
                    .get("actions")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|item| item.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                let has_actions = !actions_vec.is_empty();

                self.emit_event(SessionEvent::WindowMessage {
                    level: level.clone(),
                    message: message.clone(),
                    actions: if has_actions {
                        Some(actions_vec.clone())
                    } else {
                        None
                    },
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
                let end_of_line = payload
                    .get("endOfLine")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);

                let original = fs::read_to_string(path)
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
                let decorations = payload
                    .get("decorations")
                    .cloned()
                    .unwrap_or(Value::Array(Vec::new()));

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
                let options = payload
                    .get("options")
                    .cloned()
                    .unwrap_or(Value::Object(Map::new()));
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
                    self.unregister_file_system_watcher(id);
                }
                Ok(json!({"success": true}))
            }
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
            }
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
            }
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
                let value = payload
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();

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
                let include = payload
                    .get("include")
                    .and_then(|v| v.as_str())
                    .unwrap_or("**/*");
                let exclude = payload.get("exclude").and_then(|v| v.as_str());
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
            }
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
                let section = payload
                    .get("section")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
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
                        eprintln!(
                            "[SessionManager] Failed to forward configuration change: {}",
                            err
                        );
                    }
                }

                Ok(json!({"success": true}))
            }
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
                            .nng_manager
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
                            eprintln!("[SessionManager] Failed to notify secret change: {}", e);
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
                            .nng_manager
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
                            eprintln!("[SessionManager] Failed to notify secret deletion: {}", e);
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

                let validated_path = self.path_validator.read().await.validate_path(uri)
                    .map_err(|e| PathValidator::sanitize_error(&e))?;

                // Limit file size to 50MB to prevent OOM
                const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;
                let metadata = std::fs::metadata(&validated_path)
                    .map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))?;
                if metadata.len() > MAX_FILE_SIZE {
                    return Err("File too large to read".to_string());
                }

                let contents = tokio::task::spawn_blocking(move || std::fs::read(&validated_path))
                    .await
                    .map_err(|e| format!("Task failed: {}", e))?
                    .map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))?;

                Ok(json!({ "data": contents }))
            }
            "fsStat" => {
                let uri = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing uri".to_string())?;

                let validated_path = self.path_validator.read().await.validate_path(uri)
                    .map_err(|e| PathValidator::sanitize_error(&e))?;

                let metadata = tokio::task::spawn_blocking(move || std::fs::metadata(&validated_path))
                    .await
                    .map_err(|e| format!("Task failed: {}", e))?
                    .map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))?;

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

                let validated_path = self.path_validator.read().await.validate_path(uri)
                    .map_err(|e| PathValidator::sanitize_error(&e))?;

                let entries = tokio::task::spawn_blocking(move || {
                    let mut result: Vec<(String, u8)> = Vec::new();
                    let dir_entries = match std::fs::read_dir(&validated_path) {
                        Ok(e) => e,
                        Err(e) => return Err(format!("{}", e)),
                    };
                    for entry in dir_entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let ft = if entry.path().is_file() { 1 }
                            else if entry.path().is_dir() { 2 }
                            else if entry.path().is_symlink() { 64 }
                            else { 0 };
                        result.push((name, ft));
                    }
                    Ok(result)
                })
                .await
                .map_err(|e| format!("Task failed: {}", e))?
                .map_err(|e| PathValidator::sanitize_error(&e))?;

                Ok(json!({ "entries": entries }))
            }
            "fsCreateDirectory" => {
                let uri = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing uri".to_string())?;

                let validated_path = self.path_validator.read().await.validate_path(uri)
                    .map_err(|e| PathValidator::sanitize_error(&e))?;

                tokio::task::spawn_blocking(move || std::fs::create_dir_all(&validated_path))
                    .await
                    .map_err(|e| format!("Task failed: {}", e))?
                    .map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))?;

                Ok(json!({ "success": true }))
            }
            "fsWriteFile" => {
                let uri = payload
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing uri".to_string())?;
                let content = payload
                    .get("content")
                    .ok_or_else(|| "Missing content".to_string())?;

                let validated_path = self.path_validator.read().await.validate_path(uri)
                    .map_err(|e| PathValidator::sanitize_error(&e))?;

                let bytes: Vec<u8> = if let Some(arr) = content.as_array() {
                    arr.iter()
                        .filter_map(|v| v.as_u64().map(|n| n as u8))
                        .collect()
                } else if let Some(s) = content.as_str() {
                    s.as_bytes().to_vec()
                } else {
                    return Err("Invalid content format".to_string());
                };

                // Limit write size to 100MB
                const MAX_WRITE_SIZE: usize = 100 * 1024 * 1024;
                if bytes.len() > MAX_WRITE_SIZE {
                    return Err("File content exceeds maximum allowed size".to_string());
                }

                tokio::task::spawn_blocking(move || std::fs::write(&validated_path, bytes))
                    .await
                    .map_err(|e| format!("Task failed: {}", e))?
                    .map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))?;

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

                let validated_path = self.path_validator.read().await.validate_path(uri)
                    .map_err(|e| PathValidator::sanitize_error(&e))?;

                tokio::task::spawn_blocking(move || {
                    if validated_path.is_dir() {
                        if recursive {
                            std::fs::remove_dir_all(&validated_path)
                        } else {
                            std::fs::remove_dir(&validated_path)
                        }
                    } else {
                        std::fs::remove_file(&validated_path)
                    }
                })
                .await
                .map_err(|e| format!("Task failed: {}", e))?
                .map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))?;

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

                let old_validated = self.path_validator.read().await.validate_path(old_uri)
                    .map_err(|e| PathValidator::sanitize_error(&e))?;
                let new_validated = self.path_validator.read().await.validate_path(new_uri)
                    .map_err(|e| PathValidator::sanitize_error(&e))?;

                tokio::task::spawn_blocking(move || std::fs::rename(&old_validated, &new_validated))
                    .await
                    .map_err(|e| format!("Task failed: {}", e))?
                    .map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))?;

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

                let source_validated = self.path_validator.read().await.validate_path(source_uri)
                    .map_err(|e| PathValidator::sanitize_error(&e))?;
                let dest_validated = self.path_validator.read().await.validate_path(dest_uri)
                    .map_err(|e| PathValidator::sanitize_error(&e))?;

                tokio::task::spawn_blocking(move || std::fs::copy(&source_validated, &dest_validated))
                    .await
                    .map_err(|e| format!("Task failed: {}", e))?
                    .map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))?;

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
                let _ignore_create = payload
                    .get("ignoreCreateEvents")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let _ignore_change = payload
                    .get("ignoreChangeEvents")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let _ignore_delete = payload
                    .get("ignoreDeleteEvents")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let watcher_id = uuid::Uuid::new_v4().to_string();

                Ok(json!({ "watcherId": watcher_id }))
            }
            "disposeFileSystemWatcher" => {
                let _watcher_id = payload.get("watcherId");
                Ok(json!({ "success": true }))
            }
            _ => {
                println!("[SessionManager] Unhandled request type: {}", msg_type);
                Err(format!("Unhandled request type: {}", msg_type))
            }
        }
    }
}
