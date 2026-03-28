use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::sandbox::{apply_sandbox, SandboxConfig};

/// STATE MACHINE: ExtensionHostProcess
///
/// Tracks the lifecycle of the Node.js extension host child process.
///
/// State Diagram:
///
///   Stopped ──────► Starting ──────► Running
///     ▲                │                │
///     │    (spawn fail)│   (unexpected) │  (stop() called)
///     │                ▼      (exit)    ▼
///     │             Crashed ◄──── Unhealthy      Stopping
///     │                ▲              │              │
///     │   (max retries)│              │ (within      │
///     │                │              │  budget)     │
///     │                │              ▼              │
///     │                └──────── Restarting          │
///     │                                             │
///     └─────────────────────────────────────────────┘
///
/// Transitions:
///   Stopped    -> Starting    (start() called)
///   Starting   -> Running     (process spawned, stdout/stderr threads launched)
///   Starting   -> Crashed     (Command::spawn() failed)
///   Running    -> Unhealthy   (process exited unexpectedly, detected by try_wait)
///   Running    -> Stopping    (stop() called for graceful shutdown)
///   Unhealthy  -> Restarting  (restart_if_needed(), within MAX_RESTART_ATTEMPTS)
///   Unhealthy  -> Crashed     (restart_if_needed(), attempts exhausted)
///   Restarting -> Starting    (backoff elapsed, re-entering start flow)
///   Stopping   -> Stopped     (process killed and waited successfully)
///   Crashed    -> Starting    (explicit start() after crash, resets restart_count)
///
/// Concurrency Invariant:
///   All access to ExtensionHostManager is serialized by the
///   Arc<tokio::sync::Mutex<ExtensionHostManager>> held in SessionManager.
///   No method on this struct should be called without holding that mutex.
///   This means state transitions are inherently atomic — no TOCTOU possible
///   as long as callers hold the mutex across check-then-act sequences.
///
/// Interruption Table:
/// ┌─────────────┬──────────────────────────────────────────────────────────┐
/// │ State       │ What happens + recovery path                            │
/// ├─────────────┼──────────────────────────────────────────────────────────┤
/// │ Stopped     │ Safe resting state. No child process, no resources.     │
/// │             │ Can call start() to launch a new host.                  │
/// ├─────────────┼──────────────────────────────────────────────────────────┤
/// │ Starting    │ Child process spawning or just spawned.                 │
/// │             │ If Tauri crashes: child process orphaned, OS reaps.     │
/// │             │ If child spawn fails: -> Crashed immediately.           │
/// │             │ If child exits within 5s: next state check -> Unhealthy.│
/// │             │ stdout/stderr reader threads may be running — they exit │
/// │             │ naturally when child's pipes close.                     │
/// ├─────────────┼──────────────────────────────────────────────────────────┤
/// │ Running     │ Normal operation. Process alive and healthy.            │
/// │             │ If child exits unexpectedly: detected on next           │
/// │             │ check_and_update_state() call -> Unhealthy.             │
/// │             │ All pending IPC requests are orphaned (oneshot senders  │
/// │             │ dropped, receivers get RecvError or timeout).           │
/// │             │ Frontend NOT notified — stale state until next command. │
/// │             │ Backend SessionState still shows extensions as active.  │
/// │             │ TODO: Add proactive health monitoring (periodic check). │
/// ├─────────────┼──────────────────────────────────────────────────────────┤
/// │ Unhealthy   │ Process has exited. No child process running.           │
/// │             │ restart_if_needed() should be called to attempt         │
/// │             │ recovery. If within budget (3 attempts):                │
/// │             │   -> Restarting with exponential backoff (1s, 2s, 4s).  │
/// │             │ If budget exhausted: -> Crashed (terminal).             │
/// ├─────────────┼──────────────────────────────────────────────────────────┤
/// │ Restarting  │ Waiting for backoff timer before re-spawn.              │
/// │             │ If Tauri crashes during wait: no cleanup needed (no     │
/// │             │ child process exists yet).                              │
/// │             │ After backoff: -> Starting -> Running (or -> Crashed).  │
/// ├─────────────┼──────────────────────────────────────────────────────────┤
/// │ Stopping    │ Graceful shutdown in progress. Kill signal sent.        │
/// │             │ If process ignores SIGTERM: force kill + wait.          │
/// │             │ -> Stopped after process exits.                         │
/// │             │ If Tauri crashes during stop: child may linger          │
/// │             │ (no SIGCHLD handler). OS eventually reaps.              │
/// ├─────────────┼──────────────────────────────────────────────────────────┤
/// │ Crashed     │ Terminal failure state. Max restart attempts exhausted. │
/// │             │ All extensions in this host are effectively dead.       │
/// │             │ Backend state still shows extensions as active (stale). │
/// │             │ Frontend shows stale extension state.                   │
/// │             │ Recovery: explicit start() call resets restart_count    │
/// │             │ and re-enters Starting. Alternatively, app restart.     │
/// └─────────────┴──────────────────────────────────────────────────────────┘
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionHostState {
    Stopped,
    Starting,
    Running,
    Unhealthy,
    Restarting,
    Stopping,
    Crashed,
}

const MAX_RESTART_ATTEMPTS: u32 = 3;
const RESTART_BACKOFF_BASE_MS: u64 = 1000;

pub struct ExtensionHostManager {
    child: Option<Child>,
    pub extension_id: String,
    restart_count: u32,
    last_restart_time: Option<Instant>,
    started_at: Option<Instant>,
    state: ExtensionHostState,
}

