use lsp_types::*;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use crate::extension_host::nng_ipc::NngExtensionIpc;

#[derive(Debug)]
pub struct LspClient {
    process: Arc<Mutex<Option<Child>>>,
    ipc: Arc<Mutex<Option<Arc<NngExtensionIpc>>>>,
    language_id: String,
    server_command: String,
    server_args: Vec<String>,
    request_id: Arc<Mutex<i32>>,
    ipc_url: String,
}

impl LspClient {
    pub fn new(language_id: String, server_command: String, server_args: Vec<String>) -> Self {
        let ipc_url = format!("ipc:///tmp/dscode-lsp-{}.ipc", language_id);
        Self {
            process: Arc::new(Mutex::new(None)),
            ipc: Arc::new(Mutex::new(None)),
            language_id,
            server_command,
            server_args,
            request_id: Arc::new(Mutex::new(0)),
            ipc_url,
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        let mut process_guard = self.process.lock().unwrap();

        // Check if already running
        if process_guard.is_some() {
            return Ok(());
        }

        // Spawn language server process with NNG IPC URL
        let child = Command::new(&self.server_command)
            .args(&self.server_args)
            .env("LSP_IPC_URL", &self.ipc_url)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start language server {}: {}", self.language_id, e))?;

        println!("[LSP] Started {} language server ({}) on {}", self.language_id, self.server_command, self.ipc_url);

        *process_guard = Some(child);
        drop(process_guard);

        // Wait for LSP server to bind
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Connect NNG IPC
        let ipc = NngExtensionIpc::new(&self.ipc_url)?;
        *self.ipc.lock().unwrap() = Some(Arc::new(ipc));

        println!("[LSP] Connected NNG IPC for {}", self.language_id);
        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut process_guard = self.process.lock().unwrap();

        if let Some(mut child) = process_guard.take() {
            child.kill()
                .map_err(|e| format!("Failed to kill language server: {}", e))?;
            println!("[LSP] Stopped {} language server", self.language_id);
        }

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.process.lock().unwrap().is_some()
    }

    fn next_request_id(&self) -> i32 {
        let mut id = self.request_id.lock().unwrap();
        *id += 1;
        *id
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

    pub async fn did_open(&self, uri: Url, language_id: String, text: String) -> Result<(), String> {
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id,
                version: 1,
                text,
            },
        };

        self.send_notification("textDocument/didOpen", params).await
    }

    pub async fn did_change(&self, uri: Url, version: i32, text: String) -> Result<(), String> {
        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri,
                version,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text,
            }],
        };

        self.send_notification("textDocument/didChange", params).await
    }

    pub async fn did_save(&self, uri: Url) -> Result<(), String> {
        let params = DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier { uri },
            text: None,
        };

        self.send_notification("textDocument/didSave", params).await
    }

    pub async fn hover(&self, uri: Url, line: u32, character: u32) -> Result<Option<Hover>, String> {
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position {
                    line,
                    character,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        self.send_request("textDocument/hover", params).await
    }

    async fn send_request<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: P,
    ) -> Result<R, String> {
        let id = self.next_request_id();

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        // Send via NNG IPC
        let ipc_guard = self.ipc.lock().unwrap();
        let ipc = ipc_guard.as_ref()
            .ok_or("LSP client not connected via NNG")?;

        let response = ipc.request("lsp:request", request).await?;

        // Parse LSP response
        if let Some(result) = response.get("result") {
            serde_json::from_value(result.clone())
                .map_err(|e| format!("Failed to parse LSP response: {}", e))
        } else if let Some(error) = response.get("error") {
            Err(format!("LSP error: {}", error))
        } else {
            Err("Invalid LSP response".to_string())
        }
    }

    async fn send_notification<P: serde::Serialize>(
        &self,
        method: &str,
        params: P,
    ) -> Result<(), String> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        // Send via NNG IPC (notifications don't wait for response)
        let ipc_guard = self.ipc.lock().unwrap();
        let ipc = ipc_guard.as_ref()
            .ok_or("LSP client not connected via NNG")?;

        ipc.send("lsp:notification", notification).await
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
