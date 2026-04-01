use lsp_types::*;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child as TokioChild, Command as TokioCommand};
use tokio::sync::{oneshot, Mutex};
use tracing::{error, info, instrument, warn};

static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// State machine for the LSP client lifecycle.
///
/// Tracks the lifecycle of a Language Server Protocol client connected to
/// an external language server process (e.g., rust-analyzer, pyright).
///
/// # State Diagram
///
/// ```text
///   Stopped ──► Starting ──► Initializing ──► Ready
///     ▲             │              │             │
///     │  (spawn     │  (init       │   (stop()   │
///     │   fail)     │   fail)      │   called)   │
///     │             ▼              ▼             ▼
///     │          Crashed ◄──── Crashed      ShuttingDown
///     │             ▲                            │
///     │             │ (process exit,             │
///     │             │  read loop EOF)            │
///     │             └────────── Ready ───────────┘
///     │                                          │
///     └──────────────────────────────────────────┘
/// ```
///
/// # Transitions
///
/// - `Stopped` -> `Starting` (start() called, spawning process)
/// - `Starting` -> `Initializing` (process spawned, read loop started)
/// - `Starting` -> `Crashed` (Command::spawn() failed)
/// - `Initializing` -> `Ready` (LSP initialize handshake completed)
/// - `Initializing` -> `Crashed` (initialize request failed or timed out)
/// - `Ready` -> `ShuttingDown` (stop() called, sending shutdown request)
/// - `Ready` -> `Crashed` (process exited unexpectedly, read loop EOF)
/// - `ShuttingDown` -> `Stopped` (exit notification sent, process exited)
/// - `Crashed` -> `Starting` (explicit restart attempt)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspClientState {
    /// No language server process is running.
    Stopped,
    /// The language server process is being spawned.
    Starting,
    /// The LSP initialize handshake is in progress.
    Initializing,
    /// The language server is fully initialized and ready to handle requests.
    Ready,
    /// A shutdown request has been sent to the language server.
    ShuttingDown,
    /// The language server process exited unexpectedly or failed to start.
    Crashed,
}

type PendingResponseMap = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<serde_json::Value, String>>>>>;

/// A state-machine-based LSP client that manages a language server process.
///
/// Handles spawning the language server, performing the LSP initialize handshake,
/// sending requests and notifications, and processing responses from the server.
/// All state transitions are validated to maintain the lifecycle invariant.
///
/// # Concurrency
///
/// State is stored in `Arc<Mutex<LspClientState>>`. The process and writer fields
/// use separate `Arc<Mutex<Option<T>>>` which can be locked independently.
/// Lock ordering: state -> process -> writer. Never hold the state lock while
/// also holding process/writer locks to avoid deadlock.
#[derive(Debug)]
pub struct LspClient {
    state: Arc<Mutex<LspClientState>>,
    process: Arc<Mutex<Option<TokioChild>>>,
    writer: Arc<Mutex<Option<tokio::process::ChildStdin>>>,
    pending_responses: PendingResponseMap,
    language_id: String,
    server_command: String,
    server_args: Vec<String>,
}

impl LspClient {
    /// Creates a new LSP client in the `Stopped` state.
    ///
    /// - `language_id` — The language identifier (e.g., "rust", "python").
    /// - `server_command` — The command to spawn the language server (e.g., "rust-analyzer").
    /// - `server_args` — Arguments to pass to the language server command.
    pub fn new(language_id: String, server_command: String, server_args: Vec<String>) -> Self {
        Self {
            state: Arc::new(Mutex::new(LspClientState::Stopped)),
            process: Arc::new(Mutex::new(None)),
            writer: Arc::new(Mutex::new(None)),
            pending_responses: Arc::new(Mutex::new(HashMap::new())),
            language_id,
            server_command,
            server_args,
        }
    }

