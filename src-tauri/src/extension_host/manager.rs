use std::io::{BufRead, BufReader};
/**
 * Extension Host Process Manager
 *
 * Manages the Node.js extension host process lifecycle.
 * All IPC communication happens via NNG (managed by NngIpcManager).
 * This manager is only responsible for:
 * - Starting the extension host process with proper environment
 * - Monitoring process health
 * - Stopping the process
 * - Crash recovery and automatic restart
 */
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use super::sandbox::{apply_sandbox, SandboxConfig};

const MAX_RESTART_ATTEMPTS: u32 = 3;
const RESTART_BACKOFF_BASE_MS: u64 = 1000;
const HEALTH_CHECK_INTERVAL_MS: u64 = 5000;
const STARTUP_TIMEOUT_MS: u64 = 15000;

pub struct ExtensionHostManager {
    child: Option<Child>,
    pub extension_id: String,
    restart_count: u32,
    last_restart_time: Option<Instant>,
    started_at: Option<Instant>,
    health_check_running: Arc<AtomicBool>,
}

impl ExtensionHostManager {
    pub fn new(extension_id: String) -> Self {
        Self {
            child: None,
            extension_id,
            restart_count: 0,
            last_restart_time: None,
            started_at: None,
            health_check_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the extension host process with NNG IPC URLs
    pub fn start_with_nng(
        &mut self,
        extension_host_path: &str,
        outgoing_ipc_url: &str,
        incoming_ipc_url: &str,
    ) -> Result<(), String> {
        println!(
            "[ExtensionHost] Starting extension host from: {}",
            extension_host_path
        );

        let mut cmd = Command::new("node");
        cmd.arg(extension_host_path);

        // Set outgoing IPC URL (ExtHost listens, Tauri connects)
        cmd.env("DSCODE_IPC_URL", outgoing_ipc_url);
        println!("[ExtensionHost] Set DSCODE_IPC_URL={}", outgoing_ipc_url);

        // Set incoming IPC URL (Tauri listens, ExtHost connects)
        cmd.env("DSCODE_INCOMING_IPC_URL", incoming_ipc_url);
        println!(
            "[ExtensionHost] Set DSCODE_INCOMING_IPC_URL={}",
            incoming_ipc_url
        );

        // Apply sandboxing with restrictive defaults
        let sandbox_config = SandboxConfig::default();
        apply_sandbox(&mut cmd, &sandbox_config)?;

        // Configure stdio
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn extension host: {}", e))?;

        // Get stdout for logging
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;

        // Get stderr for logging
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

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
                    eprintln!("[ExtensionHost:stderr] {}", line);
                }
            }
        });

        self.started_at = Some(Instant::now());
        self.child = Some(child);
        println!(
            "[ExtensionHost] Process started successfully (PID: {:?})",
            self.child.as_ref().map(|c| c.id())
        );

        Ok(())
    }

    /// Check if the extension host is running
    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(None) => true, // Still running
                Ok(Some(status)) => {
                    println!("[ExtensionHost] Process exited with status: {}", status);
                    false
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// Check if the process has been running long enough to be considered "healthy"
    pub fn is_healthy(&mut self) -> bool {
        if !self.is_running() {
            return false;
        }

        if let Some(started) = self.started_at {
            // Consider healthy if it's been running for more than 5 seconds
            started.elapsed() > Duration::from_millis(5000)
        } else {
            false
        }
    }

    /// Attempt to restart the extension host after a crash
    /// Returns Ok if restart was successful, Err if max attempts exceeded
    pub fn restart_if_needed(
        &mut self,
        extension_host_path: &str,
        outgoing_ipc_url: &str,
        incoming_ipc_url: &str,
    ) -> Result<(), String> {
        if self.is_running() {
            return Ok(());
        }

        if self.restart_count >= MAX_RESTART_ATTEMPTS {
            eprintln!(
                "[ExtensionHost] Max restart attempts ({}) exceeded",
                MAX_RESTART_ATTEMPTS
            );
            return Err(format!(
                "Extension host crashed and max restart attempts ({}) exceeded",
                MAX_RESTART_ATTEMPTS
            ));
        }

        // Exponential backoff: 1s, 2s, 4s
        let backoff_ms = RESTART_BACKOFF_BASE_MS * 2u64.pow(self.restart_count);
        println!(
            "[ExtensionHost] Restarting after crash (attempt {}/{}, waiting {}ms)",
            self.restart_count + 1,
            MAX_RESTART_ATTEMPTS,
            backoff_ms
        );

        // Wait with backoff
        std::thread::sleep(Duration::from_millis(backoff_ms));

        self.restart_count += 1;
        self.last_restart_time = Some(Instant::now());

        // Reset started_at so health check works
        self.start_with_nng(extension_host_path, outgoing_ipc_url, incoming_ipc_url)?;

        println!(
            "[ExtensionHost] Restart successful (attempt {})",
            self.restart_count
        );
        Ok(())
    }

    /// Reset the restart counter after a successful run
    pub fn reset_restart_count(&mut self) {
        if self.restart_count > 0 && self.is_healthy() {
            self.restart_count = 0;
        }
    }

    /// Stop the extension host
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.child.take() {
            println!("[ExtensionHost] Stopping process (PID: {})...", child.id());

            // Try graceful shutdown first (SIGTERM on Unix)
            match child.try_wait() {
                Ok(None) => {
                    // Process is still running, kill it
                    child
                        .kill()
                        .map_err(|e| format!("Failed to kill extension host: {}", e))?;

                    // Wait up to 5 seconds for the process to exit
                    match child.wait() {
                        Ok(status) => {
                            println!("[ExtensionHost] Process exited with status: {}", status)
                        }
                        Err(e) => eprintln!("[ExtensionHost] Error waiting for process: {}", e),
                    }
                }
                Ok(Some(status)) => {
                    println!(
                        "[ExtensionHost] Process already exited with status: {}",
                        status
                    );
                }
                Err(e) => {
                    println!("[ExtensionHost] Error checking process status: {}", e);
                }
            }

            self.health_check_running.store(false, Ordering::Relaxed);
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
