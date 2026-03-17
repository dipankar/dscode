/**
 * Debug Adapter with NNG IPC
 *
 * Implements Debug Adapter Protocol (DAP) communication via NNG
 */

use super::types::*;
use crate::extension_host::nng_ipc::NngExtensionIpc;
use serde_json::Value;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

pub struct DebugAdapter {
    session: DebugSession,
    process: Arc<Mutex<Option<Child>>>,
    ipc: Arc<Mutex<Option<Arc<NngExtensionIpc>>>>,
    adapter_command: String,
    adapter_args: Vec<String>,
    ipc_url: String,
    sequence: Arc<Mutex<i32>>,
}

impl DebugAdapter {
    pub fn new(
        session: DebugSession,
        adapter_command: String,
        adapter_args: Vec<String>,
    ) -> Self {
        let ipc_url = format!("ipc:///tmp/dscode-debug-{}.ipc", session.id);
        Self {
            session,
            process: Arc::new(Mutex::new(None)),
            ipc: Arc::new(Mutex::new(None)),
            adapter_command,
            adapter_args,
            ipc_url,
            sequence: Arc::new(Mutex::new(0)),
        }
    }

    /// Start the debug adapter process
    pub async fn start(&self) -> Result<(), String> {
        let mut process_guard = self.process.lock().unwrap();

        // Check if already running
        if process_guard.is_some() {
            return Ok(());
        }

        // Spawn debug adapter process with NNG IPC URL
        let child = Command::new(&self.adapter_command)
            .args(&self.adapter_args)
            .env("DAP_IPC_URL", &self.ipc_url)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start debug adapter {}: {}", self.session.adapter_type, e))?;

        println!("[Debug] Started {} debug adapter ({}) on {}",
            self.session.adapter_type, self.adapter_command, self.ipc_url);

        *process_guard = Some(child);
        drop(process_guard);

        // Wait for adapter to bind
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Connect NNG IPC
        let ipc = NngExtensionIpc::new(&self.ipc_url)?;
        *self.ipc.lock().unwrap() = Some(Arc::new(ipc));

        println!("[Debug] Connected NNG IPC for session {}", self.session.id);
        Ok(())
    }

    /// Stop the debug adapter
    pub fn stop(&self) -> Result<(), String> {
        let mut process_guard = self.process.lock().unwrap();

        if let Some(mut child) = process_guard.take() {
            child.kill()
                .map_err(|e| format!("Failed to kill debug adapter: {}", e))?;
            println!("[Debug] Stopped debug adapter for session {}", self.session.id);
        }

        // Clear IPC connection
        *self.ipc.lock().unwrap() = None;

        Ok(())
    }

    /// Check if adapter is running
    pub fn is_running(&self) -> bool {
        self.process.lock().unwrap().is_some()
    }

    fn next_sequence(&self) -> i32 {
        let mut seq = self.sequence.lock().unwrap();
        *seq += 1;
        *seq
    }

    /// Send a DAP request
    pub async fn send_request(&self, command: &str, arguments: Option<Value>) -> Result<Value, String> {
        let seq = self.next_sequence();

        let mut request = serde_json::json!({
            "seq": seq,
            "type": "request",
            "command": command,
        });

        if let Some(args) = arguments {
            request["arguments"] = args;
        }

        // Send via NNG IPC
        let ipc_guard = self.ipc.lock().unwrap();
        let ipc = ipc_guard.as_ref()
            .ok_or("Debug adapter not connected via NNG")?;

        let response = ipc.request("dap:request", request).await?;

        // Parse DAP response
        if response.get("type").and_then(|t| t.as_str()) == Some("response") {
            if response.get("success").and_then(|s| s.as_bool()) == Some(true) {
                Ok(response.get("body").cloned().unwrap_or(Value::Null))
            } else {
                let message = response.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                Err(format!("DAP error: {}", message))
            }
        } else {
            Err("Invalid DAP response".to_string())
        }
    }

    /// Send a DAP event (notification)
    pub async fn send_event(&self, event: &str, body: Option<Value>) -> Result<(), String> {
        let seq = self.next_sequence();

        let mut event_msg = serde_json::json!({
            "seq": seq,
            "type": "event",
            "event": event,
        });

        if let Some(b) = body {
            event_msg["body"] = b;
        }

        // Send via NNG IPC
        let ipc_guard = self.ipc.lock().unwrap();
        let ipc = ipc_guard.as_ref()
            .ok_or("Debug adapter not connected via NNG")?;

        ipc.send("dap:event", event_msg).await
    }

    /// Initialize the debug adapter
    pub async fn initialize(&self) -> Result<Value, String> {
        self.send_request("initialize", Some(serde_json::json!({
            "clientID": "dscode",
            "clientName": "DSCode",
            "adapterID": self.session.adapter_type,
            "pathFormat": "path",
            "linesStartAt1": true,
            "columnsStartAt1": true,
            "supportsVariableType": true,
            "supportsVariablePaging": true,
            "supportsRunInTerminalRequest": true,
        }))).await
    }

    /// Launch a debug session
    pub async fn launch(&self, configuration: Value) -> Result<(), String> {
        self.send_request("launch", Some(configuration)).await?;
        Ok(())
    }

    /// Attach to a running process
    pub async fn attach(&self, configuration: Value) -> Result<(), String> {
        self.send_request("attach", Some(configuration)).await?;
        Ok(())
    }

    /// Set breakpoints
    pub async fn set_breakpoints(&self, source: Value, breakpoints: Vec<SourceBreakpoint>) -> Result<Vec<Breakpoint>, String> {
        let response = self.send_request("setBreakpoints", Some(serde_json::json!({
            "source": source,
            "breakpoints": breakpoints,
        }))).await?;

        let bps = response.get("breakpoints")
            .and_then(|b| b.as_array())
            .ok_or("Invalid setBreakpoints response")?;

        serde_json::from_value(Value::Array(bps.clone()))
            .map_err(|e| format!("Failed to parse breakpoints: {}", e))
    }

    /// Continue execution
    pub async fn continue_execution(&self, thread_id: i32) -> Result<(), String> {
        self.send_request("continue", Some(serde_json::json!({
            "threadId": thread_id,
        }))).await?;
        Ok(())
    }

    /// Pause execution
    pub async fn pause(&self, thread_id: i32) -> Result<(), String> {
        self.send_request("pause", Some(serde_json::json!({
            "threadId": thread_id,
        }))).await?;
        Ok(())
    }

    /// Step over
    pub async fn next(&self, thread_id: i32) -> Result<(), String> {
        self.send_request("next", Some(serde_json::json!({
            "threadId": thread_id,
        }))).await?;
        Ok(())
    }

    /// Step into
    pub async fn step_in(&self, thread_id: i32) -> Result<(), String> {
        self.send_request("stepIn", Some(serde_json::json!({
            "threadId": thread_id,
        }))).await?;
        Ok(())
    }

    /// Step out
    pub async fn step_out(&self, thread_id: i32) -> Result<(), String> {
        self.send_request("stepOut", Some(serde_json::json!({
            "threadId": thread_id,
        }))).await?;
        Ok(())
    }

    /// Disconnect from debug session
    pub async fn disconnect(&self) -> Result<(), String> {
        self.send_request("disconnect", None).await?;
        Ok(())
    }
}

impl Drop for DebugAdapter {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