impl ExtensionHostManager {
    pub fn new(extension_id: String) -> Self {
        Self {
            child: None,
            extension_id,
            restart_count: 0,
            last_restart_time: None,
            started_at: None,
            state: ExtensionHostState::Stopped,
        }
    }

    /// Validates and performs a state transition.
    /// Returns Ok(()) on valid transition, Err with details on invalid.
    /// Invalid transitions are logged but do NOT panic — graceful degradation.
    fn transition(&mut self, to: ExtensionHostState) -> Result<(), String> {
        let valid = match self.state {
            ExtensionHostState::Stopped => matches!(to, ExtensionHostState::Starting),
            ExtensionHostState::Starting => {
                matches!(to, ExtensionHostState::Running | ExtensionHostState::Crashed)
            }
            ExtensionHostState::Running => {
                matches!(to, ExtensionHostState::Unhealthy | ExtensionHostState::Stopping)
            }
            ExtensionHostState::Unhealthy => {
                matches!(to, ExtensionHostState::Restarting | ExtensionHostState::Crashed)
            }
            ExtensionHostState::Restarting => matches!(to, ExtensionHostState::Starting),
            ExtensionHostState::Stopping => matches!(to, ExtensionHostState::Stopped),
            ExtensionHostState::Crashed => matches!(to, ExtensionHostState::Starting),
        };

        if valid {
            println!("[ExtensionHost] State: {:?} -> {:?}", self.state, to);
            self.state = to;
            Ok(())
        } else {
            let msg = format!(
                "[ExtensionHost] Invalid state transition: {:?} -> {:?}",
                self.state, to
            );
            eprintln!("{}", msg);
            Err(msg)
        }
    }

    /// Returns the current state of the extension host process.
    /// Callers should use this instead of the removed is_running()/is_healthy() methods.
    pub fn state(&self) -> ExtensionHostState {
        self.state
    }

    /// Checks if the child process is still alive and updates state accordingly.
    /// Call this before making decisions based on host state.
    /// This is the ONLY place where Running -> Unhealthy transition happens passively.
    pub fn check_and_update_state(&mut self) {
        if self.state != ExtensionHostState::Running {
            return;
        }
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(status)) => {
                    println!("[ExtensionHost] Process exited with status: {}", status);
                    let _ = self.transition(ExtensionHostState::Unhealthy);
                }
                Ok(None) => {} // Still running
                Err(e) => {
                    eprintln!("[ExtensionHost] Failed to check process status: {}", e);
                    let _ = self.transition(ExtensionHostState::Unhealthy);
                }
            }
        }
    }

    pub fn start(
        &mut self, extension_host_path: &str, outgoing_socket: &str, incoming_socket: &str,
    ) -> Result<(), String> {
        self.transition(ExtensionHostState::Starting)?;

        println!("[ExtensionHost] Starting extension host from: {}", extension_host_path);

        let node_binary = match self.resolve_node_binary() {
            Ok(bin) => bin,
            Err(e) => {
                let _ = self.transition(ExtensionHostState::Crashed);
                return Err(e);
            }
        };

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
        if let Err(e) = apply_sandbox(&mut cmd, &sandbox_config) {
            let _ = self.transition(ExtensionHostState::Crashed);
            return Err(e);
        }

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                let _ = self.transition(ExtensionHostState::Crashed);
                return Err(format!("Failed to spawn extension host: {}", e));
            }
        };

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
        let _ = self.transition(ExtensionHostState::Running);
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

    pub async fn restart_if_needed(
        &mut self, extension_host_path: &str, outgoing_socket: &str, incoming_socket: &str,
    ) -> Result<(), String> {
        // If we think we're running, check whether the process actually exited.
        if self.state == ExtensionHostState::Running {
            self.check_and_update_state();
        }

        if self.state != ExtensionHostState::Unhealthy {
            return Err(format!(
                "Cannot restart from state {:?} — must be Unhealthy",
                self.state
            ));
        }

        if self.restart_count >= MAX_RESTART_ATTEMPTS {
            eprintln!("[ExtensionHost] Max restart attempts ({}) exceeded", MAX_RESTART_ATTEMPTS);
            let _ = self.transition(ExtensionHostState::Crashed);
            return Err(format!(
                "Extension host crashed and max restart attempts ({}) exceeded",
                MAX_RESTART_ATTEMPTS
            ));
        }

        let _ = self.transition(ExtensionHostState::Restarting);

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

        // Transition Restarting -> Starting happens inside start()
        self.start(extension_host_path, outgoing_socket, incoming_socket)?;

        println!("[ExtensionHost] Restart successful (attempt {})", self.restart_count);
        Ok(())
    }

    pub fn reset_restart_count(&mut self) {
        if self.restart_count > 0 && self.state == ExtensionHostState::Running {
            if let Some(started) = self.started_at {
                if started.elapsed() > Duration::from_millis(5000) {
                    self.restart_count = 0;
                }
            }
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        // Allow stop() from any state by force-setting Stopping.
        // This is intentionally permissive — stop() is a cleanup operation.
        if self.state != ExtensionHostState::Stopped {
            // Force state to Stopping regardless of current state.
            println!("[ExtensionHost] State: {:?} -> {:?}", self.state, ExtensionHostState::Stopping);
            self.state = ExtensionHostState::Stopping;
        }

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

            println!("[ExtensionHost] Process stopped");
        }

        if self.state == ExtensionHostState::Stopping {
            let _ = self.transition(ExtensionHostState::Stopped);
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
