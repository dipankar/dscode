use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use tauri::{AppHandle, Emitter};

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

/// Shutdown signal for terminal reader thread
struct ShutdownSignal {
    flag: Arc<AtomicBool>,
}

impl ShutdownSignal {
    fn new() -> Self {
        Self { flag: Arc::new(AtomicBool::new(false)) }
    }

    fn signal(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    fn is_shutdown(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    fn clone_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.flag)
    }
}

pub struct TerminalInstance {
    pub info: TerminalInfo,
    writer: Box<dyn Write + Send>,
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    start_sender: Option<Sender<()>>,
    shutdown_signal: ShutdownSignal,
    reader_handle: Option<JoinHandle<()>>,
}

pub struct TerminalManager {
    terminals: Arc<Mutex<HashMap<String, TerminalInstance>>>,
    profiles: Arc<Mutex<HashMap<String, TerminalProfile>>>,
    next_id: Arc<Mutex<u32>>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            terminals: Arc::new(Mutex::new(HashMap::new())),
            profiles: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    // ===== Profile Management =====

    pub fn register_profile(&self, profile: TerminalProfile) -> Result<String, String> {
        let mut profiles = self.profiles.lock().expect("profiles lock poisoned");
        let profile_id = profile.id.clone();

        if profiles.contains_key(&profile_id) {
            return Err(format!("Profile '{}' already exists", profile_id));
        }

        profiles.insert(profile_id.clone(), profile);
        Ok(profile_id)
    }

    pub fn unregister_profile(&self, profile_id: &str) -> Result<(), String> {
        let mut profiles = self.profiles.lock().expect("profiles lock poisoned");
        profiles.remove(profile_id).ok_or_else(|| format!("Profile '{}' not found", profile_id))?;
        Ok(())
    }

    pub fn get_profile(&self, profile_id: &str) -> Result<TerminalProfile, String> {
        let profiles = self.profiles.lock().expect("profiles lock poisoned");
        profiles
            .get(profile_id)
            .cloned()
            .ok_or_else(|| format!("Profile '{}' not found", profile_id))
    }

    pub fn list_profiles(&self) -> Vec<TerminalProfile> {
        let profiles = self.profiles.lock().expect("profiles lock poisoned");
        profiles.values().cloned().collect()
    }

    // ===== Terminal Creation =====

    pub fn create_terminal_with_options(
        &self, options: TerminalOptions, app_handle: AppHandle,
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
            let mut next = self.next_id.lock().expect("next_id lock poisoned");
            let current = *next;
            *next += 1;
            format!("terminal-{}", current)
        };

        // Create PTY system and open pair with initial size
        let pair = native_pty_system()
            .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
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
        let _child =
            slave.spawn_command(cmd).map_err(|e| format!("Failed to spawn shell: {}", e))?;

        // Prepare IO handles
        let mut reader =
            master.try_clone_reader().map_err(|e| format!("Failed to clone reader: {}", e))?;
        let writer = master.take_writer().map_err(|e| format!("Failed to get writer: {}", e))?;
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

        // Start reading from PTY in a background thread
        let terminal_id = id.clone();
        let app = app_handle.clone();
        let reader_handle = thread::spawn(move || {
            // Wait for ready signal from frontend
            if start_rx.recv().is_err() {
                eprintln!("[Terminal] Failed to receive start signal for {}", terminal_id);
                return;
            }

            let mut buf = [0u8; 8192];
            loop {
                // Check shutdown signal
                if shutdown_flag.load(Ordering::SeqCst) {
                    println!("[Terminal] Shutdown signal received for {}", terminal_id);
                    break;
                }

                match reader.read(&mut buf) {
                    Ok(0) => {
                        // EOF - terminal closed
                        println!("[Terminal] Terminal {} closed (EOF)", terminal_id);
                        let _ = app.emit(&format!("terminal-closed:{}", terminal_id), ());
                        break;
                    }
                    Ok(n) => {
                        // Send data to frontend
                        let data = String::from_utf8_lossy(&buf[0..n]).to_string();
                        let _ = app.emit(
                            &format!("terminal-data:{}", terminal_id),
                            serde_json::json!({ "data": data }),
                        );
                    }
                    Err(e) => {
                        // Check if this is a normal shutdown
                        if shutdown_flag.load(Ordering::SeqCst) {
                            println!("[Terminal] Terminal {} shutdown complete", terminal_id);
                        } else {
                            eprintln!("[Terminal] Error reading from PTY: {}", e);
                        }
                        break;
                    }
                }
            }
        });

        // Store terminal instance
        let terminal_instance = TerminalInstance {
            info: info.clone(),
            writer,
            master: Arc::clone(&master),
            start_sender: Some(start_tx),
            shutdown_signal,
            reader_handle: Some(reader_handle),
        };

        {
            let mut terminals = self.terminals.lock().expect("terminals lock poisoned");
            terminals.insert(id.clone(), terminal_instance);
        }

