use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::{oneshot, Mutex, Notify, RwLock};
use tracing::{debug, error, info};

const MAX_MESSAGE_SIZE: usize = 50 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct IPCMessage {
    pub(crate) id: String,
    pub(crate) r#type: String,
    pub(crate) payload: Value,
}

pub type IncomingRequestHandler = Arc<
    dyn Fn(String, Value) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>
        + Send
        + Sync,
>;

async fn read_message<R: AsyncReadExt + Unpin>(reader: &mut R) -> Result<IPCMessage, String> {
    let mut len_buf = [0u8; 4];
    reader
        .read_exact(&mut len_buf)
        .await
        .map_err(|e| format!("Failed to read message length: {}", e))?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len == 0 {
        return Err("Zero-length message".to_string());
    }
    if len > MAX_MESSAGE_SIZE {
        return Err(format!("Message too large: {} bytes", len));
    }
    let mut body = vec![0u8; len];
    reader
        .read_exact(&mut body)
        .await
        .map_err(|e| format!("Failed to read message body: {}", e))?;
    let msg: IPCMessage =
        serde_json::from_slice(&body).map_err(|e| format!("Failed to parse IPC message: {}", e))?;
    Ok(msg)
}

async fn write_message<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    msg: &IPCMessage,
) -> Result<(), String> {
    let body =
        serde_json::to_vec(msg).map_err(|e| format!("Failed to serialize IPC message: {}", e))?;
    let len = body.len() as u32;
    writer
        .write_all(&len.to_be_bytes())
        .await
        .map_err(|e| format!("Failed to write message length: {}", e))?;
    writer
        .write_all(&body)
        .await
        .map_err(|e| format!("Failed to write message body: {}", e))?;
    writer
        .flush()
        .await
        .map_err(|e| format!("Failed to flush message: {}", e))?;
    Ok(())
}

type PendingRequestMap = Arc<Mutex<HashMap<String, oneshot::Sender<Result<Value, String>>>>>;

pub struct ExtensionIpc {
    write: Arc<Mutex<tokio::net::unix::OwnedWriteHalf>>,
    pending_requests: PendingRequestMap,
    message_id: Arc<Mutex<u64>>,
    alive: Arc<std::sync::atomic::AtomicBool>,
}

impl ExtensionIpc {
    pub fn new(stream: UnixStream) -> Self {
        let (read_half, write_half) = stream.into_split();
        let write = Arc::new(Mutex::new(write_half));
        let pending_requests: PendingRequestMap = Arc::new(Mutex::new(HashMap::new()));
        let message_id = Arc::new(Mutex::new(0u64));
        let alive = Arc::new(std::sync::atomic::AtomicBool::new(true));

        let pending_clone = Arc::clone(&pending_requests);
        let alive_clone = Arc::clone(&alive);

        tokio::spawn(async move {
            let mut reader = BufReader::new(read_half);
            loop {
                match read_message(&mut reader).await {
                    Ok(msg) => {
                        let mut pending = pending_clone.lock().await;
                        if let Some(sender) = pending.remove(&msg.id) {
                            if msg.r#type.ends_with("-error") {
                                let error_msg = msg
                                    .payload
                                    .get("error")
                                    .and_then(|e| e.as_str())
                                    .unwrap_or("Unknown error")
                                    .to_string();
                                let _ = sender.send(Err(error_msg));
                            } else {
                                let _ = sender.send(Ok(msg.payload));
                            }
                        }
                    }
                    Err(e) => {
                        if e.contains("Failed to read message length") {
                            break;
                        }
                        error!(error = %e, "Read error");
                        break;
                    }
                }
            }
            // When the read loop breaks (socket EOF or read error):
            // 1. alive flag set to false (Relaxed ordering is sufficient —
            //    eventual consistency is fine for a "dead connection" signal)
            // 2. All pending requests are cleared by dropping their Senders,
            //    which causes each Receiver to get RecvError
            // 3. Any future request() calls will fail the alive check
            // 4. The IpcManager still holds a reference to this ExtensionIpc —
            //    it must be explicitly removed or replaced on reconnection
            alive_clone.store(false, std::sync::atomic::Ordering::Relaxed);
            let mut pending = pending_clone.lock().await;
            pending.clear();
        });

        Self {
            write,
            pending_requests,
            message_id,
            alive,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn request(&self, msg_type: &str, payload: Value) -> Result<Value, String> {
        // Check connection liveness before sending. This is a best-effort check —
        // the connection could die between this check and the actual write.
        // The timeout below protects against that race.
        if !self.alive.load(std::sync::atomic::Ordering::Relaxed) {
            return Err("IPC connection is not alive".to_string());
        }

        let id = {
            let mut message_id = self.message_id.lock().await;
            *message_id += 1;
            format!("req_{}", *message_id)
        };

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id.clone(), tx);
        }

