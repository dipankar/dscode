use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tracing::{error, info};

/// Task registry for managing task providers and definitions
pub struct TaskRegistry {
    providers: Arc<RwLock<Vec<TaskProvider>>>,
    task_executions: Arc<RwLock<HashMap<String, TaskExecution>>>,
    app_handle: AppHandle,
}

impl TaskRegistry {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            providers: Arc::new(RwLock::new(Vec::new())),
            task_executions: Arc::new(RwLock::new(HashMap::new())),
            app_handle,
        }
    }

    // ===== Task Provider Management =====

    /// Register a task provider
    pub fn register_task_provider(&self, provider: TaskProvider) -> Result<String, String> {
        let provider_id = provider.id.clone();
        let mut providers = self.providers.blocking_write();

        // Check if provider already exists
        if providers.iter().any(|p| p.id == provider_id) {
            return Err(format!("Task provider '{}' already registered", provider_id));
        }

        providers.push(provider.clone());

        info!("[Task] Registered task provider: {} (type: {})", provider.id, provider.task_type);

        Ok(provider_id)
    }

    /// Unregister a task provider
    pub fn unregister_task_provider(&self, provider_id: &str) -> Result<(), String> {
        let mut providers = self.providers.blocking_write();

        if let Some(pos) = providers.iter().position(|p| p.id == provider_id) {
            providers.remove(pos);
            info!("[Task] Unregistered task provider: {}", provider_id);
            Ok(())
        } else {
            Err(format!("Task provider '{}' not found", provider_id))
        }
    }

    /// Get all task providers
    pub fn get_task_providers(&self) -> Vec<TaskProvider> {
        self.providers.blocking_read().clone()
    }

    /// Get task providers by type
    pub fn get_task_providers_by_type(&self, task_type: &str) -> Vec<TaskProvider> {
        self.providers
            .blocking_read()
            .iter()
            .filter(|p| p.task_type == task_type)
            .cloned()
            .collect()
    }

    /// Get task provider by ID
    pub fn get_task_provider(&self, provider_id: &str) -> Result<TaskProvider, String> {
        self.providers
            .blocking_read()
            .iter()
            .find(|p| p.id == provider_id)
            .cloned()
            .ok_or_else(|| format!("Task provider '{}' not found", provider_id))
    }

    // ===== Task Execution Management =====

    /// Start a task execution
    pub fn start_task_execution(&self, execution: TaskExecution) -> Result<String, String> {
        let execution_id = execution.id.clone();
        let mut executions = self.task_executions.blocking_write();

        executions.insert(execution_id.clone(), execution.clone());

        // Emit task started event
        if let Err(e) = self.app_handle.emit("task-started", &execution) {
            error!("[Task] Failed to emit task started event: {}", e);
        }

        info!("[Task] Started task execution: {}", execution_id);

        Ok(execution_id)
    }

    /// Update task execution
    pub fn update_task_execution(
        &self, execution_id: &str, state: TaskExecutionState, exit_code: Option<i32>,
    ) -> Result<(), String> {
        let mut executions = self.task_executions.blocking_write();

        if let Some(execution) = executions.get_mut(execution_id) {
            execution.state = state.clone();
            execution.exit_code = exit_code;

            // Emit task updated event
            if let Err(e) = self.app_handle.emit("task-updated", execution.clone()) {
                error!("[Task] Failed to emit task updated event: {}", e);
            }

            info!("[Task] Updated task execution: {} (state: {:?})", execution_id, state);

            Ok(())
        } else {
            Err(format!("Task execution '{}' not found", execution_id))
        }
    }

    /// End task execution
    pub fn end_task_execution(&self, execution_id: &str, exit_code: i32) -> Result<(), String> {
        let mut executions = self.task_executions.blocking_write();

        if let Some(execution) = executions.get_mut(execution_id) {
            execution.state = if exit_code == 0 {
                TaskExecutionState::Completed
            } else {
                TaskExecutionState::Failed
            };
            execution.exit_code = Some(exit_code);

            // Emit task ended event
            if let Err(e) = self.app_handle.emit("task-ended", execution.clone()) {
                error!("[Task] Failed to emit task ended event: {}", e);
            }

            info!("[Task] Ended task execution: {} (exit code: {})", execution_id, exit_code);

            Ok(())
        } else {
            Err(format!("Task execution '{}' not found", execution_id))
        }
    }

    /// Get task execution
    pub fn get_task_execution(&self, execution_id: &str) -> Result<TaskExecution, String> {
        self.task_executions
            .blocking_read()
            .get(execution_id)
            .cloned()
            .ok_or_else(|| format!("Task execution '{}' not found", execution_id))
    }

    /// Get all task executions
    pub fn get_all_task_executions(&self) -> Vec<TaskExecution> {
        self.task_executions.blocking_read().values().cloned().collect()
    }

    /// Clear task executions
    pub fn clear_task_executions(&self) {
        let mut executions = self.task_executions.blocking_write();
        executions.clear();
        info!("[Task] Cleared all task executions");
    }

    // ===== Cleanup =====

    /// Clear all task data for an owner
    pub fn clear_task_data(&self, owner: &str) {
        let mut providers = self.providers.blocking_write();
        providers.retain(|p| p.owner != owner);

        info!("[Task] Cleared task data for owner: {}", owner);
    }
}

// ===== Type Definitions =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProvider {
    pub id: String,
    pub owner: String,
    pub task_type: String,
    pub tasks: Vec<TaskDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDefinition {
    pub name: String,
    pub task_type: String,
    pub command: String,
    pub args: Vec<String>,
    pub options: TaskOptions,
    pub problem_matchers: Vec<ProblemMatcher>,
    pub group: Option<TaskGroup>,
    pub presentation: TaskPresentation,
    pub run_options: RunOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskOptions {
    pub cwd: Option<String>,
    pub env: HashMap<String, String>,
    pub shell: Option<ShellConfiguration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellConfiguration {
    pub executable: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemMatcher {
    pub owner: String,
    pub pattern: ProblemPattern,
    pub severity: Option<String>,
    pub source: Option<String>,
    pub file_location: FileLocationKind,
    pub background: Option<BackgroundPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemPattern {
    pub regexp: String,
    pub file: Option<usize>,
    pub location: Option<usize>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub end_line: Option<usize>,
    pub end_column: Option<usize>,
    pub message: usize,
    pub code: Option<usize>,
    pub severity: Option<usize>,
    #[serde(rename = "loop")]
    pub r#loop: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileLocationKind {
    Auto,
    Relative,
    Absolute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundPattern {
    pub active_on_start: bool,
    pub begins_pattern: String,
    pub ends_pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskGroup {
    pub kind: TaskGroupKind,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskGroupKind {
    Build,
    Test,
    Clean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPresentation {
    pub reveal: RevealKind,
    pub echo: bool,
    pub focus: bool,
    pub panel: PanelKind,
    pub show_reuse_message: bool,
    pub clear: bool,
    pub group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RevealKind {
    Always,
    Silent,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PanelKind {
    Shared,
    Dedicated,
    New,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunOptions {
    pub run_on: RunOnKind,
    pub instance_limit: usize,
    pub reevaluate_on_rerun: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunOnKind {
    Default,
    FolderOpen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskExecution {
    pub id: String,
    pub task: TaskDefinition,
    pub terminal_id: Option<String>,
    pub state: TaskExecutionState,
    pub exit_code: Option<i32>,
    pub started_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskExecutionState {
    Running,
    Completed,
    Failed,
    Cancelled,
}
