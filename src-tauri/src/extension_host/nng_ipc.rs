/**
 * NNG-based IPC for Extension Host
 *
 * Uses NNG (nanomsg-next-generation) for robust async communication
 * between Tauri and the Node.js extension host.
 *
 * Supports bidirectional communication:
 * - NngExtensionIpc: For Tauri->ExtHost requests (REQ socket)
 * - NngIncomingIpc: For ExtHost->Tauri requests (REP socket)
 */

use nng::{Protocol, Socket};
use nng::options::Options;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPCMessage {
    pub id: String,
    pub r#type: String,
    pub payload: Value,
}

#[derive(Debug)]
pub struct NngExtensionIpc {
    socket: Arc<Mutex<Socket>>,
    message_id: Arc<Mutex<u64>>,
}

impl NngExtensionIpc {
    /// Create a new NNG IPC manager and connect to the extension host
    pub fn new(ipc_url: &str) -> Result<Self, String> {
        // Create a REQ (request) socket - we send requests, extension host replies
        let socket = Socket::new(Protocol::Req0)
            .map_err(|e| format!("Failed to create NNG socket: {}", e))?;

        // Set timeout for receives
        socket.set_opt::<nng::options::RecvTimeout>(Some(REQUEST_TIMEOUT))
            .map_err(|e| format!("Failed to set recv timeout: {}", e))?;

        // Connect to IPC endpoint (extension host will bind/listen)
        socket.dial(ipc_url)
            .map_err(|e| format!("Failed to connect to NNG socket at {}: {}", ipc_url, e))?;

        println!("[NNG IPC] Connected to {}", ipc_url);

        Ok(Self {
            socket: Arc::new(Mutex::new(socket)),
            message_id: Arc::new(Mutex::new(0)),
        })
    }

    /// Send a request to the extension host and wait for response
    pub async fn request(&self, msg_type: &str, payload: Value) -> Result<Value, String> {
        // Generate unique message ID
        let id = {
            let mut message_id = self.message_id.lock().await;
            *message_id += 1;
            format!("req_{}", *message_id)
        };

        let message = IPCMessage {
            id: id.clone(),
            r#type: msg_type.to_string(),
            payload,
        };

        // Serialize message
        let json = serde_json::to_string(&message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        let msg_bytes = json.as_bytes();

        // Send request
        let socket = self.socket.lock().await;

        socket.send(msg_bytes)
            .map_err(|(_, e)| format!("Failed to send message: {}", e))?;

        // Wait for response
        let response_msg = socket.recv()
            .map_err(|e| format!("Failed to receive response: {}", e))?;

        drop(socket); // Release lock

        // Parse response
        let response_str = std::str::from_utf8(&response_msg)
            .map_err(|e| format!("Invalid UTF-8 in response: {}", e))?;

        let response: IPCMessage = serde_json::from_str(response_str)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Check for error response
        if response.r#type.ends_with("-error") {
            let error_msg = response.payload.get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("Unknown error");
            return Err(error_msg.to_string());
        }

        Ok(response.payload)
    }