        let message = IPCMessage {
            id: id.clone(),
            r#type: msg_type.to_string(),
            payload,
        };

        {
            let mut writer = self.write.lock().await;
            write_message(&mut *writer, &message).await?;
        }

        // Request timeout: 30 seconds.
        // Three possible outcomes for a pending request:
        //   1. Response received: resolved normally via oneshot channel
        //   2. Connection closed: oneshot Sender dropped, Receiver gets RecvError
        //   3. Timeout: tokio::time::timeout fires, request cleaned up
        //
        // Without this timeout, if the extension host becomes unresponsive
        // (infinite loop, deadlock) but the connection stays alive, the caller
        // blocks forever. Connection close (outcome 2) only helps when the
        // process actually crashes or the socket breaks.
        match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => {
                // Sender dropped — connection closed while request was in flight.
                // The read loop (spawned task) detected socket EOF and cleared
                // pending requests by dropping all Senders.
                let mut pending = self.pending_requests.lock().await;
                pending.remove(&id);
                Err("IPC connection closed while awaiting response".to_string())
            }
            Err(_) => {
                // Timeout — extension host did not respond within 30 seconds.
                // This can happen if the host is in an infinite loop, deadlocked,
                // or simply overwhelmed. The request is removed from pending to
                // prevent memory leaks. If a response arrives later (after timeout),
                // it will be silently dropped (no matching pending entry).
                let mut pending = self.pending_requests.lock().await;
                pending.remove(&id);
                Err(format!("IPC request '{}' timed out after 30s", msg_type))
            }
        }
    }

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

        {
            let mut writer = self.write.lock().await;
            write_message(&mut *writer, &message).await?;
        }

        Ok(())
    }
}

pub struct IncomingIpc {
    listener: Arc<tokio::net::UnixListener>,
    handler: Option<IncomingRequestHandler>,
    running: Arc<Mutex<bool>>,
    shutdown: Arc<Notify>,
}

impl IncomingIpc {
    pub fn new(socket_path: &str) -> Result<Self, String> {
        let path = socket_path
            .strip_prefix("ipc://")
            .ok_or_else(|| format!("Invalid IPC URL: {}", socket_path))?;

        if std::path::Path::new(path).exists() {
            let _ = std::fs::remove_file(path);
        }

        let listener = tokio::net::UnixListener::bind(path)
            .map_err(|e| format!("Failed to bind IPC socket at {}: {}", path, e))?;

        info!(path = path, "Listening on incoming IPC socket");

        Ok(Self {
            listener: Arc::new(listener),
            handler: None,
            running: Arc::new(Mutex::new(false)),
            shutdown: Arc::new(Notify::new()),
        })
    }

    pub fn set_handler(&mut self, handler: IncomingRequestHandler) {
        self.handler = Some(handler);
    }

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

        let handler = self
            .handler
            .clone()
            .ok_or_else(|| "No handler set for incoming requests".to_string())?;
        let running_flag = Arc::clone(&self.running);
        let listener = Arc::clone(&self.listener);
        let shutdown = Arc::clone(&self.shutdown);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((stream, _)) => {
                                let handler = handler.clone();
                                let running_flag = Arc::clone(&running_flag);
                                tokio::spawn(async move {
                                    if let Err(e) =
                                        handle_incoming_connection(stream, handler, running_flag).await
                                    {
                                        error!(error = %e, "Connection handler error");
                                    }
                                });
                            }
                            Err(e) => {
                                error!(error = %e, "Accept error");
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            }
                        }
                    }
                    _ = shutdown.notified() => {
                        break;
                    }
                }
            }

            let mut running = running_flag.lock().await;
            *running = false;
            info!("Stopped listening on incoming IPC socket");
        });

        Ok(())
    }

    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        *running = false;
        drop(running);
        self.shutdown.notify_waiters();
    }
}

