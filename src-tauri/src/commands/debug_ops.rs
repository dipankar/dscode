use crate::debug::{DebugManager, DebugSession, DebugState, Breakpoint, SourceBreakpoint, LaunchRequestArguments};
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn create_debug_session(
    debug_manager: State<Mutex<DebugManager>>,
    name: String,
    adapter_type: String,
) -> Result<String, String> {
    let manager = debug_manager.lock().map_err(|e| e.to_string())?;
    manager.create_session(name, adapter_type)
}

#[tauri::command]
pub fn get_debug_session(
    debug_manager: State<Mutex<DebugManager>>,
    session_id: String,
) -> Result<DebugSession, String> {
    let manager = debug_manager.lock().map_err(|e| e.to_string())?;
    manager.get_session(&session_id)
}

#[tauri::command]
pub fn list_debug_sessions(
    debug_manager: State<Mutex<DebugManager>>,
) -> Result<Vec<DebugSession>, String> {
    let manager = debug_manager.lock().map_err(|e| e.to_string())?;
    manager.list_sessions()
}

#[tauri::command]
pub fn start_debugging(
    debug_manager: State<Mutex<DebugManager>>,
    session_id: String,
    _config: LaunchRequestArguments,
) -> Result<(), String> {
    let manager = debug_manager.lock().map_err(|e| e.to_string())?;
    manager.update_session_state(&session_id, DebugState::Running)?;
    println!("[Debug] Started debugging session: {}", session_id);
    Ok(())
}

#[tauri::command]
pub fn pause_debugging(
    debug_manager: State<Mutex<DebugManager>>,
    session_id: String,
) -> Result<(), String> {
    let manager = debug_manager.lock().map_err(|e| e.to_string())?;
    manager.update_session_state(&session_id, DebugState::Paused)?;
    println!("[Debug] Paused debugging session: {}", session_id);
    Ok(())
}

#[tauri::command]
pub fn continue_debugging(
    debug_manager: State<Mutex<DebugManager>>,
    session_id: String,
) -> Result<(), String> {
    let manager = debug_manager.lock().map_err(|e| e.to_string())?;
    manager.update_session_state(&session_id, DebugState::Running)?;
    println!("[Debug] Continued debugging session: {}", session_id);
    Ok(())
}

#[tauri::command]
pub fn stop_debugging(
    debug_manager: State<Mutex<DebugManager>>,
    session_id: String,
) -> Result<(), String> {
    let manager = debug_manager.lock().map_err(|e| e.to_string())?;
    manager.terminate_session(&session_id)?;
    println!("[Debug] Stopped debugging session: {}", session_id);
    Ok(())
}

#[tauri::command]
pub fn step_over(
    debug_manager: State<Mutex<DebugManager>>,
    session_id: String,
) -> Result<(), String> {
    let manager = debug_manager.lock().map_err(|e| e.to_string())?;
    let _session = manager.get_session(&session_id)?;
    println!("[Debug] Step over in session: {}", session_id);
    // In real implementation, would send step over command to debug adapter
    Ok(())
}

#[tauri::command]
pub fn step_into(
    debug_manager: State<Mutex<DebugManager>>,
    session_id: String,
) -> Result<(), String> {
    let manager = debug_manager.lock().map_err(|e| e.to_string())?;
    let _session = manager.get_session(&session_id)?;
    println!("[Debug] Step into in session: {}", session_id);
    // In real implementation, would send step into command to debug adapter
    Ok(())
}

#[tauri::command]
pub fn step_out(
    debug_manager: State<Mutex<DebugManager>>,
    session_id: String,
) -> Result<(), String> {
    let manager = debug_manager.lock().map_err(|e| e.to_string())?;
    let _session = manager.get_session(&session_id)?;
    println!("[Debug] Step out in session: {}", session_id);
    // In real implementation, would send step out command to debug adapter
    Ok(())
}

#[tauri::command]
pub fn set_breakpoints(
    debug_manager: State<Mutex<DebugManager>>,
    file_path: String,
    breakpoints: Vec<SourceBreakpoint>,
) -> Result<(), String> {
    let manager = debug_manager.lock().map_err(|e| e.to_string())?;
    manager.set_breakpoints(file_path, breakpoints)
}

#[tauri::command]
pub fn get_breakpoints(
    debug_manager: State<Mutex<DebugManager>>,
    file_path: String,
) -> Result<Vec<Breakpoint>, String> {
    let manager = debug_manager.lock().map_err(|e| e.to_string())?;
    manager.get_breakpoints(&file_path)
}

#[tauri::command]
pub fn clear_breakpoints(
    debug_manager: State<Mutex<DebugManager>>,
    file_path: String,
) -> Result<(), String> {
    let manager = debug_manager.lock().map_err(|e| e.to_string())?;
    manager.clear_breakpoints(&file_path)
}

#[tauri::command]
pub fn get_all_breakpoints(
    debug_manager: State<Mutex<DebugManager>>,
) -> Result<std::collections::HashMap<String, Vec<Breakpoint>>, String> {
    let manager = debug_manager.lock().map_err(|e| e.to_string())?;
    manager.get_all_breakpoints()
}
