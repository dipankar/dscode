use dscode_dap::DapError;
use dscode_dap::DebugAdapterPool;
use dscode_dap::{
    Breakpoint, DebugManager, DebugSession, DebugState, LaunchRequestArguments, SourceBreakpoint,
};
use std::sync::Mutex;
use tauri::State;
use tokio::sync::RwLock;

#[tauri::command]
pub async fn create_debug_session(
    debug_manager: State<'_, Mutex<DebugManager>>, debug_pool: State<'_, RwLock<DebugAdapterPool>>,
    name: String, adapter_type: String,
) -> Result<String, String> {
    let session_id = {
        let manager = debug_manager.lock().map_err(|e| {
            DapError::Io(format!("Failed to acquire debug manager lock: {}", e))
        })?;
        manager.create_session(name, adapter_type).map_err(|e| e.to_string())?
    };

    let session = {
        let manager = debug_manager.lock().map_err(|e| {
            DapError::Io(format!("Failed to acquire debug manager lock: {}", e))
        })?;
        manager.get_session(&session_id).map_err(|e| e.to_string())?
    };

    let pool = debug_pool.read().await;
    let adapter = pool.create_session(session).await.map_err(|e| e.to_string())?;

    adapter.initialize().await.map_err(|e| DapError::Protocol(format!("Failed to initialize debug adapter: {}", e)).to_string())?;

    {
        let manager = debug_manager.lock().map_err(|e| {
            DapError::Io(format!("Failed to acquire debug manager lock: {}", e))
        })?;
        manager.update_session_state(&session_id, DebugState::Initialized).map_err(|e| e.to_string())?;
    }

    Ok(session_id)
}

#[tauri::command]
pub fn get_debug_session(
    debug_manager: State<Mutex<DebugManager>>, session_id: String,
) -> Result<DebugSession, String> {
    let manager = debug_manager.lock().map_err(|e| {
        DapError::Io(format!("Failed to acquire debug manager lock: {}", e))
    })?;
    manager.get_session(&session_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_debug_sessions(
    debug_manager: State<Mutex<DebugManager>>,
) -> Result<Vec<DebugSession>, String> {
    let manager = debug_manager.lock().map_err(|e| {
        DapError::Io(format!("Failed to acquire debug manager lock: {}", e))
    })?;
    manager.list_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_debugging(
    debug_manager: State<'_, Mutex<DebugManager>>, debug_pool: State<'_, RwLock<DebugAdapterPool>>,
    session_id: String, config: LaunchRequestArguments,
) -> Result<(), String> {
    let pool = debug_pool.read().await;

    if let Some(adapter) = pool.get_adapter(&session_id).await {
        adapter.launch(serde_json::to_value(config).map_err(|e| DapError::Protocol(e.to_string()).to_string())?).await.map_err(|e| e.to_string())?;
    }

    {
        let manager = debug_manager.lock().map_err(|e| {
            DapError::Io(format!("Failed to acquire debug manager lock: {}", e))
        })?;
        manager.update_session_state(&session_id, DebugState::Running).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn pause_debugging(
    debug_manager: State<'_, Mutex<DebugManager>>, debug_pool: State<'_, RwLock<DebugAdapterPool>>,
    session_id: String,
) -> Result<(), String> {
    let pool = debug_pool.read().await;

    if let Some(adapter) = pool.get_adapter(&session_id).await {
        let thread_id = 1;
        adapter.pause(thread_id).await.map_err(|e| e.to_string())?;
    }

    {
        let manager = debug_manager.lock().map_err(|e| {
            DapError::Io(format!("Failed to acquire debug manager lock: {}", e))
        })?;
        manager.update_session_state(&session_id, DebugState::Paused).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn continue_debugging(
    debug_manager: State<'_, Mutex<DebugManager>>, debug_pool: State<'_, RwLock<DebugAdapterPool>>,
    session_id: String,
) -> Result<(), String> {
    let pool = debug_pool.read().await;

    if let Some(adapter) = pool.get_adapter(&session_id).await {
        let thread_id = 1;
        adapter.continue_execution(thread_id).await.map_err(|e| e.to_string())?;
    }

    {
        let manager = debug_manager.lock().map_err(|e| {
            DapError::Io(format!("Failed to acquire debug manager lock: {}", e))
        })?;
        manager.update_session_state(&session_id, DebugState::Running).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_debugging(
    debug_manager: State<'_, Mutex<DebugManager>>, debug_pool: State<'_, RwLock<DebugAdapterPool>>,
    session_id: String,
) -> Result<(), String> {
    {
        let pool = debug_pool.read().await;
        if let Some(adapter) = pool.get_adapter(&session_id).await {
            let _ = adapter.disconnect().await;
        }
    }

    {
        let pool = debug_pool.write().await;
        pool.stop_session(&session_id).await.map_err(|e| e.to_string())?;
    }

    {
        let manager = debug_manager.lock().map_err(|e| {
            DapError::Io(format!("Failed to acquire debug manager lock: {}", e))
        })?;
        manager.terminate_session(&session_id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn step_over(
    debug_manager: State<'_, Mutex<DebugManager>>, debug_pool: State<'_, RwLock<DebugAdapterPool>>,
    session_id: String,
) -> Result<(), String> {
    let pool = debug_pool.read().await;

    if let Some(adapter) = pool.get_adapter(&session_id).await {
        let thread_id = 1;
        adapter.next(thread_id).await.map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn step_into(
    debug_manager: State<'_, Mutex<DebugManager>>, debug_pool: State<'_, RwLock<DebugAdapterPool>>,
    session_id: String,
) -> Result<(), String> {
    let pool = debug_pool.read().await;

    if let Some(adapter) = pool.get_adapter(&session_id).await {
        let thread_id = 1;
        adapter.step_in(thread_id).await.map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn step_out(
    debug_manager: State<'_, Mutex<DebugManager>>, debug_pool: State<'_, RwLock<DebugAdapterPool>>,
    session_id: String,
) -> Result<(), String> {
    let pool = debug_pool.read().await;

    if let Some(adapter) = pool.get_adapter(&session_id).await {
        let thread_id = 1;
        adapter.step_out(thread_id).await.map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn set_breakpoints(
    debug_manager: State<Mutex<DebugManager>>, file_path: String,
    breakpoints: Vec<SourceBreakpoint>,
) -> Result<(), String> {
    let manager = debug_manager.lock().map_err(|e| {
        DapError::Io(format!("Failed to acquire debug manager lock: {}", e))
    })?;
    manager.set_breakpoints(file_path, breakpoints).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_breakpoints(
    debug_manager: State<Mutex<DebugManager>>, file_path: String,
) -> Result<Vec<Breakpoint>, String> {
    let manager = debug_manager.lock().map_err(|e| {
        DapError::Io(format!("Failed to acquire debug manager lock: {}", e))
    })?;
    manager.get_breakpoints(&file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_breakpoints(
    debug_manager: State<Mutex<DebugManager>>, file_path: String,
) -> Result<(), String> {
    let manager = debug_manager.lock().map_err(|e| {
        DapError::Io(format!("Failed to acquire debug manager lock: {}", e))
    })?;
    manager.clear_breakpoints(&file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_breakpoints(
    debug_manager: State<Mutex<DebugManager>>,
) -> Result<std::collections::HashMap<String, Vec<Breakpoint>>, String> {
    let manager = debug_manager.lock().map_err(|e| {
        DapError::Io(format!("Failed to acquire debug manager lock: {}", e))
    })?;
    manager.get_all_breakpoints().map_err(|e| e.to_string())
}