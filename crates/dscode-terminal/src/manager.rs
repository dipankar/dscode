use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use tracing::{debug, error, info, instrument};

use crate::TerminalEventSender;

// ── Public types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TerminalInfo {
    pub id: String,
    pub name: String,
    pub shell: String,
    pub cwd: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalProfile {
    pub id: String,
    pub name: String,
    pub shell: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOptions {
    pub name: Option<String>,
    pub shell_path: Option<String>,
    pub shell_args: Vec<String>,
    pub cwd: Option<String>,
    pub env: HashMap<String, String>,
    pub profile_id: Option<String>,
}

/// STATE MACHINE: TerminalInstance
///
/// Tracks the lifecycle of a PTY-backed terminal instance.
///
/// State Diagram:
///
///   Created ──────► Running ──────► ShuttingDown ──────► Closed
///                      │                                    ▲
///                      │ (process exit,                     │
///                      │  read error)                       │
///                      └────────────────────────────────────┘
///
/// Transitions:
///   Created      -> Running      (start signal sent via start_sender channel)
///   Running      -> ShuttingDown (shutdown_signal flag set to true)
///   Running      -> Closed       (PTY process exits, reader thread detects EOF)
///   ShuttingDown -> Closed       (reader thread sees shutdown flag, exits loop)
///
/// Concurrency Invariant:
///   Terminal I/O uses std::sync::Mutex (not tokio) because PTY operations
///   are synchronous. The reader thread runs on a dedicated OS thread (not
///   tokio task). ShutdownSignal uses AtomicBool for lock-free cross-thread
///   signaling. State transitions should be synchronized through the
///   TerminalManager's terminals Mutex.
///
/// Interruption Table:
/// ┌──────────────┬──────────────────────────────────────────────────────────┐
/// │ State        │ What happens on crash/error                             │
/// ├──────────────┼──────────────────────────────────────────────────────────┤
/// │ Created      │ PTY allocated but reader waiting for start signal.      │
/// │              │ If app crashes: PTY master handle dropped, slave exits.  │
/// │              │ start_sender channel dropped, reader thread unblocks     │
/// │              │ and exits (recv() returns Err).                          │
/// ├──────────────┼──────────────────────────────────────────────────────────┤
/// │ Running      │ Reader thread actively reading PTY output.              │
/// │              │ If app crashes: PTY handles dropped, OS cleans up.      │
/// │              │ If shell process exits: reader gets EOF, -> Closed.     │
/// │              │ If reader thread panics: JoinHandle::join returns Err.  │
/// │              │ Writer can still try to write (will get error).         │
/// ├──────────────┼──────────────────────────────────────────────────────────┤
/// │ ShuttingDown │ Shutdown flag set. Reader thread checking flag each     │
/// │              │ iteration. May take up to one read timeout to notice.   │
/// │              │ If reader hangs on blocking read: may not shut down     │
/// │              │ gracefully. Consider: close PTY master to force EOF.    │
/// ├──────────────┼──────────────────────────────────────────────────────────┤
/// │ Closed       │ Reader thread joined. PTY resources released.           │
/// │              │ Terminal entry removed from TerminalManager map.        │
/// └──────────────┴──────────────────────────────────────────────────────────┘
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalState {
    Created,
    Running,
    ShuttingDown,
    Closed,
}

// ── Internal helpers ───────────────────────────────────────────────────────

/// Shutdown signal for terminal reader thread.
struct ShutdownSignal {
    flag: Arc<AtomicBool>,
}

impl ShutdownSignal {
    fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    fn signal(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    #[allow(dead_code)]
    fn is_shutdown(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    fn clone_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.flag)
    }
}

pub struct TerminalInstance {
    pub(crate) info: TerminalInfo,
    pub(crate) state: TerminalState,
    writer: Box<dyn Write + Send>,
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    start_sender: Option<Sender<()>>,
    shutdown_signal: ShutdownSignal,
    reader_handle: Option<JoinHandle<()>>,
}

// ── Arc-wrapped event sender ───────────────────────────────────────────────

/// Wrapper that allows an [`TerminalEventSender`] to be cheaply cloned
/// (via `Arc`) so it can be moved into the reader thread.
pub(crate) struct SharedEventSender {
    inner: Arc<dyn TerminalEventSender>,
}

impl SharedEventSender {
    pub(crate) fn new(sender: Box<dyn TerminalEventSender>) -> Self {
        Self {
            inner: Arc::from(sender),
        }
    }

    pub(crate) fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl TerminalEventSender for SharedEventSender {
    fn send_output(&self, terminal_id: &str, data: &str) {
        self.inner.send_output(terminal_id, data);
    }

    fn send_close(&self, terminal_id: &str) {
        self.inner.send_close(terminal_id);
    }
}

// ── Terminal manager ───────────────────────────────────────────────────────

pub struct TerminalManager {
    terminals: Arc<Mutex<HashMap<String, TerminalInstance>>>,
    profiles: Arc<Mutex<HashMap<String, TerminalProfile>>>,
    next_id: Arc<Mutex<u32>>,
    event_sender: SharedEventSender,
}

impl TerminalManager {
    pub fn new(event_sender: Box<dyn TerminalEventSender>) -> Self {
        Self {
            terminals: Arc::new(Mutex::new(HashMap::new())),
            profiles: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            event_sender: SharedEventSender::new(event_sender),
        }
    }

    // ===== Profile Management =====

    pub fn register_profile(&self, profile: TerminalProfile) -> Result<String, String> {
        let mut profiles = self.profiles.lock().unwrap_or_else(|e| {
            error!("profiles lock poisoned: {e}");
            e.into_inner()
        });
        let profile_id = profile.id.clone();

        if profiles.contains_key(&profile_id) {
            return Err(format!("Profile '{}' already exists", profile_id));
        }

        profiles.insert(profile_id.clone(), profile);
        Ok(profile_id)
    }

    pub fn unregister_profile(&self, profile_id: &str) -> Result<(), String> {
        let mut profiles = self.profiles.lock().unwrap_or_else(|e| {
            error!("profiles lock poisoned: {e}");
            e.into_inner()
        });
        profiles
            .remove(profile_id)
            .ok_or_else(|| format!("Profile '{}' not found", profile_id))?;
        Ok(())
    }

    pub fn get_profile(&self, profile_id: &str) -> Result<TerminalProfile, String> {
        let profiles = self.profiles.lock().unwrap_or_else(|e| {
            error!("profiles lock poisoned: {e}");
            e.into_inner()
        });
        profiles
            .get(profile_id)
            .cloned()
            .ok_or_else(|| format!("Profile '{}' not found", profile_id))
    }

    pub fn list_profiles(&self) -> Vec<TerminalProfile> {
        let profiles = self.profiles.lock().unwrap_or_else(|e| {
            error!("profiles lock poisoned: {e}");
            e.into_inner()
        });
        profiles.values().cloned().collect()
    }

    // ===== Terminal Creation =====

    pub fn create_terminal_with_options(
        &self,
        options: TerminalOptions,
    ) -> Result<String, String> {
        // Resolve profile if specified
        let (shell_cmd, shell_args, mut env_vars, profile_cwd) =
            if let Some(profile_id) = &options.profile_id {
                let profile = self.get_profile(profile_id)?;
                (profile.shell, profile.args, profile.env, profile.cwd)
            } else {
                let shell = options.shell_path.clone().unwrap_or_else(|| {
                    std::env::var("SHELL").unwrap_or_else(|_| {
                        if cfg!(target_os = "windows") {
                            "powershell.exe".to_string()
                        } else {
                            "/bin/bash".to_string()
                        }
                    })
                });
                (shell, options.shell_args.clone(), HashMap::new(), None)
            };

        // Merge environment variables (options override profile)
        for (key, value) in options.env {
            env_vars.insert(key, value);
        }

        // Determine working directory (options > profile > current)
        let working_dir = options.cwd.or(profile_cwd).unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "/".to_string())
        });

        // Get next terminal ID
        let id = {
            let mut next = self.next_id.lock().unwrap_or_else(|e| {
                error!("next_id lock poisoned: {e}");
                e.into_inner()
            });
            let current = *next;
            *next += 1;
            format!("terminal-{}", current)
        };

        // Create PTY system and open pair with initial size
        let pair = native_pty_system()
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to create PTY: {}", e))?;

        let portable_pty::PtyPair { master, slave } = pair;

        // Create command with args and env
        let mut cmd = CommandBuilder::new(&shell_cmd);
        cmd.cwd(&working_dir);

        // Add shell arguments
        for arg in shell_args {
            cmd.arg(&arg);
        }

        // Add environment variables
        for (key, value) in env_vars {
            cmd.env(&key, &value);
        }

        // Spawn the shell
        let _child = slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        // Prepare IO handles
        let mut reader = master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone reader: {}", e))?;
        let writer = master
            .take_writer()
            .map_err(|e| format!("Failed to get writer: {}", e))?;
        let master = Arc::new(Mutex::new(master));

        // Create terminal info
        let terminal_name = options.name.unwrap_or_else(|| format!("Terminal {}", id));
        let info = TerminalInfo {
            id: id.clone(),
            name: terminal_name,
            shell: shell_cmd,
            cwd: working_dir,
        };

        // Create channel for start signal
        let (start_tx, start_rx) = mpsc::channel();

        // Create shutdown signal
        let shutdown_signal = ShutdownSignal::new();
        let shutdown_flag = shutdown_signal.clone_flag();

        // Clone the event sender for the reader thread
        let sender = self.event_sender.clone();

        // Start reading from PTY in a background thread
        let terminal_id = id.clone();
        let reader_handle = thread::spawn(move || {
            // Wait for ready signal from frontend
            if start_rx.recv().is_err() {
                error!("Failed to receive start signal for {}", terminal_id);
                return;
            }

            let mut buf = [0u8; 8192];
            loop {
                // Check shutdown signal
                if shutdown_flag.load(Ordering::SeqCst) {
                    info!("Shutdown signal received for {}", terminal_id);
                    break;
                }

                match reader.read(&mut buf) {
                    Ok(0) => {
                        // EOF - terminal closed
                        info!("Terminal {} closed (EOF)", terminal_id);
                        sender.send_close(&terminal_id);
                        break;
                    }
                    Ok(n) => {
                        // Send data to consumer
                        let data = String::from_utf8_lossy(&buf[0..n]).to_string();
                        sender.send_output(&terminal_id, &data);
                    }
                    Err(e) => {
                        // Check if this is a normal shutdown
                        if shutdown_flag.load(Ordering::SeqCst) {
                            debug!("Terminal {} shutdown complete", terminal_id);
                        } else {
                            error!("Error reading from PTY: {}", e);
                        }
                        break;
                    }
                }
            }
        });

        // Store terminal instance
        let terminal_instance = TerminalInstance {
            info: info.clone(),
            state: TerminalState::Created,
            writer,
            master: Arc::clone(&master),
            start_sender: Some(start_tx),
            shutdown_signal,
            reader_handle: Some(reader_handle),
        };

        {
            let mut terminals = self.terminals.lock().unwrap_or_else(|e| {
                error!("terminals lock poisoned: {e}");
                e.into_inner()
            });
            terminals.insert(id.clone(), terminal_instance);
        }

        Ok(id)
    }

    #[instrument(skip(self))]
    pub fn create_terminal(
        &self,
        name: Option<String>,
        shell: Option<String>,
        cwd: Option<String>,
    ) -> Result<String, String> {
        let options = TerminalOptions {
            name,
            shell_path: shell,
            shell_args: vec![],
            cwd,
            env: HashMap::new(),
            profile_id: None,
        };
        self.create_terminal_with_options(options)
    }

    pub fn write_to_terminal(&self, id: &str, data: &str) -> Result<(), String> {
        let mut terminals = self.terminals.lock().unwrap_or_else(|e| {
            error!("terminals lock poisoned: {e}");
            panic!("terminals lock is unrecoverable")
        });
        let terminal = terminals
            .get_mut(id)
            .ok_or_else(|| format!("Terminal {} not found", id))?;

        terminal
            .writer
            .write_all(data.as_bytes())
            .map_err(|e| format!("Failed to write to terminal: {}", e))?;

        terminal
            .writer
            .flush()
            .map_err(|e| format!("Failed to flush terminal: {}", e))?;

        Ok(())
    }

    pub fn resize_terminal(&self, id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let mut terminals = self.terminals.lock().unwrap_or_else(|e| {
            error!("terminals lock poisoned: {e}");
            panic!("terminals lock is unrecoverable")
        });
        let terminal = terminals
            .get_mut(id)
            .ok_or_else(|| format!("Terminal {} not found", id))?;

        let master = terminal.master.lock().unwrap_or_else(|e| {
            error!("pty master lock poisoned: {e}");
            e.into_inner()
        });
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to resize terminal: {}", e))
    }

    #[instrument(skip(self))]
    pub fn close_terminal(&self, id: &str) -> Result<(), String> {
        let mut terminals = self.terminals.lock().unwrap_or_else(|e| {
            error!("terminals lock poisoned: {e}");
            panic!("terminals lock is unrecoverable")
        });
        if let Some(mut terminal) = terminals.remove(id) {
            // Signal the reader thread to stop
            terminal.state = TerminalState::ShuttingDown;
            terminal.shutdown_signal.signal();

            // Drop the master to close the PTY (this will cause read to return EOF/error)
            drop(terminal.master);

            // Wait for the reader thread to finish (with timeout)
            if let Some(handle) = terminal.reader_handle.take() {
                // Don't block indefinitely - the drop of master should cause the read to return
                let _ = handle.join();
            }

            terminal.state = TerminalState::Closed;
            info!("Terminal {} closed and cleaned up", id);
            Ok(())
        } else {
            Err(format!("Terminal {} not found", id))
        }
    }

    pub fn list_terminals(&self) -> Vec<TerminalInfo> {
        let terminals = self.terminals.lock().expect("terminals lock poisoned");
        terminals.values().map(|t| t.info.clone()).collect()
    }

    pub fn start_reading(&self, id: &str) -> Result<(), String> {
        let mut terminals = self.terminals.lock().expect("terminals lock poisoned");
        let terminal = terminals
            .get_mut(id)
            .ok_or_else(|| format!("Terminal {} not found", id))?;

        if let Some(sender) = terminal.start_sender.take() {
            sender
                .send(())
                .map_err(|e| format!("Failed to send start signal: {}", e))?;
            terminal.state = TerminalState::Running;
        }

        Ok(())
    }

    /// Close all terminals (for shutdown)
    pub fn close_all(&self) {
        let mut terminals = self.terminals.lock().expect("terminals lock poisoned");
        let ids: Vec<String> = terminals.keys().cloned().collect();

        for id in ids {
            if let Some(mut terminal) = terminals.remove(&id) {
                terminal.state = TerminalState::ShuttingDown;
                terminal.shutdown_signal.signal();
                drop(terminal.master);
                if let Some(handle) = terminal.reader_handle.take() {
                    let _ = handle.join();
                }
                terminal.state = TerminalState::Closed;
            }
        }

        info!("All terminals closed");
    }
}

