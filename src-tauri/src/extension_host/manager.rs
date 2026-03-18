/**
 * Extension Host Manager (Refactored with Security)
 *
 * Manages the Node.js extension host process and IPC communication via stdio.
 * Now includes: permissions, path validation, rate limiting, timeouts, sandboxing, secret storage.
 */

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::permissions::{Permission, ExtensionPermissions};
use super::path_validator::PathValidator;
use super::secrets::SecretStorage;
use super::rate_limiter::RateLimiter;
use super::sandbox::{SandboxConfig, apply_sandbox};

// Maximum message size: 10MB
const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

// Request timeout: 30 seconds
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPCMessage {
    pub id: String,
    pub r#type: String,
    pub payload: Value,
}

struct PendingRequest {
    callback: Box<dyn FnOnce(Result<Value, String>) + Send>,
    timestamp: Instant,
}

type ResponseCallback = Box<dyn FnOnce(Result<Value, String>) + Send>;

pub struct ExtensionHostManager {
    child: Option<Child>,
    pending_requests: Arc<Mutex<HashMap<String, PendingRequest>>>,
    message_id: Arc<Mutex<u64>>,
    stdin: Arc<Mutex<Option<std::process::ChildStdin>>>,

    // Security components
    path_validator: Arc<Mutex<PathValidator>>,
    permissions: Arc<Mutex<ExtensionPermissions>>,
    secret_storage: Arc<SecretStorage>,
    rate_limiter: Arc<RateLimiter>,
    pub extension_id: String,

    // Command registry
    registered_commands: Arc<RwLock<Vec<String>>>,
}

impl ExtensionHostManager {
    pub fn new(extension_id: String) -> Self {
        Self {
            child: None,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            message_id: Arc::new(Mutex::new(0)),
            stdin: Arc::new(Mutex::new(None)),
            path_validator: Arc::new(Mutex::new(PathValidator::new())),
            permissions: Arc::new(Mutex::new(ExtensionPermissions::new(extension_id.clone(), vec![]))),
            secret_storage: Arc::new(SecretStorage::new()),
            rate_limiter: Arc::new(RateLimiter::new()),
            extension_id,
            registered_commands: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn set_permissions(&mut self, permissions: ExtensionPermissions) {
        *self.permissions.lock().unwrap() = permissions;
    }

    pub fn add_workspace_folder(&mut self, path: std::path::PathBuf) {
        self.path_validator.lock().unwrap().add_workspace_folder(path);
    }

    /// Start the extension host process with sandboxing
    pub fn start(&mut self, extension_host_path: &str) -> Result<(), String> {
        self.start_with_ipc(extension_host_path, "")
    }

    /// Start the extension host process with NNG IPC (bidirectional)
    pub fn start_with_ipc(&mut self, extension_host_path: &str, ipc_url: &str) -> Result<(), String> {
        self.start_with_bidirectional_ipc(extension_host_path, ipc_url, "")
    }

    /// Start the extension host process with bidirectional NNG IPC
    pub fn start_with_bidirectional_ipc(&mut self, extension_host_path: &str, outgoing_ipc_url: &str, incoming_ipc_url: &str) -> Result<(), String> {
        println!("[ExtensionHost] Starting extension host from: {}", extension_host_path);

        let mut cmd = Command::new("node");
        cmd.arg(extension_host_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set outgoing IPC URL (ExtHost listens, Tauri connects)
        if !outgoing_ipc_url.is_empty() {
            cmd.env("DSCODE_IPC_URL", outgoing_ipc_url);
            println!("[ExtensionHost] Set DSCODE_IPC_URL={}", outgoing_ipc_url);
        }

        // Set incoming IPC URL (Tauri listens, ExtHost connects)
        if !incoming_ipc_url.is_empty() {
            cmd.env("DSCODE_INCOMING_IPC_URL", incoming_ipc_url);
            println!("[ExtensionHost] Set DSCODE_INCOMING_IPC_URL={}", incoming_ipc_url);
        }

        // Apply sandboxing
        let sandbox_config = SandboxConfig::default();
        apply_sandbox(&mut cmd, &sandbox_config)?;

        let mut child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn extension host: {}", e))?;

        // Get stdin for sending messages
        let stdin = child.stdin.take()
            .ok_or("Failed to capture stdin")?;

        *self.stdin.lock().unwrap() = Some(stdin);

        // Get stdout for reading messages
        let stdout = child.stdout.take()
            .ok_or("Failed to capture stdout")?;

        // Get stderr for logging
        let stderr = child.stderr.take()
            .ok_or("Failed to capture stderr")?;

        // Spawn thread to read stderr
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("[ExtensionHost] {}", line);
                }
            }
        });

