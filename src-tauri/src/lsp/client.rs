use lsp_types::*;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child as TokioChild, Command as TokioCommand};
use tokio::sync::{oneshot, Mutex};

static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub struct LspClient {
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
            process: Arc::new(Mutex::new(None)),
            writer: Arc::new(Mutex::new(None)),
            pending_responses: Arc::new(Mutex::new(HashMap::new())),
            language_id,
            server_command,
            server_args,
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        let mut process_guard = self.process.lock().await;

        if process_guard.is_some() {
            return Ok(());
        }

        let mut cmd = TokioCommand::new(&self.server_command);
        cmd.args(&self.server_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start language server {}: {}", self.language_id, e))?;

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

        println!("[LSP] Started {} language server ({})", self.language_id, self.server_command);

        let pending = Arc::clone(&self.pending_responses);
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut header_buf = String::new();

            loop {
                header_buf.clear();
                loop {
                    let mut byte = [0u8; 1];
                    if reader.read_exact(&mut byte).await.is_err() {
                        return;
                    }
                    header_buf.push(byte[0] as char);

                    if header_buf.ends_with("\r\n\r\n") {
                        break;
                    }

                    if header_buf.len() > 4096 {
                        eprintln!("[LSP] Header too long, disconnecting");
                        return;
                    }
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
                    return;
                }

                let response: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[LSP] Failed to parse response: {}", e);
                        continue;
                    }
                };

                if let Some(id) = response.get("id").and_then(|v| v.as_u64()) {
                    let mut pending_guard = pending.lock().await;
                    if let Some(sender) = pending_guard.remove(&id) {
                        if let Some(error) = response.get("error") {
                            let _ = sender.send(Err(format!("LSP error: {}", error)));
                        } else {
                            let _ = sender.send(Ok(response));
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        let mut process_guard = self.process.lock().await;

        if let Some(mut child) = process_guard.take() {
            let _ = child.kill().await;
            println!("[LSP] Stopped {} language server", self.language_id);
        }

        *self.writer.lock().await = None;

        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        let mut process_guard = self.process.lock().await;
        if let Some(child) = process_guard.as_mut() {
            match child.try_wait() {
                Ok(None) => true,
                Ok(Some(_)) => false,
                Err(_) => false,
            }
        } else {
            false
        }
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

        self.send_request("initialize", params).await
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

        match rx.await {
            Ok(Ok(response)) => {
                if let Some(result) = response.get("result") {
                    serde_json::from_value(result.clone())
                        .map_err(|e| format!("Failed to parse LSP response: {}", e))
                } else if let Some(error) = response.get("error") {
                    Err(format!("LSP error: {}", error))
                } else {
                    Err("Invalid LSP response".to_string())
                }
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                let mut pending = self.pending_responses.lock().await;
                pending.remove(&id);
                Err("LSP request channel closed".to_string())
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
