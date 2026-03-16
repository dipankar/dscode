/**
 * Extension Host Manager
 *
 * Manages the Node.js extension host process and IPC communication via stdio.
 */

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPCMessage {
    pub id: String,
    pub r#type: String,
    pub payload: Value,
}

type ResponseCallback = Box<dyn FnOnce(Result<Value, String>) + Send>;

pub struct ExtensionHostManager {
    child: Option<Child>,
    pending_requests: Arc<Mutex<HashMap<String, ResponseCallback>>>,
    message_id: Arc<Mutex<u64>>,
    stdin: Arc<Mutex<Option<std::process::ChildStdin>>>,
}

impl ExtensionHostManager {
    pub fn new() -> Self {
        Self {
            child: None,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            message_id: Arc::new(Mutex::new(0)),
            stdin: Arc::new(Mutex::new(None)),
        }
    }

    /// Start the extension host process
    pub fn start(&mut self, extension_host_path: &str) -> Result<(), String> {
        println!("[ExtensionHost] Starting extension host from: {}", extension_host_path);

        let mut child = Command::new("node")
            .arg(extension_host_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn extension host: {}", e))?;

        // Get stdin for sending messages
        let stdin = child.stdin.take()
            .ok_or("Failed to capture stdin")?;

        // Save stdin for sending responses
        *self.stdin.lock().unwrap() = Some(stdin);

        // Get stdout for reading messages
        let stdout = child.stdout.take()
            .ok_or("Failed to capture stdout")?;

        // Get stderr for logging
        let stderr = child.stderr.take()
            .ok_or("Failed to capture stderr")?;

        // Spawn thread to read stderr (logs from extension host)
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

        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if let Err(e) = Self::handle_message(&line, &pending_requests, &stdin_handle) {
                        eprintln!("[ExtensionHost] Error handling message: {}", e);
                    }
                }
            }
        });

        self.child = Some(child);
        println!("[ExtensionHost] Started successfully");

        Ok(())
    }

    /// Handle incoming message from extension host
    fn handle_message(
        line: &str,
        pending_requests: &Arc<Mutex<HashMap<String, ResponseCallback>>>,
        stdin: &Arc<Mutex<Option<std::process::ChildStdin>>>,
    ) -> Result<(), String> {
        let message: IPCMessage = serde_json::from_str(line)
            .map_err(|e| format!("Failed to parse message: {}", e))?;

        println!("[ExtensionHost] Received message: {} ({})", message.r#type, message.id);

        // Check if this is a response to a pending request
        let mut requests = pending_requests.lock().unwrap();
        if let Some(callback) = requests.remove(&message.id) {
            drop(requests); // Release lock
            if message.r#type.ends_with("-error") {
                let error = message.payload.get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();
                callback(Err(error));
            } else {
                callback(Ok(message.payload));
            }
        } else {
            drop(requests); // Release lock
            // This is a request from the extension host
            Self::handle_request(&message, stdin)?;
        }

        Ok(())
    }

    /// Handle a request from the extension host
    fn handle_request(
        message: &IPCMessage,
        stdin: &Arc<Mutex<Option<std::process::ChildStdin>>>,
    ) -> Result<(), String> {
        // Handle specific request types
        let response_payload = match message.r#type.as_str() {
            "get-extensions-dir" => {
                // Get extensions directory
                let extensions_dir = std::env::current_dir()
                    .map_err(|e| e.to_string())?
                    .parent()
                    .ok_or("Failed to get parent directory")?
                    .join("extensions");

                // Create directory if it doesn't exist
                std::fs::create_dir_all(&extensions_dir)
                    .map_err(|e| format!("Failed to create extensions directory: {}", e))?;

                let path_str = extensions_dir.to_str()
                    .ok_or("Failed to convert path to string")?
                    .to_string();

                serde_json::json!(path_str)
            },
            "workspace-get-folders" => {
                // For now, return empty array
                // TODO: Track actual workspace folders
                serde_json::json!([])
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

            println!("[ExtensionHost] Sent response: {}", message.r#type);
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

    /// Send a request and wait for response
    pub fn request<F>(&mut self, msg_type: &str, payload: Value, callback: F) -> Result<(), String>
    where
        F: FnOnce(Result<Value, String>) + Send + 'static,
    {
        let mut message_id = self.message_id.lock().unwrap();
        *message_id += 1;
        let id = format!("req_{}", *message_id);
        drop(message_id);

        // Register callback
        let mut requests = self.pending_requests.lock().unwrap();
        requests.insert(id.clone(), Box::new(callback));
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

    /// Stop the extension host
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.child.take() {
            println!("[ExtensionHost] Stopping...");
            child.kill()
                .map_err(|e| format!("Failed to kill extension host: {}", e))?;
            child.wait()
                .map_err(|e| format!("Failed to wait for extension host: {}", e))?;
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