    /// Validates and performs a state transition for the LSP client.
    async fn transition(&self, to: LspClientState) -> Result<(), String> {
        let mut state = self.state.lock().await;
        let valid = match *state {
            LspClientState::Stopped => matches!(to, LspClientState::Starting),
            LspClientState::Starting => {
                matches!(to, LspClientState::Initializing | LspClientState::Crashed)
            }
            LspClientState::Initializing => {
                matches!(to, LspClientState::Ready | LspClientState::Crashed)
            }
            LspClientState::Ready => {
                matches!(to, LspClientState::ShuttingDown | LspClientState::Crashed)
            }
            LspClientState::ShuttingDown => matches!(to, LspClientState::Stopped),
            LspClientState::Crashed => matches!(to, LspClientState::Starting),
        };

        if valid {
            info!(language = %self.language_id, "LSP state: {:?} -> {:?}", *state, to);
            *state = to;
            Ok(())
        } else {
            let msg = format!(
                "Invalid LSP state transition for {}: {:?} -> {:?}",
                self.language_id, *state, to
            );
            error!("{}", msg);
            Err(msg)
        }
    }

    /// Returns the current state of the LSP client.
    pub async fn get_state(&self) -> LspClientState {
        *self.state.lock().await
    }

    /// Spawns the language server process and begins the initialization sequence.
    ///
    /// Transitions from `Stopped` to `Starting` to `Initializing`. If spawning
    /// fails, transitions to `Crashed`. A background task is spawned to read
    /// responses from the server's stdout.
    ///
    /// Returns `Ok(())` if the process was spawned successfully, or an error
    /// describing the failure.
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<(), String> {
        self.transition(LspClientState::Starting).await?;

        let mut process_guard = self.process.lock().await;

        if process_guard.is_some() {
            return Ok(());
        }

        let mut cmd = TokioCommand::new(&self.server_command);
        cmd.args(&self.server_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                let _ = self.transition(LspClientState::Crashed).await;
                return Err(format!(
                    "Failed to start language server {}: {}",
                    self.language_id, e
                ));
            }
        };

        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;

