use crate::terminal::{TerminalInfo, TerminalManager, TerminalProfile, TerminalOptions};
use std::sync::Mutex;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn create_terminal(
    name: Option<String>,
    shell: Option<String>,
    cwd: Option<String>,
    app_handle: AppHandle,
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<String, String> {
    let manager = terminal_manager.lock().unwrap();
    manager.create_terminal(name, shell, cwd, app_handle)
}

#[tauri::command]
pub fn write_to_terminal(
    terminal_id: String,
    data: String,
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<(), String> {
    let manager = terminal_manager.lock().unwrap();
    manager.write_to_terminal(&terminal_id, &data)
}

#[tauri::command]
pub fn resize_terminal(
    terminal_id: String,
    cols: u16,
    rows: u16,
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<(), String> {
    let manager = terminal_manager.lock().unwrap();
    manager.resize_terminal(&terminal_id, cols, rows)
}

#[tauri::command]
pub fn close_terminal(
    terminal_id: String,
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<(), String> {
    let manager = terminal_manager.lock().unwrap();
    manager.close_terminal(&terminal_id)
}

#[tauri::command]
pub fn list_terminals(
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<Vec<TerminalInfo>, String> {
    let manager = terminal_manager.lock().unwrap();
    Ok(manager.list_terminals())
}

#[tauri::command]
pub fn terminal_ready(
    terminal_id: String,
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<(), String> {
    let manager = terminal_manager.lock().unwrap();
    manager.start_reading(&terminal_id)
}

// ===== Enhanced Terminal Commands =====

#[tauri::command]
pub fn create_terminal_with_options(
    options: TerminalOptions,
    app_handle: AppHandle,
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<String, String> {
    let manager = terminal_manager.lock().unwrap();
    manager.create_terminal_with_options(options, app_handle)
}

// ===== Profile Management Commands =====

#[tauri::command]
pub fn register_terminal_profile(
    profile: TerminalProfile,
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<String, String> {
    let manager = terminal_manager.lock().unwrap();
    manager.register_profile(profile)
}

#[tauri::command]
pub fn unregister_terminal_profile(
    profile_id: String,
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<(), String> {
    let manager = terminal_manager.lock().unwrap();
    manager.unregister_profile(&profile_id)
}

#[tauri::command]
pub fn get_terminal_profile(
    profile_id: String,
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<TerminalProfile, String> {
    let manager = terminal_manager.lock().unwrap();
    manager.get_profile(&profile_id)
}

#[tauri::command]
pub fn list_terminal_profiles(
    terminal_manager: State<Mutex<TerminalManager>>,
) -> Result<Vec<TerminalProfile>, String> {
    let manager = terminal_manager.lock().unwrap();
    Ok(manager.list_profiles())
}
