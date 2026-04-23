use crate::types::DebugSession;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child as TokioChild, Command as TokioCommand};
use tokio::sync::{oneshot, Mutex};
use tracing::{error, info, instrument};

/// State machine for the DAP debug adapter lifecycle.
///
/// Tracks the lifecycle of a Debug Adapter Protocol (DAP) adapter process.
///
/// # State Diagram
///
/// ```text
///   Stopped ──► Starting ──► Initializing ──► Configured ──► Running
///     ▲             │              │               │            │
///     │             │              │               │            │
///     │             ▼              ▼               ▼            ▼
///     │          Crashed ◄──── Crashed ◄──── Crashed ◄──── Crashed
///     │                                                        │
///     │                                                        │
///     │                                            ShuttingDown│
///     │                                                │       │
///     └────────────────────────────────────────────────┘       │
///                                                     ▲        │
///                                                     └────────┘
/// ```
///
/// # Transitions
///
/// - `Stopped` -> `Starting` (start() called)
/// - `Starting` -> `Initializing` (process spawned, DAP initialize sent)
/// - `Starting` -> `Crashed` (spawn failed)
/// - `Initializing` -> `Configured` (initialize response received)
/// - `Initializing` -> `Crashed` (initialize failed or timed out)
/// - `Configured` -> `Running` (launch/attach response received)
/// - `Configured` -> `Crashed` (launch/attach failed)
/// - `Running` -> `ShuttingDown` (disconnect/terminate requested)
/// - `Running` -> `Crashed` (adapter process exited unexpectedly)
/// - `ShuttingDown` -> `Stopped` (adapter exited cleanly)
/// - `Crashed` -> `Starting` (restart attempt)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugAdapterState {
    /// No debug adapter process is running.
    Stopped,
    /// The debug adapter process is being spawned.
    Starting,
    /// The DAP initialize request has been sent and a response is pending.
    Initializing,
    /// The adapter has been initialized and configured, ready for launch/attach.
    Configured,
    /// The debug session is actively running with a live adapter.
    Running,
    /// A disconnect request has been sent and the adapter is shutting down.
    ShuttingDown,
    /// The adapter process exited unexpectedly or failed to start.
    Crashed,
}

type PendingResponseMap = Arc<Mutex<HashMap<i32, oneshot::Sender<Result<Value, String>>>>>;

/// A state-machine-based DAP client that manages a debug adapter process.
///
/// Handles spawning the debug adapter, performing the DAP initialize and
/// launch/attach handshakes, sending requests, and processing responses.
/// All state transitions are validated to maintain the lifecycle invariant.
///
/// # Concurrency
///
/// State is held in `Arc<Mutex<>>`, separate from process/writer.
/// Lock ordering: state -> process -> writer -> pending_responses.
pub struct DebugAdapter {
    state: Arc<Mutex<DebugAdapterState>>,
    session: DebugSession,
    process: Arc<Mutex<Option<TokioChild>>>,
    writer: Arc<Mutex<Option<tokio::process::ChildStdin>>>,
    pending_responses: PendingResponseMap,
    adapter_command: String,
    adapter_args: Vec<String>,
    sequence: Arc<Mutex<i32>>,
}

impl std::fmt::Debug for DebugAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugAdapter")
            .field("session", &self.session)
            .field("adapter_command", &self.adapter_command)
            .field("adapter_args", &self.adapter_args)
            .finish_non_exhaustive()
    }
}

impl DebugAdapter {
    /// Creates a new debug adapter in the `Stopped` state.
    ///
    /// - `session` — The debug session this adapter belongs to.
    /// - `adapter_command` — The command to spawn the debug adapter (e.g., "/usr/bin/gdb").
    /// - `adapter_args` — Arguments to pass to the adapter command.
    pub fn new(session: DebugSession, adapter_command: String, adapter_args: Vec<String>) -> Self {
        Self {
            state: Arc::new(Mutex::new(DebugAdapterState::Stopped)),
            session,
            process: Arc::new(Mutex::new(None)),
            writer: Arc::new(Mutex::new(None)),
            pending_responses: Arc::new(Mutex::new(HashMap::new())),
            adapter_command,
            adapter_args,
            sequence: Arc::new(Mutex::new(0)),
        }
    }

