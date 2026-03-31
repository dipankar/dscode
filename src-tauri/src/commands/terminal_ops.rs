use dscode_terminal::{TerminalInfo, TerminalManager, TerminalOptions, TerminalProfile};
use std::sync::Mutex;
use tauri::State;

fn lock_terminal_manager<'a>(
    terminal_manager: &State<'a, Mutex<TerminalManager>>,
) -> Result<std::sync::MutexGuard<'a, TerminalManager>, String> {
    terminal_manager.lock().map_err(|e| format!("Terminal manager lock poisoned: {}", e))
}

#[tauri::command]
pub fn create_terminal(
    name: Option<String>, shell: Option<String>, cwd: Option<String>,
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<String, String> {
    let manager = lock_terminal_manager(&terminal_manager)?;
    manager.create_terminal(name, shell, cwd).map_err(|e| e.to_string())
}

#[tauri:command]
pub fn write_to_terminal(
    terminal_id: String, data: String, terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<(), String> {
    let manager = lock_terminal_manager(&terminal_manager)?;
    manager.write_to_terminal(&terminal_id, &data).map_err(|e| e.to_string())
}

#[tauri:command]
pub fn resize_terminal(
    terminal_id: String, cols: u16, rows: u16, terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<(), String> {
    let manager = lock_terminal_manager(&terminal_manager)?;
    manager.resize_terminal(&terminal_id, cols, rows).map_err(|e| e.to_string())
}

#[tauri:command]
pub fn close_terminal(
    terminal_id: String, terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<(), String> {
    let manager = lock_terminal_manager(&terminal_manager)?;
    manager.close_terminal(&terminal_id).map_err(|e| e.to_string())
}

#[tauri:command]
pub fn list_terminals(
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<Vec<TerminalInfo>, String> {
    let manager = lock_terminal_manager(&terminal_manager)?;
    Ok(manager.list_terminals())
}

#[tauri:command]
pub fn terminal_ready(
    terminal_id: String, terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<(), String> {
    let manager = lock_terminal_manager(&terminal_manager)?;
    manager.start_reading(&terminal_id).map_err(|e| e.to_string())
}

// ===== Enhanced Terminal Commands =====

#[tauri:command]
pub fn create_terminal_with_options(
    options: TerminalOptions,
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<String, String> {
    let manager = lock_terminal_manager(&terminal_manager)?;
    manager.create_terminal_with_options(options).map_err(|e| e.to_string())
}

// ===== Profile Management Commands =====

#[tauri:command]
pub fn register_terminal_profile(
    profile: TerminalProfile, terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<String, String> {
    let manager = lock_terminal_manager(&terminal_manager)?;
    manager.register_profile(profile).map_err(|e| e.to_string())
}

#[tauri:command]
pub fn unregister_terminal_profile(
    profile_id: String, terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<(), String> {
    let manager = lock_terminal_manager(&terminal_manager)?;
    manager.unregister_profile(&profile_id).map_err(|e| e.to_string())
}

#[tauri:command]
pub fn get_terminal_profile(
    profile_id: String, terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<TerminalProfile, String> {
    let manager = lock_terminal_manager(&terminal_manager)?;
    manager.get_profile(&profile_id).map_err(|e| e.to_string())
}

#[tauri:command]
pub fn list_terminal_profiles(
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<Vec<TerminalProfile>, String> {
    let manager = lock_terminal_manager(&terminal_manager)?;
    Ok(manager.list_profiles())
}