use crate::terminal::{TerminalInfo, TerminalManager};
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