async fn handle_incoming_connection(
    stream: tokio::net::UnixStream,
    handler: IncomingRequestHandler,
    running_flag: Arc<Mutex<bool>>,
) -> Result<(), String> {
    let (read_half, write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    let write = Arc::new(Mutex::new(write_half));

    loop {
        {
            let running = running_flag.lock().await;
            if !*running {
                break;
            }
        }

        let msg = match read_message(&mut reader).await {
            Ok(msg) => msg,
            Err(e) => {
                if e.contains("Failed to read message length") {
                    break;
                }
                error!(error = %e, "Read error, closing connection");
                break;
            }
        };

        let response_payload = match handler(msg.r#type.clone(), msg.payload.clone()).await {
            Ok(payload) => payload,
            Err(e) => {
                let error_response = IPCMessage {
                    id: msg.id.clone(),
                    r#type: format!("{}-error", msg.r#type),
                    payload: serde_json::json!({ "error": e }),
                };

                let mut writer = write.lock().await;
                let _ = write_message(&mut *writer, &error_response).await;
                continue;
            }
        };

        let response = IPCMessage {
            id: msg.id,
            r#type: format!("{}-response", msg.r#type),
            payload: response_payload,
        };

        let mut writer = write.lock().await;
        let _ = write_message(&mut *writer, &response).await;
    }

    Ok(())
}

pub struct IpcManager {
    outgoing: Arc<RwLock<HashMap<String, Arc<ExtensionIpc>>>>,
    incoming: Arc<RwLock<HashMap<String, Arc<IncomingIpc>>>>,
}

impl Default for IpcManager {
    fn default() -> Self {
        Self::new()
    }
}

impl IpcManager {
    pub fn new() -> Self {
        Self {
            outgoing: Arc::new(RwLock::new(HashMap::new())),
            incoming: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn connect_outgoing(&self, id: &str, socket_path: &str) -> Result<(), String> {
        debug!(id = id, socket_path = socket_path, "Connecting outgoing");

        let path = socket_path
            .strip_prefix("ipc://")
            .ok_or_else(|| format!("Invalid IPC URL: {}", socket_path))?;

        let max_attempts = 10;
        for attempt in 1..=max_attempts {
            match UnixStream::connect(path).await {
                Ok(stream) => {
                    let ipc = Arc::new(ExtensionIpc::new(stream));
                    let mut conns = self.outgoing.write().await;
                    conns.insert(id.to_string(), ipc);
                    info!(id = id, "Outgoing connection established");
                    return Ok(());
                }
                Err(_) if attempt < max_attempts => {
                    let delay = tokio::time::Duration::from_millis(attempt as u64 * 200);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                Err(e) => {
                    return Err(format!(
                        "Failed to connect after {} attempts: {}",
                        max_attempts, e
                    ));
                }
            }
        }

        Err("Failed to connect".to_string())
    }

    pub async fn setup_incoming(
        &self,
        id: &str,
        socket_path: &str,
        handler: IncomingRequestHandler,
    ) -> Result<(), String> {
        debug!(id = id, socket_path = socket_path, "Setting up incoming");

        let mut ipc = IncomingIpc::new(socket_path)?;
        ipc.set_handler(handler);

        let ipc_arc = Arc::new(ipc);
        ipc_arc.start().await?;

        let mut conns = self.incoming.write().await;
        conns.insert(id.to_string(), ipc_arc);

        info!(id = id, "Incoming connection ready");
        Ok(())
    }

    pub async fn disconnect(&self, id: &str) {
        let mut outgoing = self.outgoing.write().await;
        let mut incoming = self.incoming.write().await;

        outgoing.remove(id);
        if let Some(incoming) = incoming.remove(id) {
            incoming.stop().await;
        }
    }

    pub async fn is_connected(&self, id: &str) -> bool {
        let conns = self.outgoing.read().await;
        conns.get(id).is_some_and(|ipc| ipc.is_alive())
    }

    pub async fn reconnect_outgoing(&self, id: &str, socket_path: &str) -> Result<(), String> {
        {
            let mut conns = self.outgoing.write().await;
            if let Some(old) = conns.remove(id) {
                drop(old);
            }
        }
        self.connect_outgoing(id, socket_path).await
    }

    pub async fn request(&self, id: &str, msg_type: &str, payload: Value) -> Result<Value, String> {
        let ipc = self
            .get_outgoing(id)
            .await
            .ok_or_else(|| format!("Extension host '{}' not connected", id))?;
        ipc.request(msg_type, payload).await
    }

    pub async fn get_outgoing(&self, id: &str) -> Option<Arc<ExtensionIpc>> {
        let conns = self.outgoing.read().await;
        conns.get(id).cloned()
    }

    pub async fn connect(&self, id: &str, socket_path: &str) -> Result<(), String> {
        self.connect_outgoing(id, socket_path).await
    }

    pub async fn get(&self, id: &str) -> Option<Arc<ExtensionIpc>> {
        self.get_outgoing(id).await
    }
}
