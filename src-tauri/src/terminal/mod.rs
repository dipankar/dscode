use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender};
use std::thread;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, serde::Serialize)]
pub struct TerminalInfo {
    pub id: String,
    pub name: String,
    pub shell: String,
    pub cwd: String,
}

pub struct TerminalInstance {
    pub info: TerminalInfo,
    writer: Box<dyn Write + Send>,
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    start_sender: Option<Sender<()>>,
}

pub struct TerminalManager {
    terminals: Arc<Mutex<HashMap<String, TerminalInstance>>>,
    next_id: Arc<Mutex<u32>>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            terminals: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    pub fn create_terminal(
        &self,
        name: Option<String>,
        shell: Option<String>,
        cwd: Option<String>,
        app_handle: AppHandle,
    ) -> Result<String, String> {
        // Get next terminal ID
        let id = {
            let mut next = self.next_id.lock().unwrap();
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
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to create PTY: {}", e))?;

        let portable_pty::PtyPair { mut master, slave } = pair;

        // Create command
        let mut cmd = CommandBuilder::new(&shell_cmd);
        cmd.cwd(&working_dir);

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
        let terminal_name = name.unwrap_or_else(|| format!("Terminal {}", id));
        let info = TerminalInfo {
            id: id.clone(),
            name: terminal_name,
            shell: shell_cmd,
            cwd: working_dir,
        };

        // Create channel for start signal
        let (start_tx, start_rx) = mpsc::channel();

        // Store terminal instance
        let terminal_instance = TerminalInstance {
            info: info.clone(),
            writer,
            master: Arc::clone(&master),
            start_sender: Some(start_tx),
        };

        {
            let mut terminals = self.terminals.lock().unwrap();
            terminals.insert(id.clone(), terminal_instance);
        }

        // Start reading from PTY in a background thread
        let terminal_id = id.clone();
        let app = app_handle.clone();
        thread::spawn(move || {
            // Wait for ready signal from frontend
            if start_rx.recv().is_err() {
                eprintln!("[Terminal] Failed to receive start signal for {}", terminal_id);
                return;
            }

            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        // EOF - terminal closed
                        println!("[Terminal] Terminal {} closed", terminal_id);
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
                        eprintln!("[Terminal] Error reading from PTY: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(id)
    }

    pub fn write_to_terminal(&self, id: &str, data: &str) -> Result<(), String> {
        let mut terminals = self.terminals.lock().unwrap();
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
        let mut terminals = self.terminals.lock().unwrap();
        let terminal = terminals
            .get_mut(id)
            .ok_or_else(|| format!("Terminal {} not found", id))?;

        let mut master = terminal.master.lock().unwrap();
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to resize terminal: {}", e))
    }

    pub fn close_terminal(&self, id: &str) -> Result<(), String> {
        let mut terminals = self.terminals.lock().unwrap();
        terminals
            .remove(id)
            .ok_or_else(|| format!("Terminal {} not found", id))?;
        Ok(())
    }

    pub fn list_terminals(&self) -> Vec<TerminalInfo> {
        let terminals = self.terminals.lock().unwrap();
        terminals.values().map(|t| t.info.clone()).collect()
    }

    pub fn start_reading(&self, id: &str) -> Result<(), String> {
        let mut terminals = self.terminals.lock().unwrap();
        let terminal = terminals
            .get_mut(id)
            .ok_or_else(|| format!("Terminal {} not found", id))?;

        if let Some(sender) = terminal.start_sender.take() {
            sender.send(()).map_err(|e| format!("Failed to send start signal: {}", e))?;
        }

        Ok(())
    }
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}