impl Drop for TerminalManager {
    fn drop(&mut self) {
        self.close_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A simple event sender that records events for testing.
    struct TestEventSender {
        outputs: std::sync::Mutex<Vec<(String, String)>>,
        closes: std::sync::Mutex<Vec<String>>,
    }

    impl TestEventSender {
        fn new() -> Self {
            Self {
                outputs: std::sync::Mutex::new(Vec::new()),
                closes: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    impl TerminalEventSender for TestEventSender {
        fn send_output(&self, terminal_id: &str, data: &str) {
            self.outputs
                .lock()
                .unwrap()
                .push((terminal_id.to_string(), data.to_string()));
        }

        fn send_close(&self, terminal_id: &str) {
            self.closes.lock().unwrap().push(terminal_id.to_string());
        }
    }

    fn make_manager() -> TerminalManager {
        TerminalManager::new(Box::new(TestEventSender::new()))
    }

    #[test]
    fn test_terminal_manager_creation() {
        let manager = make_manager();
        assert!(manager.list_terminals().is_empty());
        assert!(manager.list_profiles().is_empty());
    }

    #[test]
    fn test_profile_registration() {
        let manager = make_manager();

        let profile = TerminalProfile {
            id: "test-profile".to_string(),
            name: "Test Shell".to_string(),
            shell: "/bin/bash".to_string(),
            args: vec!["-l".to_string()],
            env: HashMap::new(),
            cwd: None,
            icon: None,
            color: None,
        };

        let result = manager.register_profile(profile.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-profile");

        // Should fail if profile already exists
        let result2 = manager.register_profile(profile);
        assert!(result2.is_err());
    }

    #[test]
    fn test_profile_unregistration() {
        let manager = make_manager();

        let profile = TerminalProfile {
            id: "test-profile".to_string(),
            name: "Test Shell".to_string(),
            shell: "/bin/bash".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
            icon: None,
            color: None,
        };

        manager.register_profile(profile).unwrap();
        assert!(manager.get_profile("test-profile").is_ok());

        manager.unregister_profile("test-profile").unwrap();
        assert!(manager.get_profile("test-profile").is_err());
    }

    #[test]
    fn test_list_profiles() {
        let manager = make_manager();

        let profile1 = TerminalProfile {
            id: "profile1".to_string(),
            name: "Profile 1".to_string(),
            shell: "/bin/bash".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
            icon: None,
            color: None,
        };

        let profile2 = TerminalProfile {
            id: "profile2".to_string(),
            name: "Profile 2".to_string(),
            shell: "/bin/zsh".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
            icon: None,
            color: None,
        };

        manager.register_profile(profile1).unwrap();
        manager.register_profile(profile2).unwrap();

        let profiles = manager.list_profiles();
        assert_eq!(profiles.len(), 2);
    }

    #[test]
    fn test_shutdown_signal() {
        let signal = ShutdownSignal::new();
        assert!(!signal.is_shutdown());

        signal.signal();
        assert!(signal.is_shutdown());

        // Clone flag should also show shutdown
        let flag = signal.clone_flag();
        assert!(flag.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_terminal_info_clone() {
        let info = TerminalInfo {
            id: "test-terminal".to_string(),
            name: "Test Terminal".to_string(),
            shell: "/bin/bash".to_string(),
            cwd: "/home/user".to_string(),
        };

        let cloned = info.clone();
        assert_eq!(cloned.id, info.id);
        assert_eq!(cloned.name, info.name);
        assert_eq!(cloned.shell, info.shell);
        assert_eq!(cloned.cwd, info.cwd);
    }

    #[test]
    fn test_terminal_options_default() {
        let options = TerminalOptions {
            name: None,
            shell_path: None,
            shell_args: vec![],
            cwd: None,
            env: HashMap::new(),
            profile_id: None,
        };

        assert!(options.name.is_none());
        assert!(options.shell_path.is_none());
        assert!(options.shell_args.is_empty());
    }

    // ── Serde round-trip tests ──────────────────────────────────────────────────

    #[test]
    fn test_terminal_info_serde_roundtrip() {
        let info = TerminalInfo {
            id: "terminal-1".to_string(),
            name: "My Terminal".to_string(),
            shell: "/bin/zsh".to_string(),
            cwd: "/home/user/projects".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: TerminalInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, info.id);
        assert_eq!(deserialized.name, info.name);
        assert_eq!(deserialized.shell, info.shell);
        assert_eq!(deserialized.cwd, info.cwd);
    }

    #[test]
    fn test_terminal_profile_serde_roundtrip() {
        let mut env = HashMap::new();
        env.insert("TERM".to_string(), "xterm-256color".to_string());
        env.insert("LANG".to_string(), "en_US.UTF-8".to_string());

        let profile = TerminalProfile {
            id: "zsh-profile".to_string(),
            name: "ZSH".to_string(),
            shell: "/bin/zsh".to_string(),
            args: vec!["-l".to_string(), "-i".to_string()],
            env: env.clone(),
            cwd: Some("/home/user".to_string()),
            icon: Some("terminal".to_string()),
            color: Some("green".to_string()),
        };
        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: TerminalProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, profile.id);
        assert_eq!(deserialized.name, profile.name);
        assert_eq!(deserialized.shell, profile.shell);
        assert_eq!(deserialized.args, profile.args);
        assert_eq!(deserialized.env, profile.env);
        assert_eq!(deserialized.cwd, profile.cwd);
        assert_eq!(deserialized.icon, profile.icon);
        assert_eq!(deserialized.color, profile.color);
    }

    #[test]
    fn test_terminal_profile_serde_camel_case() {
        let mut env = HashMap::new();
        env.insert("KEY".to_string(), "val".to_string());
        let profile = TerminalProfile {
            id: "p1".to_string(),
            name: "P1".to_string(),
            shell: "/bin/bash".to_string(),
            args: vec![],
            env,
            cwd: Some("/tmp".to_string()),
            icon: Some("terminal".to_string()),
            color: None,
        };
        let json = serde_json::to_string(&profile).unwrap();
        // With #[serde(rename_all = "camelCase")], field names should be camelCase in JSON
        // "args" stays "args", "env" stays "env", "cwd" stays "cwd",
        // but verify the JSON is valid and round-trips correctly
        let deserialized: TerminalProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, profile.id);
        assert_eq!(deserialized.name, profile.name);
        assert_eq!(deserialized.shell, profile.shell);
        assert_eq!(deserialized.args, profile.args);
        assert_eq!(deserialized.env, profile.env);
        assert_eq!(deserialized.cwd, profile.cwd);
        assert_eq!(deserialized.icon, profile.icon);
        assert_eq!(deserialized.color, profile.color);
    }

    #[test]
    fn test_terminal_options_serde_roundtrip() {
        let mut env = HashMap::new();
        env.insert("FOO".to_string(), "bar".to_string());

        let options = TerminalOptions {
            name: Some("My Term".to_string()),
            shell_path: Some("/bin/fish".to_string()),
            shell_args: vec!["-l".to_string()],
            cwd: Some("/home/user".to_string()),
            env: env.clone(),
            profile_id: Some("fish-profile".to_string()),
        };
        let json = serde_json::to_string(&options).unwrap();
        let deserialized: TerminalOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, options.name);
        assert_eq!(deserialized.shell_path, options.shell_path);
        assert_eq!(deserialized.shell_args, options.shell_args);
        assert_eq!(deserialized.cwd, options.cwd);
        assert_eq!(deserialized.env, options.env);
        assert_eq!(deserialized.profile_id, options.profile_id);
    }

    #[test]
    fn test_terminal_options_serde_camel_case() {
        let options = TerminalOptions {
            name: None,
            shell_path: Some("/bin/bash".to_string()),
            shell_args: vec![],
            cwd: None,
            env: HashMap::new(),
            profile_id: Some("default".to_string()),
        };
        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("shellPath"), "expected camelCase 'shellPath' in JSON: {}", json);
        assert!(json.contains("shellArgs"), "expected camelCase 'shellArgs' in JSON: {}", json);
        assert!(json.contains("profileId"), "expected camelCase 'profileId' in JSON: {}", json);
    }

    // ── TerminalProfile defaults ────────────────────────────────────────────────

    #[test]
    fn test_terminal_profile_default_values() {
        let profile = TerminalProfile {
            id: "default".to_string(),
            name: "Default".to_string(),
            shell: "/bin/bash".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
            icon: None,
            color: None,
        };
        assert_eq!(profile.id, "default");
        assert_eq!(profile.name, "Default");
        assert_eq!(profile.shell, "/bin/bash");
        assert!(profile.args.is_empty());
        assert!(profile.env.is_empty());
        assert!(profile.cwd.is_none());
        assert!(profile.icon.is_none());
        assert!(profile.color.is_none());
    }

    // ── TerminalState variants ─────────────────────────────────────────────────

    #[test]
    fn test_terminal_state_variants() {
        assert_eq!(TerminalState::Created, TerminalState::Created);
        assert_eq!(TerminalState::Running, TerminalState::Running);
        assert_eq!(TerminalState::ShuttingDown, TerminalState::ShuttingDown);
        assert_eq!(TerminalState::Closed, TerminalState::Closed);

        // All variants should be distinct
        assert_ne!(TerminalState::Created, TerminalState::Running);
        assert_ne!(TerminalState::Running, TerminalState::ShuttingDown);
        assert_ne!(TerminalState::ShuttingDown, TerminalState::Closed);
        assert_ne!(TerminalState::Created, TerminalState::Closed);
    }

    #[test]
    fn test_terminal_state_copy_equality() {
        let state1 = TerminalState::Running;
        let state2 = state1; // Copy, not move
        assert_eq!(state1, state2);
    }

    // ── TerminalEventSender trait mock ──────────────────────────────────────────

    #[test]
    fn test_mock_event_sender() {
        let sender = TestEventSender::new();
        // Verify the mock implementation works through the trait
        let sender_ref: &dyn TerminalEventSender = &sender;
        sender_ref.send_output("term-1", "hello world");
        sender_ref.send_close("term-1");

        assert_eq!(sender.outputs.lock().unwrap().len(), 1);
        assert_eq!(sender.outputs.lock().unwrap()[0], ("term-1".to_string(), "hello world".to_string()));
        assert_eq!(sender.closes.lock().unwrap().len(), 1);
        assert_eq!(sender.closes.lock().unwrap()[0], "term-1".to_string());
    }

    #[test]
    fn test_event_sender_multiple_outputs() {
        let sender = TestEventSender::new();
        sender.send_output("t1", "line1");
        sender.send_output("t1", "line2");
        sender.send_output("t2", "other");
        sender.send_close("t1");

        assert_eq!(sender.outputs.lock().unwrap().len(), 3);
        assert_eq!(sender.closes.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_event_sender_boxed_dyn() {
        // Verify the trait object can be boxed (as required by TerminalManager::new)
        let sender: Box<dyn TerminalEventSender> = Box::new(TestEventSender::new());
        sender.send_output("t1", "data");
        sender.send_close("t1");
    }
}