/**
 * Extension Host Process Manager
 *
 * Manages the Node.js extension host process lifecycle.
 * All IPC communication happens via NNG (managed by NngIpcManager).
 * This manager is only responsible for:
 * - Starting the extension host process with proper environment
 * - Monitoring process health
 * - Stopping the process
 */

use std::process::{Child, Command, Stdio};
use std::thread;
use std::io::{BufRead, BufReader};

use super::sandbox::{SandboxConfig, apply_sandbox};

pub struct ExtensionHostManager {
    child: Option<Child>,
    pub extension_id: String,
}

impl ExtensionHostManager {
    pub fn new(extension_id: String) -> Self {
        Self {
            child: None,
            extension_id,
        }
    }

    /// Start the extension host process with NNG IPC URLs
    pub fn start_with_nng(&mut self, extension_host_path: &str, outgoing_ipc_url: &str, incoming_ipc_url: &str) -> Result<(), String> {
        println!("[ExtensionHost] Starting extension host from: {}", extension_host_path);

        let mut cmd = Command::new("node");
        cmd.arg(extension_host_path);

        // Set outgoing IPC URL (ExtHost listens, Tauri connects)
        cmd.env("DSCODE_IPC_URL", outgoing_ipc_url);
        println!("[ExtensionHost] Set DSCODE_IPC_URL={}", outgoing_ipc_url);

        // Set incoming IPC URL (Tauri listens, ExtHost connects)
        cmd.env("DSCODE_INCOMING_IPC_URL", incoming_ipc_url);
        println!("[ExtensionHost] Set DSCODE_INCOMING_IPC_URL={}", incoming_ipc_url);

        // Apply sandboxing
        let sandbox_config = SandboxConfig::default();
        apply_sandbox(&mut cmd, &sandbox_config)?;

        // Configure stdio - we only need stderr for logging
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn extension host: {}", e))?;

        // Get stdout for logging (extensions might log to stdout)
        let stdout = child.stdout.take()
            .ok_or("Failed to capture stdout")?;

        // Get stderr for logging
        let stderr = child.stderr.take()
            .ok_or("Failed to capture stderr")?;

        // Spawn thread to read and log stdout
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("[ExtensionHost:stdout] {}", line);
                }
            }
        });

        // Spawn thread to read and log stderr
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("[ExtensionHost:stderr] {}", line);
                }
            }
        });

        self.child = Some(child);
        println!("[ExtensionHost] Process started successfully (PID: {:?})",
            self.child.as_ref().map(|c| c.id()));

        Ok(())
    }

    /// Check if the extension host is running
    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            // Try to check if process is still alive
            match child.try_wait() {
                Ok(None) => true,  // Still running
                Ok(Some(_)) => false,  // Exited
                Err(_) => false,  // Error checking
            }
        } else {
            false
        }
    }

    /// Stop the extension host
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.child.take() {
            println!("[ExtensionHost] Stopping process (PID: {})...", child.id());

            // Try graceful shutdown first (if process is still running)
            match child.try_wait() {
                Ok(None) => {
                    // Process is still running, kill it
                    child.kill()
                        .map_err(|e| format!("Failed to kill extension host: {}", e))?;
                    child.wait()
                        .map_err(|e| format!("Failed to wait for extension host: {}", e))?;
                }
                Ok(Some(status)) => {
                    println!("[ExtensionHost] Process already exited with status: {}", status);
                }
                Err(e) => {
                    println!("[ExtensionHost] Error checking process status: {}", e);
                }
            }

            println!("[ExtensionHost] Process stopped");
        }
        Ok(())
    }

    /// Shutdown (alias for stop for compatibility)
    pub fn shutdown(&mut self) {
        let _ = self.stop();
    }
}

impl Drop for ExtensionHostManager {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