    /// Send a one-way message (fire and forget)
    pub async fn send(&self, msg_type: &str, payload: Value) -> Result<(), String> {
        let id = {
            let mut message_id = self.message_id.lock().await;
            *message_id += 1;
            format!("msg_{}", *message_id)
        };

        let message = IPCMessage {
            id,
            r#type: msg_type.to_string(),
            payload,
        };

        let json = serde_json::to_string(&message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        let socket = self.socket.lock().await;

        socket.send(json.as_bytes())
            .map_err(|(_, e)| format!("Failed to send message: {}", e))?;

        // For one-way messages, we still need to receive the ack since we're using REQ/REP
        let _ = socket.recv()
            .map_err(|e| format!("Failed to receive ack: {}", e))?;

        Ok(())
    }
}

/// Handler function type for incoming requests
pub type IncomingRequestHandler = Arc<dyn Fn(String, Value) -> Result<Value, String> + Send + Sync>;

/// NNG IPC for handling incoming requests from Extension Host
pub struct NngIncomingIpc {
    socket: Arc<Mutex<Socket>>,
    handler: Option<IncomingRequestHandler>,
    running: Arc<Mutex<bool>>,
}

impl NngIncomingIpc {
    /// Create a new NNG incoming IPC manager and bind to an endpoint
    pub fn new(ipc_url: &str) -> Result<Self, String> {
        // Create a REP (reply) socket - extension host will send requests, we reply
        let socket = Socket::new(Protocol::Rep0)
            .map_err(|e| format!("Failed to create NNG REP socket: {}", e))?;

        // Bind to IPC endpoint (we listen, extension host connects)
        socket.listen(ipc_url)
            .map_err(|e| format!("Failed to bind NNG socket at {}: {}", ipc_url, e))?;

        println!("[NNG Incoming] Listening on {}", ipc_url);

        Ok(Self {
            socket: Arc::new(Mutex::new(socket)),
            handler: None,
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// Set the handler for incoming requests
    pub fn set_handler<F>(&mut self, handler: F)
    where
        F: Fn(String, Value) -> Result<Value, String> + Send + Sync + 'static,
    {
        self.handler = Some(Arc::new(handler));
    }

    /// Start listening for incoming requests
    pub async fn start(&self) -> Result<(), String> {
        if self.handler.is_none() {
            return Err("No handler set for incoming requests".to_string());
        }

        let mut running = self.running.lock().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        let socket = Arc::clone(&self.socket);
        let handler = self.handler.clone().unwrap();
        let running_flag = Arc::clone(&self.running);

        tokio::spawn(async move {
            loop {
                // Check if still running
                {
                    let running = running_flag.lock().await;
                    if !*running {
                        break;
                    }
                }

                // Receive request
                let request_msg = {
                    let sock = socket.lock().await;
                    match sock.recv() {
                        Ok(msg) => msg,
                        Err(e) => {
                            eprintln!("[NNG Incoming] Failed to receive: {}", e);
                            continue;
                        }
                    }
                };

                // Parse request
                let request_str = match std::str::from_utf8(&request_msg) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[NNG Incoming] Invalid UTF-8: {}", e);
                        continue;
                    }
                };

                let request: IPCMessage = match serde_json::from_str(request_str) {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("[NNG Incoming] Failed to parse request: {}", e);
                        continue;
                    }
                };

                // Handle request
                let response_payload = match handler(request.r#type.clone(), request.payload.clone()) {
                    Ok(payload) => payload,
                    Err(e) => {
                        // Send error response
                        let error_response = IPCMessage {
                            id: request.id.clone(),
                            r#type: format!("{}-error", request.r#type),
                            payload: serde_json::json!({ "error": e }),
                        };

                        let json = match serde_json::to_string(&error_response) {
                            Ok(j) => j,
                            Err(e) => {
                                eprintln!("[NNG Incoming] Failed to serialize error: {}", e);
                                continue;
                            }
                        };

                        let sock = socket.lock().await;
                        if let Err(e) = sock.send(json.as_bytes()) {
                            eprintln!("[NNG Incoming] Failed to send error response: {:?}", e);
                        }
                        continue;
                    }
                };

                // Send success response
                let response = IPCMessage {
                    id: request.id,
                    r#type: format!("{}-response", request.r#type),
                    payload: response_payload,
                };

                let json = match serde_json::to_string(&response) {
                    Ok(j) => j,
                    Err(e) => {
                        eprintln!("[NNG Incoming] Failed to serialize response: {}", e);
                        continue;
                    }
                };

                let sock = socket.lock().await;
                if let Err(e) = sock.send(json.as_bytes()) {
                    eprintln!("[NNG Incoming] Failed to send response: {:?}", e);
                }
            }

            println!("[NNG Incoming] Stopped listening");
        });

        Ok(())
    }

    /// Stop listening
    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        *running = false;
    }
}
