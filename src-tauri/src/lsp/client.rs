use lsp_types::*;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child as TokioChild, Command as TokioCommand};
use tokio::sync::{oneshot, Mutex};

static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// STATE MACHINE: LspClient
///
/// Tracks the lifecycle of a Language Server Protocol client connected to
/// an external language server process (e.g., rust-analyzer, pyright).
///
/// State Diagram:
///
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
///
/// Transitions:
///   Stopped      -> Starting      (start() called, spawning process)
///   Starting     -> Initializing  (process spawned, read loop started)
///   Starting     -> Crashed       (Command::spawn() failed)
///   Initializing -> Ready         (LSP initialize handshake completed)
///   Initializing -> Crashed       (initialize request failed or timed out)
///   Ready        -> ShuttingDown  (stop() called, sending shutdown request)
///   Ready        -> Crashed       (process exited unexpectedly, read loop EOF)
///   ShuttingDown -> Stopped       (exit notification sent, process exited)
///   Crashed      -> Starting      (explicit restart attempt)
///
/// Concurrency Invariant:
///   State is stored in Arc<Mutex<LspClientState>>. All state reads and
///   transitions acquire the mutex. The process and writer fields use
///   separate Arc<Mutex<Option<T>>> which can be locked independently.
///   IMPORTANT: Never hold the state lock while also holding process/writer
///   locks to avoid deadlock. Lock ordering: state -> process -> writer.
///
/// Interruption Table:
/// ┌──────────────┬────────────────────────────────────────────────────────────┐
/// │ State        │ What happens + impact on pending requests                 │
/// ├──────────────┼────────────────────────────────────────────────────────────┤
/// │ Stopped      │ Safe. No process, no resources, no pending requests.      │
/// ├──────────────┼────────────────────────────────────────────────────────────┤
/// │ Starting     │ If spawn fails: -> Crashed. No pending requests yet.      │
/// │              │ If Tauri crashes: child process orphaned, OS reaps.       │
/// ├──────────────┼────────────────────────────────────────────────────────────┤
/// │ Initializing │ Initialize request is pending. If process exits:          │
/// │              │ -> Crashed. Initialize caller gets timeout error (NEW).   │
/// │              │ Previously: caller would hang FOREVER (no timeout).       │
/// │              │ No language features available until Ready.               │
/// ├──────────────┼────────────────────────────────────────────────────────────┤
/// │ Ready        │ If process exits unexpectedly: -> Crashed.               │
/// │              │ ALL pending requests (hover, completion, etc.) were       │
/// │              │ hanging FOREVER (BUG). After fix: rejected after 30s.    │
/// │              │ User sees: language features stop responding for 30s,     │
/// │              │ then errors. No auto-restart (TODO).                      │
/// │              │ Impact: hover shows nothing, completions empty,           │
/// │              │ diagnostics stale, go-to-definition fails.               │
/// ├──────────────┼────────────────────────────────────────────────────────────┤
/// │ ShuttingDown │ Shutdown request sent. If process ignores it: exit        │
/// │              │ notification sent anyway, process may need force kill.    │
/// │              │ Pending requests drained and rejected.                    │
/// ├──────────────┼────────────────────────────────────────────────────────────┤
/// │ Crashed      │ All pending requests rejected (via timeout). Pool should  │
/// │              │ detect this and remove client from active pool.           │
/// │              │ TODO: Implement auto-restart with exponential backoff.    │
/// │              │ Until then: language features dead for this language.     │
/// └──────────────┴────────────────────────────────────────────────────────────┘
///
/// Cross-Layer Impact:
///   LSP is an internal subsystem — frontend doesn't directly know about
///   individual LSP server states. When LSP crashes:
///   - Diagnostics stop updating (frontend shows stale diagnostics)
///   - Hover/completion requests return empty/error
///   - User experience: editor feels "broken" for that language
///   - No notification to user about LSP failure (TODO)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspClientState {
    Stopped,
    Starting,
    Initializing,
    Ready,
    ShuttingDown,
    Crashed,
}

#[derive(Debug)]
pub struct LspClient {
    state: Arc<Mutex<LspClientState>>,
    process: Arc<Mutex<Option<TokioChild>>>,
    writer: Arc<Mutex<Option<tokio::process::ChildStdin>>>,
    pending_responses: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<serde_json::Value, String>>>>>,
    language_id: String,
    server_command: String,
    server_args: Vec<String>,
}

impl LspClient {
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
            println!("[LSP:{}] State: {:?} -> {:?}", self.language_id, *state, to);
            *state = to;
            Ok(())
        } else {
            let msg = format!(
                "[LSP:{}] Invalid state transition: {:?} -> {:?}",
                self.language_id, *state, to
            );
            eprintln!("{}", msg);
            Err(msg)
        }
    }

    /// Returns the current state.
    pub async fn get_state(&self) -> LspClientState {
        *self.state.lock().await
    }

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
                    eprintln!("[LSP:{}] {}", lang_id, line);
                }
            });
        }

        *self.writer.lock().await = Some(stdin);
        *process_guard = Some(child);
        // Release process lock before transitioning state (lock ordering: state -> process)
        drop(process_guard);

        println!("[LSP] Started {} language server ({})", self.language_id, self.server_command);

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
                        eprintln!("[LSP] Header too long, disconnecting");
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
                        eprintln!("[LSP] Failed to parse response: {}", e);
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
                    println!(
                        "[LSP:{}] Read loop ended, state -> Crashed",
                        language_id_clone
                    );
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

    pub async fn stop(&self) -> Result<(), String> {
        let _ = self.transition(LspClientState::ShuttingDown).await;

        let mut process_guard = self.process.lock().await;

        if let Some(mut child) = process_guard.take() {
            let _ = child.kill().await;
            println!("[LSP] Stopped {} language server", self.language_id);
        }

        *self.writer.lock().await = None;

        let _ = self.transition(LspClientState::Stopped).await;

        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        let state = self.state.lock().await;
        matches!(*state, LspClientState::Initializing | LspClientState::Ready)
    }

    fn next_request_id(&self) -> u64 {
        REQUEST_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }

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

    pub async fn did_open(
        &self, uri: Url, language_id: String, text: String,
    ) -> Result<(), String> {
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem { uri, language_id, version: 1, text },
        };

        self.send_notification("textDocument/didOpen", params).await
    }

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

    pub async fn did_save(&self, uri: Url) -> Result<(), String> {
        let params =
            DidSaveTextDocumentParams { text_document: TextDocumentIdentifier { uri }, text: None };

        self.send_notification("textDocument/didSave", params).await
    }

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
    pub async fn shutdown(&self) -> Result<(), String> {
        self.send_notification("shutdown", serde_json::json!({})).await
    }

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