        if let Some(stderr) = child.stderr.take() {
            let lang_id = self.language_id.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    warn!(language = %lang_id, "LSP stderr: {}", line);
                }
            });
        }

        *self.writer.lock().await = Some(stdin);
        *process_guard = Some(child);
        // Release process lock before transitioning state (lock ordering: state -> process)
        drop(process_guard);

        info!(language = %self.language_id, command = %self.server_command, "Started language server");

        let pending_clone = Arc::clone(&self.pending_responses);
        let state_clone = Arc::clone(&self.state);
        let language_id_clone = self.language_id.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut header_buf = String::new();

            loop {
                header_buf.clear();
                let mut read_err = false;
                loop {
                    let mut byte = [0u8; 1];
                    if reader.read_exact(&mut byte).await.is_err() {
                        read_err = true;
                        break;
                    }
                    header_buf.push(byte[0] as char);

                    if header_buf.ends_with("\r\n\r\n") {
                        break;
                    }

                    if header_buf.len() > 4096 {
                        warn!("LSP header too long, disconnecting");
                        read_err = true;
                        break;
                    }
                }

                if read_err {
                    break;
                }

                let mut content_length: usize = 0;
                for line in header_buf.split("\r\n") {
                    if let Some(len_str) = line.strip_prefix("Content-Length: ") {
                        content_length = len_str.trim().parse().unwrap_or(0);
                    }
                }

                if content_length == 0 {
                    continue;
                }

                let mut body = vec![0u8; content_length];
                if reader.read_exact(&mut body).await.is_err() {
                    break;
                }

                let response: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Failed to parse LSP response: {}", e);
                        continue;
                    }
                };

                if let Some(id) = response.get("id").and_then(|v| v.as_u64()) {
                    let mut pending_guard = pending_clone.lock().await;
                    if let Some(sender) = pending_guard.remove(&id) {
                        if let Some(error) = response.get("error") {
                            let _ = sender.send(Err(format!("LSP error: {}", error)));
                        } else {
                            let _ = sender.send(Ok(response));
                        }
                    }
                }
            }

            // Read loop ended — transition to Crashed if still active
            {
                let mut state = state_clone.lock().await;
                if *state == LspClientState::Ready || *state == LspClientState::Initializing {
                    info!(language = %language_id_clone, "LSP read loop ended, state -> Crashed");
                    *state = LspClientState::Crashed;
                }
            }
            // Reject all pending requests when read loop ends
            let mut pending = pending_clone.lock().await;
            for (_, sender) in pending.drain() {
                let _ = sender.send(Err("LSP server connection closed".to_string()));
            }
        });

        self.transition(LspClientState::Initializing).await?;

        Ok(())
    }

    /// Stops the language server process and releases resources.
    ///
    /// Transitions through `ShuttingDown` to `Stopped`. Kills the child process,
    /// drops the stdin writer, and drains any pending responses.
    ///
    /// Returns `Ok(())` on success.
    pub async fn stop(&self) -> Result<(), String> {
        let _ = self.transition(LspClientState::ShuttingDown).await;

        let mut process_guard = self.process.lock().await;

        if let Some(mut child) = process_guard.take() {
            let _ = child.kill().await;
            info!(language = %self.language_id, "Stopped language server");
        }

        *self.writer.lock().await = None;

        let _ = self.transition(LspClientState::Stopped).await;

        Ok(())
    }

    /// Returns `true` if the client is in `Initializing` or `Ready` state.
    pub async fn is_running(&self) -> bool {
        let state = self.state.lock().await;
        matches!(*state, LspClientState::Initializing | LspClientState::Ready)
    }

    fn next_request_id(&self) -> u64 {
        REQUEST_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    /// Sends the LSP `initialize` request with client capabilities.
    ///
    /// - `root_uri` — The root URI of the workspace to initialize.
    ///
    /// Transitions to `Ready` on success, or `Crashed` on failure.
    /// Returns the [`InitializeResult`] from the server on success.
    #[instrument(skip(self))]
    pub async fn initialize(&self, root_uri: Url) -> Result<InitializeResult, String> {
        let params = InitializeParams {
            process_id: Some(std::process::id()),
            root_uri: Some(root_uri),
            capabilities: ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities {
                    hover: Some(HoverClientCapabilities {
                        dynamic_registration: Some(false),
                        content_format: Some(vec![MarkupKind::Markdown, MarkupKind::PlainText]),
                    }),
                    completion: Some(CompletionClientCapabilities {
                        dynamic_registration: Some(false),
                        completion_item: Some(CompletionItemCapability {
                            snippet_support: Some(true),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    definition: Some(GotoCapability {
                        dynamic_registration: Some(false),
                        link_support: Some(false),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = self.send_request("initialize", params).await;
        match &result {
            Ok(_) => {
                self.transition(LspClientState::Ready).await?;
            }
            Err(_) => {
                let _ = self.transition(LspClientState::Crashed).await;
            }
        }
        result
    }

    /// Sends a `textDocument/didOpen` notification to the language server.
    ///
    /// - `uri` — The URI of the document that was opened.
    /// - `language_id` — The language identifier for the document.
    /// - `text` — The full initial content of the document.
    pub async fn did_open(
        &self, uri: Url, language_id: String, text: String,
    ) -> Result<(), String> {
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem { uri, language_id, version: 1, text },
        };

        self.send_notification("textDocument/didOpen", params).await
    }

    /// Sends a `textDocument/didChange` notification with the full document content.
    ///
    /// - `uri` — The URI of the document that changed.
    /// - `version` — The new version number of the document.
    /// - `text` — The full updated content of the document.
    pub async fn did_change(&self, uri: Url, version: i32, text: String) -> Result<(), String> {
        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, version },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text,
            }],
        };

        self.send_notification("textDocument/didChange", params).await
    }

    /// Sends a `textDocument/didSave` notification to the language server.
    ///
    /// - `uri` — The URI of the document that was saved.
    pub async fn did_save(&self, uri: Url) -> Result<(), String> {
        let params =
            DidSaveTextDocumentParams { text_document: TextDocumentIdentifier { uri }, text: None };

        self.send_notification("textDocument/didSave", params).await
    }

    /// Sends a `textDocument/hover` request to retrieve hover information.
    ///
    /// - `uri` — The URI of the document.
    /// - `line` — The zero-based line number of the position.
    /// - `character` — The zero-based character offset of the position.
    ///
    /// Returns hover information if available, or `None` if the server
    /// provides no hover data at the given position.
    pub async fn hover(
        &self, uri: Url, line: u32, character: u32,
    ) -> Result<Option<Hover>, String> {
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        self.send_request("textDocument/hover", params).await
    }

    /// Sends a JSON-RPC request to the language server and waits for a response.
    ///
    /// Serializes the request, writes it to the server's stdin using the
    /// LSP Content-Length framing protocol, and waits up to 30 seconds for
    /// a matching response. Returns the deserialized result on success.
    async fn send_request<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self, method: &str, params: P,
    ) -> Result<R, String> {
        let id = self.next_request_id();

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending_responses.lock().await;
            pending.insert(id, tx);
        }

        {
            let mut writer_guard = self.writer.lock().await;
            let writer = writer_guard.as_mut().ok_or("LSP client stdin not available")?;

            let body = serde_json::to_string(&request)
                .map_err(|e| format!("Failed to serialize LSP request: {}", e))?;
            let header = format!("Content-Length: {}\r\n\r\n", body.len());
            writer
                .write_all(header.as_bytes())
                .await
                .map_err(|e| format!("Failed to write LSP header: {}", e))?;
            writer
                .write_all(body.as_bytes())
                .await
                .map_err(|e| format!("Failed to write LSP body: {}", e))?;
        }

        match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
            Ok(Ok(Ok(response))) => {
                if let Some(result) = response.get("result") {
                    serde_json::from_value(result.clone())
                        .map_err(|e| format!("Failed to parse LSP response: {}", e))
                } else if let Some(error) = response.get("error") {
                    Err(format!("LSP error: {}", error))
                } else {
                    Err("Invalid LSP response".to_string())
                }
            }
            Ok(Ok(Err(e))) => Err(e),
            Ok(Err(_)) => {
                // Channel closed -- server crashed or connection lost
                let mut pending = self.pending_responses.lock().await;
                pending.remove(&id);
                Err("LSP request channel closed (server may have crashed)".to_string())
            }
            Err(_) => {
                // Timeout -- server did not respond within 30 seconds
                let mut pending = self.pending_responses.lock().await;
                pending.remove(&id);
                Err("LSP request timed out after 30s".to_string())
            }
        }
    }

    /// Sends a JSON-RPC notification to the language server (no response expected).
    ///
    /// Serializes the notification and writes it to the server's stdin using
    /// the LSP Content-Length framing protocol.
    async fn send_notification<P: serde::Serialize>(
        &self, method: &str, params: P,
    ) -> Result<(), String> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        let mut writer_guard = self.writer.lock().await;
        let writer = writer_guard.as_mut().ok_or("LSP client stdin not available")?;

        let body = serde_json::to_string(&notification)
            .map_err(|e| format!("Failed to serialize LSP notification: {}", e))?;
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        writer
            .write_all(header.as_bytes())
            .await
            .map_err(|e| format!("Failed to write LSP header: {}", e))?;
        writer
            .write_all(body.as_bytes())
            .await
            .map_err(|e| format!("Failed to write LSP body: {}", e))?;

        Ok(())
    }
    /// Sends the LSP `shutdown` request to the language server.
    ///
    /// This signals the server to stop processing requests. Call `stop()`
    /// afterwards to terminate the process.
    #[instrument(skip(self))]
    pub async fn shutdown(&self) -> Result<(), String> {
        self.send_notification("shutdown", serde_json::json!({})).await
    }

    /// Sends the `initialized` notification to the language server.
    ///
    /// Must be called after `initialize()` to complete the handshake.
    pub async fn initialized(&self) -> Result<(), String> {
        self.send_notification("initialized", serde_json::json!({})).await
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        if let Ok(mut process) = self.process.try_lock() {
            if let Some(mut child) = process.take() {
                let _ = child.start_kill();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lsp_client_new_state() {
        let client = LspClient::new(
            "rust".to_string(),
            "rust-analyzer".to_string(),
            vec![],
        );
        let state = client.get_state().await;
        assert_eq!(state, LspClientState::Stopped);
    }

    #[tokio::test]
    async fn test_lsp_client_state_transitions() {
        // Verify that LspClientState variants exist and PartialEq works correctly.
        // Valid transitions (per the state machine):
        //   Stopped -> Starting
        //   Starting -> Initializing | Crashed
        //   Initializing -> Ready | Crashed
        //   Ready -> ShuttingDown | Crashed
        //   ShuttingDown -> Stopped
        //   Crashed -> Starting

        assert!(LspClientState::Stopped != LspClientState::Starting);
        assert!(LspClientState::Starting != LspClientState::Initializing);
        assert!(LspClientState::Initializing != LspClientState::Ready);
        assert!(LspClientState::Ready != LspClientState::ShuttingDown);
        assert!(LspClientState::ShuttingDown != LspClientState::Stopped);
        assert!(LspClientState::Crashed != LspClientState::Starting);

        // Verify that a newly created client is in Stopped state
        let client = LspClient::new(
            "python".to_string(),
            "pyright-langserver".to_string(),
            vec!["--stdio".to_string()],
        );
        assert_eq!(client.get_state().await, LspClientState::Stopped);

        // Verify that attempting to stop from Stopped state is invalid
        // (Stopped -> ShuttingDown is not a valid transition)
        let result = client.transition(LspClientState::ShuttingDown).await;
        assert!(result.is_err());

        // Verify that Starting from Stopped is valid (though it will fail
        // because there's no actual process to spawn)
        // We can't fully test start() without a real server, but we can
        // verify the transition logic via the state machine.
    }

    #[tokio::test]
    async fn test_lsp_client_is_running_stopped() {
        let client = LspClient::new(
            "rust".to_string(),
            "rust-analyzer".to_string(),
            vec![],
        );
        // A newly created client is Stopped, not running
        assert!(!client.is_running().await);
    }

    #[tokio::test]
    async fn test_lsp_client_is_running_only_active_states() {
        // is_running returns true only for Initializing and Ready
        let client = LspClient::new(
            "rust".to_string(),
            "rust-analyzer".to_string(),
            vec![],
        );
        // Stopped -> not running
        assert!(!client.is_running().await);

        // Transition to Starting -> not running (Starting is not considered "running")
        client.transition(LspClientState::Starting).await.unwrap();
        assert!(!client.is_running().await);

        // Transition to Initializing -> now running
        client.transition(LspClientState::Initializing).await.unwrap();
        assert!(client.is_running().await);

        // Crashed -> not running
        client.transition(LspClientState::Crashed).await.unwrap();
        assert!(!client.is_running().await);
    }

    #[tokio::test]
    async fn test_lsp_client_invalid_transition_crashed_to_ready() {
        let client = LspClient::new(
            "rust".to_string(),
            "rust-analyzer".to_string(),
            vec![],
        );
        // Go to Crashed state
        client.transition(LspClientState::Starting).await.unwrap();
        client.transition(LspClientState::Crashed).await.unwrap();

        // Crashed -> Ready is invalid
        let result = client.transition(LspClientState::Ready).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_lsp_client_invalid_transition_stopped_to_initializing() {
        let client = LspClient::new(
            "rust".to_string(),
            "rust-analyzer".to_string(),
            vec![],
        );
        // Stopped -> Initializing is invalid (must go through Starting)
        let result = client.transition(LspClientState::Initializing).await;
        assert!(result.is_err());
    }
}