        // Spawn thread to read stdout (IPC messages)
        let pending_requests = Arc::clone(&self.pending_requests);
        let stdin_handle = Arc::clone(&self.stdin);
        let path_validator = Arc::clone(&self.path_validator);
        let permissions = Arc::clone(&self.permissions);
        let secret_storage = Arc::clone(&self.secret_storage);
        let rate_limiter = Arc::clone(&self.rate_limiter);
        let extension_id = self.extension_id.clone();
        let registered_commands = Arc::clone(&self.registered_commands);

        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            let mut buffer = String::new();

            for line in reader.lines() {
                if let Ok(line) = line {
                    if line.trim().is_empty() {
                        continue;
                    }

                    // Check message size limit
                    if line.len() > MAX_MESSAGE_SIZE {
                        eprintln!("[ExtensionHost] Message too large: {} bytes", line.len());
                        continue;
                    }

                    buffer.push_str(&line);

                    if let Err(e) = Self::handle_message(
                        &buffer,
                        &pending_requests,
                        &stdin_handle,
                        &path_validator,
                        &permissions,
                        &secret_storage,
                        &rate_limiter,
                        &extension_id,
                        &registered_commands,
                    ) {
                        eprintln!("[ExtensionHost] Error handling message: {}", e);
                    }

                    buffer.clear();
                }
            }
        });

        // Start timeout cleanup thread
        let pending_requests_cleanup = Arc::clone(&self.pending_requests);
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(10));
                Self::cleanup_expired_requests(&pending_requests_cleanup);
            }
        });

        self.child = Some(child);
        println!("[ExtensionHost] Started successfully");

        Ok(())
    }

    /// Cleanup expired requests
    fn cleanup_expired_requests(pending_requests: &Arc<Mutex<HashMap<String, PendingRequest>>>) {
        let mut requests = pending_requests.lock().unwrap();
        let now = Instant::now();

        requests.retain(|id, req| {
            if now.duration_since(req.timestamp) > REQUEST_TIMEOUT {
                println!("[ExtensionHost] Request {} timed out", id);
                false
            } else {
                true
            }
        });
    }

    /// Handle incoming message from extension host
    #[allow(clippy::too_many_arguments)]
    fn handle_message(
        line: &str,
        pending_requests: &Arc<Mutex<HashMap<String, PendingRequest>>>,
        stdin: &Arc<Mutex<Option<std::process::ChildStdin>>>,
        path_validator: &Arc<Mutex<PathValidator>>,
        permissions: &Arc<Mutex<ExtensionPermissions>>,
        secret_storage: &Arc<SecretStorage>,
        rate_limiter: &Arc<RateLimiter>,
        extension_id: &str,
        registered_commands: &Arc<RwLock<Vec<String>>>,
    ) -> Result<(), String> {
        let message: IPCMessage = serde_json::from_str(line)
            .map_err(|e| format!("Failed to parse message: {}", e))?;

        // Check if this is a response to a pending request
        let mut requests = pending_requests.lock().unwrap();
        if let Some(pending) = requests.remove(&message.id) {
            drop(requests);

            if message.r#type.ends_with("-error") {
                let error = message.payload.get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();
                (pending.callback)(Err(error));
            } else {
                (pending.callback)(Ok(message.payload));
            }
        } else {
            drop(requests);

            // This is a request from the extension host
            // Check rate limit
            rate_limiter.check_rate_limit(extension_id)?;

            // Handle in background thread to not block IPC
            let msg = message.clone();
            let stdin_clone = Arc::clone(stdin);
            let validator_clone = Arc::clone(path_validator);
            let perms_clone = Arc::clone(permissions);
            let secrets_clone = Arc::clone(secret_storage);
            let ext_id = extension_id.to_string();

            let cmds_clone = Arc::clone(registered_commands);
            tokio::spawn(async move {
                if let Err(e) = Self::handle_request(
                    &msg,
                    &stdin_clone,
                    &validator_clone,
                    &perms_clone,
                    &secrets_clone,
                    &ext_id,
                    &cmds_clone,
                ).await {
                    eprintln!("[ExtensionHost] Request error: {}", e);

                    // Send error response
                    let error_response = IPCMessage {
                        id: msg.id.clone(),
                        r#type: format!("{}-error", msg.r#type),
                        payload: serde_json::json!({ "error": e }),
                    };
                    let _ = Self::send_response(error_response, &stdin_clone);
                }
            });
        }

        Ok(())
    }

    /// Handle a request from the extension host (async with spawn_blocking for I/O)
    #[allow(clippy::too_many_arguments)]
    async fn handle_request(
        message: &IPCMessage,
        stdin: &Arc<Mutex<Option<std::process::ChildStdin>>>,
        path_validator: &Arc<Mutex<PathValidator>>,
        permissions: &Arc<Mutex<ExtensionPermissions>>,
        secret_storage: &Arc<SecretStorage>,
        extension_id: &str,
        registered_commands: &Arc<RwLock<Vec<String>>>,
    ) -> Result<(), String> {
        let response_payload = match message.r#type.as_str() {
            "ready" => {
                println!("[ExtensionHost] Extension host is ready");
                serde_json::json!({ "status": "ok" })
            },

            // Command registration
            "command-registered" => {
                let command = message.payload.get("command")
                    .and_then(|c| c.as_str())
                    .ok_or("Missing command name")?;

                registered_commands.write().unwrap().push(command.to_string());
                println!("[ExtensionHost] Registered command: {}", command);
                serde_json::json!({ "status": "ok" })
            },

            // ========== File System Operations (with validation) ==========
            "fsStat" => {
                permissions.lock().unwrap().check_permission(&Permission::FileSystemRead)?;

                let uri = message.payload.get("uri")
                    .and_then(|u| u.as_str())
                    .ok_or("Missing uri")?;

                let path = {
                    let validator = path_validator.lock().unwrap();
                    validator.validate_path(uri)?
                };

                tokio::task::spawn_blocking(move || {
                    match std::fs::metadata(&path) {
                        Ok(metadata) => {
                            let file_type = if metadata.is_dir() { 2 } else if metadata.is_file() { 1 } else { 0 };
                            Ok(serde_json::json!({
                                "stat": {
                                    "type": file_type,
                                    "ctime": 0,
                                    "mtime": 0,
                                    "size": metadata.len(),
                                    "permissions": if metadata.permissions().readonly() { 1 } else { 0 }
                                }
                            }))
                        },
                        Err(e) => Err(PathValidator::sanitize_error(&format!("Failed to stat file: {}", e))),
                    }
                }).await.map_err(|e| e.to_string())??
            },

            "fsReadFile" => {
                permissions.lock().unwrap().check_permission(&Permission::FileSystemRead)?;

                let uri = message.payload.get("uri")
                    .and_then(|u| u.as_str())
                    .ok_or("Missing uri")?;

                let path = {
                    let validator = path_validator.lock().unwrap();
                    validator.validate_path(uri)?
                };

                tokio::task::spawn_blocking(move || {
                    match std::fs::read(&path) {
                        Ok(data) => Ok(serde_json::json!({ "data": data })),
                        Err(e) => Err(PathValidator::sanitize_error(&format!("Failed to read file: {}", e))),
                    }
                }).await.map_err(|e| e.to_string())??
            },

            "fsWriteFile" => {
                permissions.lock().unwrap().check_permission(&Permission::FileSystemWrite)?;

                let uri = message.payload.get("uri")
                    .and_then(|u| u.as_str())
                    .ok_or("Missing uri")?;
                let content = message.payload.get("content")
                    .and_then(|c| c.as_array())
                    .ok_or("Missing content")?
                    .iter()
                    .filter_map(|v| v.as_u64().map(|n| n as u8))
                    .collect::<Vec<u8>>();

                let path = {
                    let validator = path_validator.lock().unwrap();
                    validator.validate_path(uri)?
                };

                tokio::task::spawn_blocking(move || {
                    match std::fs::write(&path, content) {
                        Ok(_) => Ok(serde_json::json!(null)),
                        Err(e) => Err(PathValidator::sanitize_error(&format!("Failed to write file: {}", e))),
                    }
                }).await.map_err(|e| e.to_string())??
            },

            "fsDelete" => {
                permissions.lock().unwrap().check_permission(&Permission::FileSystemDelete)?;

                let uri = message.payload.get("uri")
                    .and_then(|u| u.as_str())
                    .ok_or("Missing uri")?;
                let recursive = message.payload.get("options")
                    .and_then(|o| o.get("recursive"))
                    .and_then(|r| r.as_bool())
                    .unwrap_or(false);

                let path = {
                    let validator = path_validator.lock().unwrap();
                    validator.validate_path(uri)?
                };

                tokio::task::spawn_blocking(move || {
                    let result = if recursive {
                        std::fs::remove_dir_all(&path)
                    } else if path.is_dir() {
                        std::fs::remove_dir(&path)
                    } else {
                        std::fs::remove_file(&path)
                    };

                    match result {
                        Ok(_) => Ok(serde_json::json!(null)),
                        Err(e) => Err(PathValidator::sanitize_error(&format!("Failed to delete: {}", e))),
                    }
                }).await.map_err(|e| e.to_string())??
            },

            "fsReadDirectory" => {
                permissions.lock().unwrap().check_permission(&Permission::FileSystemRead)?;

                let uri = message.payload.get("uri")
                    .and_then(|u| u.as_str())
                    .ok_or("Missing uri")?;

                let path = {
                    let validator = path_validator.lock().unwrap();
                    validator.validate_path(uri)?
                };

                tokio::task::spawn_blocking(move || {
                    match std::fs::read_dir(&path) {
                        Ok(entries) => {
                            let mut result = Vec::new();
                            for entry in entries.flatten() {
                                if let (Ok(name), Ok(metadata)) = (entry.file_name().into_string(), entry.metadata()) {
                                    let file_type = if metadata.is_dir() { 2 } else if metadata.is_file() { 1 } else { 0 };
                                    result.push(serde_json::json!([name, file_type]));
                                }
                            }
                            Ok(serde_json::json!({ "entries": result }))
                        },
                        Err(e) => Err(PathValidator::sanitize_error(&format!("Failed to read directory: {}", e))),
                    }
                }).await.map_err(|e| e.to_string())??
            },

            "fsCreateDirectory" => {
                permissions.lock().unwrap().check_permission(&Permission::FileSystemWrite)?;

                let uri = message.payload.get("uri")
                    .and_then(|u| u.as_str())
                    .ok_or("Missing uri")?;

                let path = {
                    let validator = path_validator.lock().unwrap();
                    validator.validate_path(uri)?
                };

                tokio::task::spawn_blocking(move || {
                    match std::fs::create_dir_all(&path) {
                        Ok(_) => Ok(serde_json::json!(null)),
                        Err(e) => Err(PathValidator::sanitize_error(&format!("Failed to create directory: {}", e))),
                    }
                }).await.map_err(|e| e.to_string())??
            },

            "fsRename" => {
                permissions.lock().unwrap().check_permission(&Permission::FileSystemWrite)?;

                let source = message.payload.get("source")
                    .and_then(|s| s.as_str())
                    .ok_or("Missing source")?;
                let target = message.payload.get("target")
                    .and_then(|t| t.as_str())
                    .ok_or("Missing target")?;

                let (source_path, target_path) = {
                    let validator = path_validator.lock().unwrap();
                    let sp = validator.validate_path(source)?;
                    let tp = validator.validate_path(target)?;
                    (sp, tp)
                };

                tokio::task::spawn_blocking(move || {
                    match std::fs::rename(&source_path, &target_path) {
                        Ok(_) => Ok(serde_json::json!(null)),
                        Err(e) => Err(PathValidator::sanitize_error(&format!("Failed to rename: {}", e))),
                    }
                }).await.map_err(|e| e.to_string())??
            },

            "fsCopy" => {
                permissions.lock().unwrap().check_permission(&Permission::FileSystemWrite)?;

                let source = message.payload.get("source")
                    .and_then(|s| s.as_str())
                    .ok_or("Missing source")?;
                let target = message.payload.get("target")
                    .and_then(|t| t.as_str())
                    .ok_or("Missing target")?;

                let (source_path, target_path) = {
                    let validator = path_validator.lock().unwrap();
                    let sp = validator.validate_path(source)?;
                    let tp = validator.validate_path(target)?;
                    (sp, tp)
                };

                tokio::task::spawn_blocking(move || {
                    match std::fs::copy(&source_path, &target_path) {
                        Ok(_) => Ok(serde_json::json!(null)),
                        Err(e) => Err(PathValidator::sanitize_error(&format!("Failed to copy: {}", e))),
                    }
                }).await.map_err(|e| e.to_string())??
            },

            // ========== Secret Storage ==========
            "secretGet" => {
                permissions.lock().unwrap().check_permission(&Permission::SecretsRead)?;

                let key = message.payload.get("key")
                    .and_then(|k| k.as_str())
                    .ok_or("Missing key")?;

                let ext_id = extension_id.to_string();
                let key_owned = key.to_string();
                let secrets = Arc::clone(secret_storage);

                tokio::task::spawn_blocking(move || {
                    match secrets.get(&ext_id, &key_owned) {
                        Ok(value) => Ok(serde_json::json!(value)),
                        Err(e) => Err(e),
                    }
                }).await.map_err(|e| e.to_string())??
            },

            "secretStore" => {
                permissions.lock().unwrap().check_permission(&Permission::SecretsWrite)?;

                let key = message.payload.get("key")
                    .and_then(|k| k.as_str())
                    .ok_or("Missing key")?;
                let value = message.payload.get("value")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing value")?;

                let ext_id = extension_id.to_string();
                let key_owned = key.to_string();
                let value_owned = value.to_string();
                let secrets = Arc::clone(secret_storage);

                tokio::task::spawn_blocking(move || {
                    match secrets.set(&ext_id, &key_owned, &value_owned) {
                        Ok(_) => Ok(serde_json::json!(null)),
                        Err(e) => Err(e),
                    }
                }).await.map_err(|e| e.to_string())??
            },

            "secretDelete" => {
                permissions.lock().unwrap().check_permission(&Permission::SecretsWrite)?;

                let key = message.payload.get("key")
                    .and_then(|k| k.as_str())
                    .ok_or("Missing key")?;

                let ext_id = extension_id.to_string();
                let key_owned = key.to_string();
                let secrets = Arc::clone(secret_storage);

                tokio::task::spawn_blocking(move || {
                    match secrets.delete(&ext_id, &key_owned) {
                        Ok(_) => Ok(serde_json::json!(null)),
                        Err(e) => Err(e),
                    }
                }).await.map_err(|e| e.to_string())??
            },

            // ========== UI Operations ==========
            "window-show-message-with-actions" => {
                permissions.lock().unwrap().check_permission(&Permission::ShowDialogs)?;

                let message_text = message.payload.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Message");
                println!("[ExtensionHost] Window message: {}", message_text);
                serde_json::json!({ "action": null })
            },

            "window-show-input-box" => {
                permissions.lock().unwrap().check_permission(&Permission::ShowDialogs)?;
                serde_json::json!({ "value": null })
            },

            "window-show-quick-pick" => {
                permissions.lock().unwrap().check_permission(&Permission::ShowQuickPick)?;
                serde_json::json!({ "selected": null })
            },

            // ========== Clipboard Operations ==========
            "clipboardReadText" => {
                permissions.lock().unwrap().check_permission(&Permission::ClipboardRead)?;
                serde_json::json!("")
            },

            "clipboardWriteText" => {
                permissions.lock().unwrap().check_permission(&Permission::ClipboardWrite)?;
                serde_json::json!(null)
            },

            // Extensions directory
            "get-extensions-dir" => {
                // Return the extensions directory path
                let extensions_dir = std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .join("extensions");
                println!("[ExtensionHost] Extensions directory: {:?}", extensions_dir);
                serde_json::json!(extensions_dir.to_string_lossy().to_string())
            },

            // ========== Stubs for other operations ==========
            "openTextDocument" | "saveDocument" | "workspace-save-document" |
            "applyEdits" | "insertSnippet" | "workspace-find-files" |
            "workspace-get-folders" |
            "debugCustomRequest" | "debugGetProtocolBreakpoint" | "startDebugging" | "stopDebugging" |
            "fetchTasks" | "executeTask" | "getTerminalProcessId" | "execute-command-request" |
            "authGetSession" |
            "notebookCellClearOutput" | "notebookCellReplaceOutput" | "notebookCellAppendOutput" |
            "notebookCellReplaceOutputItems" | "notebookCellAppendOutputItems" => {
                println!("[ExtensionHost] Stub handler for: {}", message.r#type);
                serde_json::json!(null)
            },

            _ => {
                println!("[ExtensionHost] Unhandled request type: {}", message.r#type);
                serde_json::json!(null)
            }
        };

        // Send response
        let response = IPCMessage {
            id: message.id.clone(),
            r#type: format!("{}-response", message.r#type),
            payload: response_payload,
        };

        Self::send_response(response, stdin)
    }

    /// Send a response to the extension host
    fn send_response(
        message: IPCMessage,
        stdin: &Arc<Mutex<Option<std::process::ChildStdin>>>,
    ) -> Result<(), String> {
        let mut stdin_guard = stdin.lock().unwrap();
        if let Some(ref mut stdin_stream) = *stdin_guard {
            let json = serde_json::to_string(&message)
                .map_err(|e| format!("Failed to serialize response: {}", e))?;

            writeln!(stdin_stream, "{}", json)
                .map_err(|e| format!("Failed to write response: {}", e))?;

            stdin_stream.flush()
                .map_err(|e| format!("Failed to flush response: {}", e))?;

            Ok(())
        } else {
            Err("Extension host stdin not available".to_string())
        }
    }

    /// Send a message to the extension host
    pub fn send(&mut self, msg_type: &str, payload: Value) -> Result<(), String> {
        let mut message_id = self.message_id.lock().unwrap();
        *message_id += 1;
        let id = format!("msg_{}", *message_id);
        drop(message_id);

        let message = IPCMessage {
            id,
            r#type: msg_type.to_string(),
            payload,
        };

        self.send_message(message)
    }

    /// Send a request and wait for response (with timeout)
    pub fn request<F>(&mut self, msg_type: &str, payload: Value, callback: F) -> Result<(), String>
    where
        F: FnOnce(Result<Value, String>) + Send + 'static,
    {
        let mut message_id = self.message_id.lock().unwrap();
        *message_id += 1;
        let id = format!("req_{}", *message_id);
        drop(message_id);

        // Register callback with timestamp
        let mut requests = self.pending_requests.lock().unwrap();
        requests.insert(id.clone(), PendingRequest {
            callback: Box::new(callback),
            timestamp: Instant::now(),
        });
        drop(requests);

        let message = IPCMessage {
            id,
            r#type: msg_type.to_string(),
            payload,
        };

        self.send_message(message)
    }

    /// Send a message via stdin
    fn send_message(&mut self, message: IPCMessage) -> Result<(), String> {
        if let Some(ref mut child) = self.child {
            if let Some(ref mut stdin) = child.stdin {
                let json = serde_json::to_string(&message)
                    .map_err(|e| format!("Failed to serialize message: {}", e))?;

                writeln!(stdin, "{}", json)
                    .map_err(|e| format!("Failed to write to stdin: {}", e))?;

                stdin.flush()
                    .map_err(|e| format!("Failed to flush stdin: {}", e))?;

                Ok(())
            } else {
                Err("Extension host stdin not available".to_string())
            }
        } else {
            Err("Extension host not running".to_string())
        }
    }

    /// Check if the extension host is running
    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }

    /// Get list of registered commands
    pub fn get_registered_commands(&self) -> Vec<String> {
        self.registered_commands.read().unwrap().clone()
    }

    /// Stop the extension host
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.child.take() {
            println!("[ExtensionHost] Stopping...");
            child.kill()
                .map_err(|e| format!("Failed to kill extension host: {}", e))?;
            child.wait()
                .map_err(|e| format!("Failed to wait for extension host: {}", e))?;

            // Cleanup rate limiter
            self.rate_limiter.remove_limiter(&self.extension_id);

            println!("[ExtensionHost] Stopped");
        }
        Ok(())
    }
}

impl Drop for ExtensionHostManager {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