    /// Validates and performs a state transition for the debug adapter.
    pub(crate) async fn transition(&self, to: DebugAdapterState) -> Result<(), String> {
        let mut state = self.state.lock().await;
        let valid = match *state {
            DebugAdapterState::Stopped => matches!(to, DebugAdapterState::Starting),
            DebugAdapterState::Starting => {
                matches!(
                    to,
                    DebugAdapterState::Initializing | DebugAdapterState::Crashed
                )
            }
            DebugAdapterState::Initializing => {
                matches!(
                    to,
                    DebugAdapterState::Configured | DebugAdapterState::Crashed
                )
            }
            DebugAdapterState::Configured => {
                matches!(to, DebugAdapterState::Running | DebugAdapterState::Crashed)
            }
            DebugAdapterState::Running => {
                matches!(
                    to,
                    DebugAdapterState::ShuttingDown | DebugAdapterState::Crashed
                )
            }
            DebugAdapterState::ShuttingDown => matches!(to, DebugAdapterState::Stopped),
            DebugAdapterState::Crashed => matches!(to, DebugAdapterState::Starting),
        };

        if valid {
            info!(
                id = %self.session.id,
                from = ?*state,
                to = ?to,
                "State transition"
            );
            *state = to;
            Ok(())
        } else {
            let msg = format!("Invalid state transition: {:?} -> {:?}", *state, to);
            error!(
                id = %self.session.id,
                from = ?*state,
                to = ?to,
                "{}",
                msg
            );
            Err(msg)
        }
    }

    /// Returns the current state of the debug adapter.
    pub async fn get_state(&self) -> DebugAdapterState {
        *self.state.lock().await
    }

    /// Spawns the debug adapter process and begins the initialization sequence.
    ///
    /// Transitions from `Stopped` to `Starting` to `Initializing`. If spawning
    /// fails, transitions to `Crashed`. A background task is spawned to read
    /// responses from the adapter's stdout.
    ///
    /// Returns `Ok(())` if the process was spawned successfully, or an error
    /// describing the failure.
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<(), String> {
        self.transition(DebugAdapterState::Starting).await?;

        let mut process_guard = self.process.lock().await;

        if process_guard.is_some() {
            return Ok(());
        }

        let mut cmd = TokioCommand::new(&self.adapter_command);
        cmd.args(&self.adapter_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                let _ = self.transition(DebugAdapterState::Crashed).await;
                return Err(format!(
                    "Failed to start debug adapter {}: {}",
                    self.session.adapter_type, e
                ));
            }
        };

        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;

