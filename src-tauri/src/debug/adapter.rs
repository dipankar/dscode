use super::types::*;
use serde_json::Value;
use std::collections::HashMap;
use std::process::{Child, Stdio};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child as TokioChild, Command as TokioCommand};
use tokio::sync::{oneshot, Mutex};

pub struct DebugAdapter {
    session: DebugSession,
    process: Arc<Mutex<Option<TokioChild>>>,
    writer: Arc<Mutex<Option<tokio::process::ChildStdin>>>,
    pending_responses: Arc<Mutex<HashMap<i32, oneshot::Sender<Result<Value, String>>>>>,
    adapter_command: String,
    adapter_args: Vec<String>,
    sequence: Arc<Mutex<i32>>,
}

impl DebugAdapter {
    pub fn new(session: DebugSession, adapter_command: String, adapter_args: Vec<String>) -> Self {
        Self {
            session,
            process: Arc::new(Mutex::new(None)),
            writer: Arc::new(Mutex::new(None)),
            pending_responses: Arc::new(Mutex::new(HashMap::new())),
            adapter_command,
            adapter_args,
            sequence: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        let mut process_guard = self.process.lock().await;

        if process_guard.is_some() {
            return Ok(());
        }

        let mut cmd = TokioCommand::new(&self.adapter_command);
        cmd.args(&self.adapter_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            format!("Failed to start debug adapter {}: {}", self.session.adapter_type, e)
        })?;

        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;

        if let Some(stderr) = child.stderr.take() {
            let session_id = self.session.id.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    eprintln!("[Debug:{}] {}", session_id, line);
                }
            });
        }

        println!(
            "[Debug] Started {} debug adapter ({})",
            self.session.adapter_type, self.adapter_command
        );

        *self.writer.lock().await = Some(stdin);
        *process_guard = Some(child);

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
                        eprintln!("[Debug] Header too long, disconnecting");
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

                let response: Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[Debug] Failed to parse response: {}", e);
                        continue;
                    }
                };

                if response.get("type").and_then(|t| t.as_str()) == Some("response") {
                    if let Some(seq) = response.get("request_seq").and_then(|v| v.as_i64()) {
                        let mut pending_guard = pending.lock().await;
                        if let Some(sender) = pending_guard.remove(&(seq as i32)) {
                            if response.get("success").and_then(|s| s.as_bool()) == Some(true) {
                                let _ = sender.send(Ok(response));
                            } else {
                                let message = response
                                    .get("message")
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("Unknown error");
                                let _ = sender.send(Err(message.to_string()));
                            }
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
            println!("[Debug] Stopped debug adapter for session {}", self.session.id);
        }

        *self.writer.lock().await = None;

        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        let guard = self.process.lock().await;
        guard.is_some()
    }

    async fn next_sequence(&self) -> i32 {
        let mut seq = self.sequence.lock().await;
        *seq += 1;
        *seq
    }

    pub async fn send_request(
        &self, command: &str, arguments: Option<Value>,
    ) -> Result<Value, String> {
        let seq = self.next_sequence().await;

        let mut request = serde_json::json!({
            "seq": seq,
            "type": "request",
            "command": command,
        });

        if let Some(args) = arguments {
            request["arguments"] = args;
        }

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending_responses.lock().await;
            pending.insert(seq, tx);
        }

        {
            let mut writer_guard = self.writer.lock().await;
            let writer = writer_guard.as_mut().ok_or("Debug adapter stdin not available")?;

            let body = serde_json::to_string(&request)
                .map_err(|e| format!("Failed to serialize DAP request: {}", e))?;
            let header = format!("Content-Length: {}\r\n\r\n", body.len());
            writer
                .write_all(header.as_bytes())
                .await
                .map_err(|e| format!("Failed to write DAP header: {}", e))?;
            writer
                .write_all(body.as_bytes())
                .await
                .map_err(|e| format!("Failed to write DAP body: {}", e))?;
        }

        match rx.await {
            Ok(Ok(response)) => Ok(response.get("body").cloned().unwrap_or(Value::Null)),
            Ok(Err(e)) => Err(format!("DAP error: {}", e)),
            Err(_) => {
                let mut pending = self.pending_responses.lock().await;
                pending.remove(&seq);
                Err("DAP request channel closed".to_string())
            }
        }
    }

    pub async fn send_event(&self, event: &str, body: Option<Value>) -> Result<(), String> {
        let seq = self.next_sequence().await;

        let mut event_msg = serde_json::json!({
            "seq": seq,
            "type": "event",
            "event": event,
        });

        if let Some(b) = body {
            event_msg["body"] = b;
        }

        {
            let mut writer_guard = self.writer.lock().await;
            let writer = writer_guard.as_mut().ok_or("Debug adapter stdin not available")?;

            let body = serde_json::to_string(&event_msg)
                .map_err(|e| format!("Failed to serialize DAP event: {}", e))?;
            let header = format!("Content-Length: {}\r\n\r\n", body.len());
            writer
                .write_all(header.as_bytes())
                .await
                .map_err(|e| format!("Failed to write DAP header: {}", e))?;
            writer
                .write_all(body.as_bytes())
                .await
                .map_err(|e| format!("Failed to write DAP body: {}", e))?;
        }

        Ok(())
    }

    pub async fn initialize(&self) -> Result<Value, String> {
        self.send_request(
            "initialize",
            Some(serde_json::json!({
                "clientID": "dscode",
                "clientName": "DSCode",
                "adapterID": self.session.adapter_type,
                "pathFormat": "path",
                "linesStartAt1": true,
                "columnsStartAt1": true,
                "supportsVariableType": true,
                "supportsVariablePaging": true,
                "supportsRunInTerminalRequest": true,
            })),
        )
        .await
    }

    pub async fn launch(&self, configuration: Value) -> Result<(), String> {
        self.send_request("launch", Some(configuration)).await?;
        Ok(())
    }

    pub async fn attach(&self, configuration: Value) -> Result<(), String> {
        self.send_request("attach", Some(configuration)).await?;
        Ok(())
    }

    pub async fn set_breakpoints(
        &self, source: Value, breakpoints: Vec<SourceBreakpoint>,
    ) -> Result<Vec<Breakpoint>, String> {
        let response = self
            .send_request(
                "setBreakpoints",
                Some(serde_json::json!({
                    "source": source,
                    "breakpoints": breakpoints,
                })),
            )
            .await?;

        let bps = response
            .get("breakpoints")
            .and_then(|b| b.as_array())
            .ok_or("Invalid setBreakpoints response")?;

        serde_json::from_value(Value::Array(bps.clone()))
            .map_err(|e| format!("Failed to parse breakpoints: {}", e))
    }

    pub async fn continue_execution(&self, thread_id: i32) -> Result<(), String> {
        self.send_request(
            "continue",
            Some(serde_json::json!({
                "threadId": thread_id,
            })),
        )
        .await?;
        Ok(())
    }

    pub async fn pause(&self, thread_id: i32) -> Result<(), String> {
        self.send_request(
            "pause",
            Some(serde_json::json!({
                "threadId": thread_id,
            })),
        )
        .await?;
        Ok(())
    }

    pub async fn next(&self, thread_id: i32) -> Result<(), String> {
        self.send_request(
            "next",
            Some(serde_json::json!({
                "threadId": thread_id,
            })),
        )
        .await?;
        Ok(())
    }

    pub async fn step_in(&self, thread_id: i32) -> Result<(), String> {
        self.send_request(
            "stepIn",
            Some(serde_json::json!({
                "threadId": thread_id,
            })),
        )
        .await?;
        Ok(())
    }

    pub async fn step_out(&self, thread_id: i32) -> Result<(), String> {
        self.send_request(
            "stepOut",
            Some(serde_json::json!({
                "threadId": thread_id,
            })),
        )
        .await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), String> {
        self.send_request("disconnect", None).await?;
        Ok(())
    }
}

impl Drop for DebugAdapter {
    fn drop(&mut self) {}
}
