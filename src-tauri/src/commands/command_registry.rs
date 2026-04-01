use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    pub id: String,
    pub label: String,
    pub category: Option<String>,
    pub owner: String, // Extension ID that registered this command
    pub keybinding: Option<String>,
    pub when: Option<String>, // When clause for conditional availability
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecutionResult {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Central registry for all commands (built-in + extension-contributed)
#[derive(Clone)]
pub struct CommandRegistry {
    commands: Arc<RwLock<HashMap<String, CommandInfo>>>,
    app_handle: AppHandle,
}

impl CommandRegistry {
    pub fn new(app_handle: AppHandle) -> Self {
        let registry = Self { commands: Arc::new(RwLock::new(HashMap::new())), app_handle };

        // Register built-in commands
        registry.register_builtin_commands();

        registry
    }

    fn register_builtin_commands(&self) {
        let builtins = vec![
            CommandInfo {
                id: "file.save".to_string(),
                label: "File: Save".to_string(),
                category: Some("File".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+S".to_string()),
                when: Some("activeEditor".to_string()),
            },
            CommandInfo {
                id: "file.saveAll".to_string(),
                label: "File: Save All".to_string(),
                category: Some("File".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+K S".to_string()),
                when: Some("activeEditor".to_string()),
            },
            CommandInfo {
                id: "file.close".to_string(),
                label: "File: Close Editor".to_string(),
                category: Some("File".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+W".to_string()),
                when: Some("activeEditor".to_string()),
            },
            CommandInfo {
                id: "file.closeAll".to_string(),
                label: "File: Close All Editors".to_string(),
                category: Some("File".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+K W".to_string()),
                when: Some("activeEditor".to_string()),
            },
            CommandInfo {
                id: "view.toggleSidebar".to_string(),
                label: "View: Toggle Sidebar".to_string(),
                category: Some("View".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+B".to_string()),
                when: None,
            },
            CommandInfo {
                id: "view.togglePanel".to_string(),
                label: "View: Toggle Panel".to_string(),
                category: Some("View".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+J".to_string()),
                when: None,
            },
            CommandInfo {
                id: "editor.action.formatDocument".to_string(),
                label: "Format Document".to_string(),
                category: Some("Editor".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Shift+Alt+F".to_string()),
                when: Some("editorTextFocus".to_string()),
            },
            CommandInfo {
                id: "actions.find".to_string(),
                label: "Edit: Find".to_string(),
                category: Some("Edit".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+F".to_string()),
                when: Some("editorTextFocus".to_string()),
            },
            CommandInfo {
                id: "editor.action.startFindReplaceAction".to_string(),
                label: "Edit: Replace".to_string(),
                category: Some("Edit".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+H".to_string()),
                when: Some("editorTextFocus".to_string()),
            },
            CommandInfo {
                id: "workbench.action.showCommands".to_string(),
                label: "Show All Commands".to_string(),
                category: Some("View".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+Shift+P".to_string()),
                when: None,
            },
            CommandInfo {
                id: "workbench.action.quickOpen".to_string(),
                label: "Go to File...".to_string(),
                category: Some("View".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+P".to_string()),
                when: None,
            },
            CommandInfo {
                id: "workbench.action.showAllSymbols".to_string(),
                label: "Go to Symbol in Workspace...".to_string(),
                category: Some("View".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+T".to_string()),
                when: None,
            },
            CommandInfo {
                id: "workbench.view.explorer".to_string(),
                label: "View: Show Explorer".to_string(),
                category: Some("View".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+Shift+E".to_string()),
                when: None,
            },
            CommandInfo {
                id: "workbench.action.findInFiles".to_string(),
                label: "Search: Find in Files".to_string(),
                category: Some("Search".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+Shift+F".to_string()),
                when: None,
            },
            CommandInfo {
                id: "workbench.view.scm".to_string(),
                label: "View: Show Source Control".to_string(),
                category: Some("View".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+Shift+G".to_string()),
                when: None,
            },
            CommandInfo {
                id: "workbench.view.extensions".to_string(),
                label: "View: Show Extensions".to_string(),
                category: Some("View".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+Shift+X".to_string()),
                when: None,
            },
            CommandInfo {
                id: "workbench.action.openSettings".to_string(),
                label: "Preferences: Open Settings".to_string(),
                category: Some("Preferences".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+,".to_string()),
                when: None,
            },
            CommandInfo {
                id: "dscode.toggleResourceMetrics".to_string(),
                label: "Developer: Toggle Resource Metrics".to_string(),
                category: Some("Developer".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+Shift+M".to_string()),
                when: None,
            },
            CommandInfo {
                id: "workbench.action.reloadWindow".to_string(),
                label: "Developer: Reload Window".to_string(),
                category: Some("Developer".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+R".to_string()),
                when: None,
            },
            CommandInfo {
                id: "workbench.action.navigateBack".to_string(),
                label: "Go: Navigate Back".to_string(),
                category: Some("Go".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+Alt+Left".to_string()),
                when: None,
            },
            CommandInfo {
                id: "workbench.action.navigateForward".to_string(),
                label: "Go: Navigate Forward".to_string(),
                category: Some("Go".to_string()),
                owner: "__builtin__".to_string(),
                keybinding: Some("Ctrl+Alt+Right".to_string()),
                when: None,
            },
        ];

        let mut commands = self.commands.write().unwrap_or_else(|e| {
            warn!("command_registry write lock poisoned, recovering: {}", e);
            e.into_inner()
        });
        for cmd in builtins {
            commands.insert(cmd.id.clone(), cmd);
        }
    }

    /// Register a command from an extension
    pub fn register_command(&self, command: CommandInfo) -> Result<(), String> {
        let mut commands = self.commands.write().unwrap_or_else(|e| {
            warn!("command_registry write lock poisoned, recovering: {}", e);
            e.into_inner()
        });

        // Check for conflicts
        if let Some(existing) = commands.get(&command.id) {
            if existing.owner != command.owner {
                warn!(
                    "Command '{}' already registered by '{}', overriding with '{}'",
                    command.id, existing.owner, command.owner
                );
            }
        }

        commands.insert(command.id.clone(), command.clone());

        // Emit event to frontend
        let _ = self.app_handle.emit("command-registered", &command);

        Ok(())
    }

    /// Unregister a command
    pub fn unregister_command(&self, command_id: &str, owner: &str) -> Result<(), String> {
        let mut commands = self.commands.write().unwrap_or_else(|e| {
            warn!("command_registry write lock poisoned, recovering: {}", e);
            e.into_inner()
        });

        if let Some(existing) = commands.get(command_id) {
            // Only allow owner to unregister their own commands
            if existing.owner != owner {
                return Err(format!(
                    "Cannot unregister command '{}': owned by '{}', not '{}'",
                    command_id, existing.owner, owner
                ));
            }

            commands.remove(command_id);

            // Emit event to frontend
            let _ = self.app_handle.emit("command-unregistered", command_id);

            Ok(())
        } else {
            Err(format!("Command '{}' not found", command_id))
        }
    }

    /// Get all registered commands
    pub fn get_all_commands(&self) -> Vec<CommandInfo> {
        let commands = self.commands.read().unwrap_or_else(|e| {
            warn!("command_registry read lock poisoned, recovering: {}", e);
            e.into_inner()
        });
        commands.values().cloned().collect()
    }

    /// Get a specific command
    pub fn get_command(&self, command_id: &str) -> Option<CommandInfo> {
        let commands = self.commands.read().unwrap_or_else(|e| {
            warn!("command_registry read lock poisoned, recovering: {}", e);
            e.into_inner()
        });
        commands.get(command_id).cloned()
    }

    /// Search commands by query
    pub fn search_commands(&self, query: &str) -> Vec<CommandInfo> {
        let commands = self.commands.read().unwrap_or_else(|e| {
            warn!("command_registry read lock poisoned, recovering: {}", e);
            e.into_inner()
        });
        let query_lower = query.to_lowercase();

        let mut results: Vec<CommandInfo> = commands
            .values()
            .filter(|cmd| {
                cmd.label.to_lowercase().contains(&query_lower)
                    || cmd.id.to_lowercase().contains(&query_lower)
                    || cmd
                        .category
                        .as_ref()
                        .map(|c| c.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .cloned()
            .collect();

        // Sort by relevance (exact matches first, then starts with, then contains)
        results.sort_by(|a, b| {
            let a_label_lower = a.label.to_lowercase();
            let b_label_lower = b.label.to_lowercase();

            let a_exact = a_label_lower == query_lower;
            let b_exact = b_label_lower == query_lower;
            if a_exact && !b_exact {
                return std::cmp::Ordering::Less;
            }
            if !a_exact && b_exact {
                return std::cmp::Ordering::Greater;
            }

            let a_starts = a_label_lower.starts_with(&query_lower);
            let b_starts = b_label_lower.starts_with(&query_lower);
            if a_starts && !b_starts {
                return std::cmp::Ordering::Less;
            }
            if !a_starts && b_starts {
                return std::cmp::Ordering::Greater;
            }

            a.label.cmp(&b.label)
        });

        results
    }

    /// Clear all commands from a specific owner (used when extension deactivates)
    pub fn clear_commands_by_owner(&self, owner: &str) {
        let mut commands = self.commands.write().unwrap_or_else(|e| {
            warn!("command_registry write lock poisoned, recovering: {}", e);
            e.into_inner()
        });
        commands.retain(|_, cmd| cmd.owner != owner);

        // Emit event to frontend
        let _ = self.app_handle.emit("commands-cleared", owner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a Tauri AppHandle, so they're more like integration tests
    // For unit tests, we'd need to mock the AppHandle or extract the logic
}
