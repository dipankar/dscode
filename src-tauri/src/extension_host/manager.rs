use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use super::sandbox::{apply_sandbox, SandboxConfig};

const MAX_RESTART_ATTEMPTS: u32 = 3;
const RESTART_BACKOFF_BASE_MS: u64 = 1000;

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

    pub fn start(
        &mut self, extension_host_path: &str, outgoing_socket: &str, incoming_socket: &str,
    ) -> Result<(), String> {
        println!("[ExtensionHost] Starting extension host from: {}", extension_host_path);

        let node_binary = self.resolve_node_binary()?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&node_binary, std::fs::Permissions::from_mode(0o755));
        }

        let mut cmd = Command::new(&node_binary);
        cmd.arg(extension_host_path);

        cmd.env("DSCODE_IPC_URL", outgoing_socket);
        println!("[ExtensionHost] Set DSCODE_IPC_URL={}", outgoing_socket);

        cmd.env("DSCODE_INCOMING_IPC_URL", incoming_socket);
        println!("[ExtensionHost] Set DSCODE_INCOMING_IPC_URL={}", incoming_socket);

        if let Ok(resource_dir) = std::env::var("DSCODE_RESOURCE_PATH") {
            cmd.env("DSCODE_RESOURCE_PATH", &resource_dir);
        } else if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                let resource_path = exe_dir.join("resources");
                if resource_path.exists() {
                    cmd.env("DSCODE_RESOURCE_PATH", resource_path.to_string_lossy().to_string());
                }
            }
        }

        let sandbox_config = SandboxConfig::default();
        apply_sandbox(&mut cmd, &sandbox_config)?;

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child =
            cmd.spawn().map_err(|e| format!("Failed to spawn extension host: {}", e))?;

        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("[ExtensionHost:stdout] {}", line);
                }
            }
        });

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

    fn resolve_node_binary(&self) -> Result<String, String> {
        #[cfg(target_os = "windows")]
        let node_binary_name = "node.exe";
        #[cfg(not(target_os = "windows"))]
        let node_binary_name = "node";

        let target_triple = std::env::consts::ARCH.to_string()
            + "-"
            + match std::env::consts::OS {
                "macos" => "apple-darwin",
                "linux" => "unknown-linux-gnu",
                "windows" => "pc-windows-msvc",
                other => other,
            };

        #[cfg(target_os = "windows")]
        let ext_bin_name = format!("node-{}.exe", target_triple);
        #[cfg(not(target_os = "windows"))]
        let ext_bin_name = format!("node-{}", target_triple);

        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                let bundled = exe_dir.join(&ext_bin_name);
                if bundled.exists() {
                    println!("[ExtensionHost] Using bundled Node.js (externalBin): {:?}", bundled);
                    return Ok(bundled.to_string_lossy().to_string());
                }

                let bundled = exe_dir.join(node_binary_name);
                if bundled.exists() {
                    println!("[ExtensionHost] Using bundled Node.js: {:?}", bundled);
                    return Ok(bundled.to_string_lossy().to_string());
                }
            }
        }

        if let Ok(resource_dir) = std::env::var("DSCODE_RESOURCE_PATH") {
            let bundled = std::path::Path::new(&resource_dir).join(node_binary_name);
            if bundled.exists() {
                println!("[ExtensionHost] Using bundled Node.js: {:?}", bundled);
                return Ok(bundled.to_string_lossy().to_string());
            }
        }

        match which::which("node") {
            Ok(path) => {
                println!("[ExtensionHost] Using system Node.js: {:?}", path);
                Ok(path.to_string_lossy().to_string())
            }
            Err(_) => {
                Err("Node.js not found. Install Node.js or bundle it with the application."
                    .to_string())
            }
        }
    }

    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(None) => true,
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

    pub fn is_healthy(&mut self) -> bool {
        if !self.is_running() {
            return false;
        }

        if let Some(started) = self.started_at {
            started.elapsed() > Duration::from_millis(5000)
        } else {
            false
        }
    }

    pub async fn restart_if_needed(
        &mut self, extension_host_path: &str, outgoing_socket: &str, incoming_socket: &str,
    ) -> Result<(), String> {
        if self.is_running() {
            return Ok(());
        }

        if self.restart_count >= MAX_RESTART_ATTEMPTS {
            eprintln!("[ExtensionHost] Max restart attempts ({}) exceeded", MAX_RESTART_ATTEMPTS);
            return Err(format!(
                "Extension host crashed and max restart attempts ({}) exceeded",
                MAX_RESTART_ATTEMPTS
            ));
        }

        let backoff_ms = RESTART_BACKOFF_BASE_MS * 2u64.pow(self.restart_count);
        println!(
            "[ExtensionHost] Restarting after crash (attempt {}/{}, waiting {}ms)",
            self.restart_count + 1,
            MAX_RESTART_ATTEMPTS,
            backoff_ms
        );

        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;

        self.restart_count += 1;
        self.last_restart_time = Some(Instant::now());

        self.start(extension_host_path, outgoing_socket, incoming_socket)?;

        println!("[ExtensionHost] Restart successful (attempt {})", self.restart_count);
        Ok(())
    }

    pub fn reset_restart_count(&mut self) {
        if self.restart_count > 0 && self.is_healthy() {
            self.restart_count = 0;
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.child.take() {
            println!("[ExtensionHost] Stopping process (PID: {})...", child.id());

            match child.try_wait() {
                Ok(None) => {
                    child.kill().map_err(|e| format!("Failed to kill extension host: {}", e))?;

                    match child.wait() {
                        Ok(status) => {
                            println!("[ExtensionHost] Process exited with status: {}", status)
                        }
                        Err(e) => eprintln!("[ExtensionHost] Error waiting for process: {}", e),
                    }
                }
                Ok(Some(status)) => {
                    println!("[ExtensionHost] Process already exited with status: {}", status);
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

    pub fn shutdown(&mut self) {
        let _ = self.stop();
    }
}

impl Drop for ExtensionHostManager {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
