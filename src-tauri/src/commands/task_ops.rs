use super::task_registry::*;
use tauri::State;

// ===== Task Provider Commands =====

/// Register a task provider
#[tauri::command]
pub async fn register_task_provider(
    provider: TaskProvider,
    registry: State<'_, TaskRegistry>,
) -> Result<String, String> {
    registry.register_task_provider(provider)
}

/// Unregister a task provider
#[tauri::command]
pub async fn unregister_task_provider(
    provider_id: String,
    registry: State<'_, TaskRegistry>,
) -> Result<(), String> {
    registry.unregister_task_provider(&provider_id)
}

/// Get all task providers
#[tauri::command]
pub async fn get_task_providers(
    registry: State<'_, TaskRegistry>,
) -> Result<Vec<TaskProvider>, String> {
    Ok(registry.get_task_providers())
}

/// Get task providers by type
#[tauri::command]
pub async fn get_task_providers_by_type(
    task_type: String,
    registry: State<'_, TaskRegistry>,
) -> Result<Vec<TaskProvider>, String> {
    Ok(registry.get_task_providers_by_type(&task_type))
}

/// Get task provider by ID
#[tauri::command]
pub async fn get_task_provider(
    provider_id: String,
    registry: State<'_, TaskRegistry>,
) -> Result<TaskProvider, String> {
    registry.get_task_provider(&provider_id)
}

// ===== Task Execution Commands =====

/// Start a task execution
#[tauri::command]
pub async fn start_task_execution(
    execution: TaskExecution,
    registry: State<'_, TaskRegistry>,
) -> Result<String, String> {
    registry.start_task_execution(execution)
}

/// Update task execution
#[tauri::command]
pub async fn update_task_execution(
    execution_id: String,
    state: TaskExecutionState,
    exit_code: Option<i32>,
    registry: State<'_, TaskRegistry>,
) -> Result<(), String> {
    registry.update_task_execution(&execution_id, state, exit_code)
}

/// End task execution
#[tauri::command]
pub async fn end_task_execution(
    execution_id: String,
    exit_code: i32,
    registry: State<'_, TaskRegistry>,
) -> Result<(), String> {
    registry.end_task_execution(&execution_id, exit_code)
}

/// Get task execution
#[tauri::command]
pub async fn get_task_execution(
    execution_id: String,
    registry: State<'_, TaskRegistry>,
) -> Result<TaskExecution, String> {
    registry.get_task_execution(&execution_id)
}

/// Get all task executions
#[tauri::command]
pub async fn get_all_task_executions(
    registry: State<'_, TaskRegistry>,
) -> Result<Vec<TaskExecution>, String> {
    Ok(registry.get_all_task_executions())
}

/// Clear task executions
#[tauri::command]
pub async fn clear_task_executions(
    registry: State<'_, TaskRegistry>,
) -> Result<(), String> {
    registry.clear_task_executions();
    Ok(())
}

// ===== Cleanup Commands =====

/// Clear task data for owner
#[tauri::command]
pub async fn clear_task_data(
    owner: String,
    registry: State<'_, TaskRegistry>,
) -> Result<(), String> {
    registry.clear_task_data(&owner);
    Ok(())
}