        Ok(id)
    }

    pub fn create_terminal(
        &self, name: Option<String>, shell: Option<String>, cwd: Option<String>,
        app_handle: AppHandle,
    ) -> Result<String, String> {
        // Get next terminal ID
        let id = {
            let mut next = self.next_id.lock().expect("next_id lock poisoned");
            let current = *next;
            *next += 1;
            format!("terminal-{}", current)
        };

        // Determine shell to use
        let shell_cmd = shell.unwrap_or_else(|| {
            std::env::var("SHELL").unwrap_or_else(|_| {
                if cfg!(target_os = "windows") {
                    "powershell.exe".to_string()
                } else {
                    "/bin/bash".to_string()
                }
            })
        });

        // Determine working directory
        let working_dir = cwd.unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "/".to_string())
        });

        // Create PTY system and open pair with initial size
        let pair = native_pty_system()
            .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| format!("Failed to create PTY: {}", e))?;

        let portable_pty::PtyPair { master, slave } = pair;

        // Create command
        let mut cmd = CommandBuilder::new(&shell_cmd);
        cmd.cwd(&working_dir);

        // Spawn the shell
        let _child =
            slave.spawn_command(cmd).map_err(|e| format!("Failed to spawn shell: {}", e))?;

        // Prepare IO handles
        let mut reader =
            master.try_clone_reader().map_err(|e| format!("Failed to clone reader: {}", e))?;
        let writer = master.take_writer().map_err(|e| format!("Failed to get writer: {}", e))?;
        let master = Arc::new(Mutex::new(master));

        // Create terminal info
        let terminal_name = name.unwrap_or_else(|| format!("Terminal {}", id));
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

        // Start reading from PTY in a background thread
        let terminal_id = id.clone();
        let app = app_handle.clone();
        let reader_handle = thread::spawn(move || {
            // Wait for ready signal from frontend
            if start_rx.recv().is_err() {
                eprintln!("[Terminal] Failed to receive start signal for {}", terminal_id);
                return;
            }

            let mut buf = [0u8; 8192];
            loop {
                // Check shutdown signal
                if shutdown_flag.load(Ordering::SeqCst) {
                    println!("[Terminal] Shutdown signal received for {}", terminal_id);
                    break;
                }

                match reader.read(&mut buf) {
                    Ok(0) => {
                        // EOF - terminal closed
                        println!("[Terminal] Terminal {} closed (EOF)", terminal_id);
                        let _ = app.emit(&format!("terminal-closed:{}", terminal_id), ());
                        break;
                    }
                    Ok(n) => {
                        // Send data to frontend
                        let data = String::from_utf8_lossy(&buf[0..n]).to_string();
                        let _ = app.emit(
                            &format!("terminal-data:{}", terminal_id),
                            serde_json::json!({ "data": data }),
                        );
                    }
                    Err(e) => {
                        // Check if this is a normal shutdown
                        if shutdown_flag.load(Ordering::SeqCst) {
                            println!("[Terminal] Terminal {} shutdown complete", terminal_id);
                        } else {
                            eprintln!("[Terminal] Error reading from PTY: {}", e);
                        }
                        break;
                    }
                }
            }
        });

        // Store terminal instance
        let terminal_instance = TerminalInstance {
            info: info.clone(),
            writer,
            master: Arc::clone(&master),
            start_sender: Some(start_tx),
            shutdown_signal,
            reader_handle: Some(reader_handle),
        };

        {
            let mut terminals = self.terminals.lock().expect("terminals lock poisoned");
            terminals.insert(id.clone(), terminal_instance);
        }

        Ok(id)
    }

    pub fn write_to_terminal(&self, id: &str, data: &str) -> Result<(), String> {
        let mut terminals = self.terminals.lock().expect("terminals lock poisoned");
        let terminal = terminals.get_mut(id).ok_or_else(|| format!("Terminal {} not found", id))?;

        terminal
            .writer
            .write_all(data.as_bytes())
            .map_err(|e| format!("Failed to write to terminal: {}", e))?;

        terminal.writer.flush().map_err(|e| format!("Failed to flush terminal: {}", e))?;

        Ok(())
    }

    pub fn resize_terminal(&self, id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let mut terminals = self.terminals.lock().expect("terminals lock poisoned");
        let terminal = terminals.get_mut(id).ok_or_else(|| format!("Terminal {} not found", id))?;

        let master = terminal.master.lock().expect("pty master lock poisoned");
        master
            .resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| format!("Failed to resize terminal: {}", e))
    }

    pub fn close_terminal(&self, id: &str) -> Result<(), String> {
        let mut terminals = self.terminals.lock().expect("terminals lock poisoned");
        if let Some(mut terminal) = terminals.remove(id) {
            // Signal the reader thread to stop
            terminal.shutdown_signal.signal();

            // Drop the master to close the PTY (this will cause read to return EOF/error)
            drop(terminal.master);

            // Wait for the reader thread to finish (with timeout)
            if let Some(handle) = terminal.reader_handle.take() {
                // Don't block indefinitely - the drop of master should cause the read to return
                let _ = handle.join();
            }

            println!("[Terminal] Terminal {} closed and cleaned up", id);
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
        let terminal = terminals.get_mut(id).ok_or_else(|| format!("Terminal {} not found", id))?;

        if let Some(sender) = terminal.start_sender.take() {
            sender.send(()).map_err(|e| format!("Failed to send start signal: {}", e))?;
        }

        Ok(())
    }

    /// Close all terminals (for shutdown)
    pub fn close_all(&self) {
        let mut terminals = self.terminals.lock().expect("terminals lock poisoned");
        let ids: Vec<String> = terminals.keys().cloned().collect();

        for id in ids {
            if let Some(mut terminal) = terminals.remove(&id) {
                terminal.shutdown_signal.signal();
                drop(terminal.master);
                if let Some(handle) = terminal.reader_handle.take() {
                    let _ = handle.join();
                }
            }
        }

        println!("[Terminal] All terminals closed");
    }
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
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

    #[test]
    fn test_terminal_manager_creation() {
        let manager = TerminalManager::new();
        assert!(manager.list_terminals().is_empty());
        assert!(manager.list_profiles().is_empty());
    }

    #[test]
    fn test_terminal_manager_default() {
        let manager = TerminalManager::default();
        assert!(manager.list_terminals().is_empty());
    }

    #[test]
    fn test_profile_registration() {
        let manager = TerminalManager::new();

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
        let manager = TerminalManager::new();

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
        let manager = TerminalManager::new();

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
}