        if let Some(stderr) = child.stderr.take() {
            let session_id = self.session.id.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    error!(id = %session_id, "{}", line);
                }
            });
        }

        info!(
            adapter_type = %self.session.adapter_type,
            command = %self.adapter_command,
            "Started debug adapter"
        );

        *self.writer.lock().await = Some(stdin);
        *process_guard = Some(child);
        // Release process lock before transitioning state (lock ordering: state -> process)
        drop(process_guard);

        let pending_clone = Arc::clone(&self.pending_responses);
        let state_clone = Arc::clone(&self.state);
        let session_id_clone = self.session.id.clone();
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
                        error!("Header too long, disconnecting");
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

                let response: Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Failed to parse response: {}", e);
                        continue;
                    }
                };

                if response.get("type").and_then(|t| t.as_str()) == Some("response") {
                    if let Some(seq) = response.get("request_seq").and_then(|v| v.as_i64()) {
                        let mut pending_guard = pending_clone.lock().await;
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

            // Read loop ended -- transition to Crashed if still active
            {
                let mut state = state_clone.lock().await;
                if matches!(
                    *state,
                    DebugAdapterState::Initializing
                        | DebugAdapterState::Configured
                        | DebugAdapterState::Running
                ) {
                    info!(
                        id = %session_id_clone,
                        "Read loop ended, state -> Crashed"
                    );
                    *state = DebugAdapterState::Crashed;
                }
            }
            // Reject all pending requests when read loop ends
            let mut pending = pending_clone.lock().await;
            for (_, sender) in pending.drain() {
                let _ = sender.send(Err("Debug adapter connection closed".to_string()));
            }
        });

        self.transition(DebugAdapterState::Initializing).await?;

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        let _ = self.transition(DebugAdapterState::ShuttingDown).await;

        let mut process_guard = self.process.lock().await;

        if let Some(mut child) = process_guard.take() {
            let _ = child.kill().await;
            info!(id = %self.session.id, "Stopped debug adapter");
        }

        *self.writer.lock().await = None;

        let _ = self.transition(DebugAdapterState::Stopped).await;

        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        let state = self.state.lock().await;
        matches!(
            *state,
            DebugAdapterState::Initializing
                | DebugAdapterState::Configured
                | DebugAdapterState::Running
        )
    }

    async fn next_sequence(&self) -> i32 {
        let mut seq = self.sequence.lock().await;
        *seq += 1;
        *seq
    }

    pub(crate) async fn send_request(
        &self,
        command: &str,
        arguments: Option<Value>,
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
            let writer = writer_guard
                .as_mut()
                .ok_or("Debug adapter stdin not available")?;

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

        match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
            Ok(Ok(Ok(response))) => Ok(response.get("body").cloned().unwrap_or(Value::Null)),
            Ok(Ok(Err(e))) => Err(format!("DAP error: {}", e)),
            Ok(Err(_)) => {
                // Channel closed -- adapter crashed or connection lost
                let mut pending = self.pending_responses.lock().await;
                pending.remove(&seq);
                Err("DAP request channel closed (adapter may have crashed)".to_string())
            }
            Err(_) => {
                // Timeout -- adapter did not respond within 30 seconds
                let mut pending = self.pending_responses.lock().await;
                pending.remove(&seq);
                Err("DAP request timed out after 30s".to_string())
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) async fn send_event(&self, event: &str, body: Option<Value>) -> Result<(), String> {
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
            let writer = writer_guard
                .as_mut()
                .ok_or("Debug adapter stdin not available")?;

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

    #[instrument(skip(self))]
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
        &self,
        source: Value,
        breakpoints: Vec<crate::types::SourceBreakpoint>,
    ) -> Result<Vec<crate::types::Breakpoint>, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DebugState;

    #[tokio::test]
    async fn test_debug_adapter_state_transitions() {
        let session = DebugSession {
            id: "test-session".to_string(),
            name: "Test Session".to_string(),
            state: DebugState::Stopped,
            adapter_type: "test-adapter".to_string(),
        };
        let adapter = DebugAdapter::new(session, "nonexistent-adapter".to_string(), vec![]);

        // Initial state should be Stopped
        assert_eq!(adapter.get_state().await, DebugAdapterState::Stopped);

        // Valid: Stopped -> Starting
        assert!(adapter
            .transition(DebugAdapterState::Starting)
            .await
            .is_ok());
        assert_eq!(adapter.get_state().await, DebugAdapterState::Starting);

        // Valid: Starting -> Initializing
        assert!(adapter
            .transition(DebugAdapterState::Initializing)
            .await
            .is_ok());
        assert_eq!(adapter.get_state().await, DebugAdapterState::Initializing);

        // Valid: Initializing -> Configured
        assert!(adapter
            .transition(DebugAdapterState::Configured)
            .await
            .is_ok());
        assert_eq!(adapter.get_state().await, DebugAdapterState::Configured);

        // Valid: Configured -> Running
        assert!(adapter.transition(DebugAdapterState::Running).await.is_ok());
        assert_eq!(adapter.get_state().await, DebugAdapterState::Running);

        // Valid: Running -> ShuttingDown
        assert!(adapter
            .transition(DebugAdapterState::ShuttingDown)
            .await
            .is_ok());
        assert_eq!(adapter.get_state().await, DebugAdapterState::ShuttingDown);

        // Valid: ShuttingDown -> Stopped
        assert!(adapter.transition(DebugAdapterState::Stopped).await.is_ok());
        assert_eq!(adapter.get_state().await, DebugAdapterState::Stopped);

        // Test Crashed -> Starting restart path
        // First go Stopped -> Starting -> Crashed
        assert!(adapter
            .transition(DebugAdapterState::Starting)
            .await
            .is_ok());
        assert!(adapter.transition(DebugAdapterState::Crashed).await.is_ok());
        assert_eq!(adapter.get_state().await, DebugAdapterState::Crashed);

        // Valid: Crashed -> Starting (restart)
        assert!(adapter
            .transition(DebugAdapterState::Starting)
            .await
            .is_ok());

        // Test invalid transitions
        let adapter2 = DebugAdapter::new(
            DebugSession {
                id: "test-2".to_string(),
                name: "Test 2".to_string(),
                state: DebugState::Stopped,
                adapter_type: "test".to_string(),
            },
            "noop".to_string(),
            vec![],
        );

        // Invalid: Stopped -> Running (must go through Starting first)
        assert!(adapter2
            .transition(DebugAdapterState::Running)
            .await
            .is_err());
        // Invalid: Stopped -> ShuttingDown
        assert!(adapter2
            .transition(DebugAdapterState::ShuttingDown)
            .await
            .is_err());
    }
}
