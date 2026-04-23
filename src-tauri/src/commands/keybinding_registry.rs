use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter};
use tracing::warn;

/// A single keybinding contribution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Keybinding {
    /// Command to execute
    pub command: String,

    /// Key combination (e.g., "Ctrl+Shift+P", "Cmd+K Cmd+S")
    pub key: String,

    /// When clause (conditional activation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,

    /// Platform-specific (windows, mac, linux)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,

    /// Owner (extension ID that contributed this)
    pub owner: String,

    /// Arguments to pass to command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,
}

impl Keybinding {
    /// Normalize the key string for comparison
    /// Converts to lowercase, standardizes separators
    pub fn normalize_key(key: &str) -> String {
        key.to_lowercase()
            .replace(" ", "+")
            .replace("command", "cmd")
            .replace("control", "ctrl")
            .replace("option", "alt")
            .replace("meta", "cmd")
    }

    /// Get normalized key for this keybinding
    pub fn normalized_key(&self) -> String {
        Self::normalize_key(&self.key)
    }

    /// Check if this keybinding applies to the current platform
    pub fn applies_to_platform(&self, current_platform: &str) -> bool {
        if let Some(platform) = &self.platform {
            platform.to_lowercase() == current_platform.to_lowercase()
        } else {
            // No platform specified means it applies to all platforms
            true
        }
    }

    /// Convert platform-specific key to current platform
    /// E.g., "Cmd+S" on Mac becomes "Ctrl+S" on Windows/Linux
    pub fn platform_key(&self, target_platform: &str) -> String {
        let key = self.key.as_str();

        match target_platform {
            "macos" => {
                // Convert Ctrl to Cmd on Mac
                key.replace("Ctrl", "Cmd")
            }
            "windows" | "linux" => {
                // Convert Cmd to Ctrl on Windows/Linux
                key.replace("Cmd", "Ctrl")
            }
            _ => key.to_string(),
        }
    }
}

/// Registry for all keybindings
#[derive(Clone)]
pub struct KeybindingRegistry {
    /// Map from normalized key to list of keybindings
    /// Multiple commands can be bound to the same key (differentiated by when clause)
    keybindings: Arc<RwLock<HashMap<String, Vec<Keybinding>>>>,

    /// Current platform (windows, macos, linux)
    platform: String,

    app_handle: AppHandle,
}

impl KeybindingRegistry {
    pub fn new(app_handle: AppHandle) -> Self {
        let platform = Self::detect_platform();

        let registry = Self {
            keybindings: Arc::new(RwLock::new(HashMap::new())),
            platform: platform.clone(),
            app_handle,
        };

        // Register built-in keybindings
        registry.register_builtin_keybindings(&platform);

        registry
    }

    /// Detect current platform
    fn detect_platform() -> String {
        #[cfg(target_os = "macos")]
        return "macos".to_string();

        #[cfg(target_os = "windows")]
        return "windows".to_string();

        #[cfg(target_os = "linux")]
        return "linux".to_string();

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        return "unknown".to_string();
    }

    fn register_builtin_keybindings(&self, platform: &str) {
        let modifier = if platform == "macos" { "Cmd" } else { "Ctrl" };

        let builtins = vec![
            // File operations
            Keybinding {
                command: "file.save".to_string(),
                key: format!("{}+S", modifier),
                when: Some("activeEditor".to_string()),
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            Keybinding {
                command: "file.saveAll".to_string(),
                key: format!("{}+K {}+S", modifier, modifier),
                when: Some("activeEditor".to_string()),
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            Keybinding {
                command: "file.close".to_string(),
                key: format!("{}+W", modifier),
                when: Some("activeEditor".to_string()),
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            Keybinding {
                command: "file.closeAll".to_string(),
                key: format!("{}+K {}+W", modifier, modifier),
                when: Some("activeEditor".to_string()),
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            // View operations
            Keybinding {
                command: "view.toggleSidebar".to_string(),
                key: format!("{}+B", modifier),
                when: None,
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            Keybinding {
                command: "view.togglePanel".to_string(),
                key: format!("{}+J", modifier),
                when: None,
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            // Editor operations
            Keybinding {
                command: "editor.action.formatDocument".to_string(),
                key: "Shift+Alt+F".to_string(),
                when: Some("editorTextFocus".to_string()),
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            // Command palette and quick open
            Keybinding {
                command: "workbench.action.showCommands".to_string(),
                key: format!("{}+Shift+P", modifier),
                when: None,
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            Keybinding {
                command: "workbench.action.quickOpen".to_string(),
                key: format!("{}+P", modifier),
                when: None,
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            Keybinding {
                command: "workbench.action.showAllSymbols".to_string(),
                key: format!("{}+T", modifier),
                when: None,
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            Keybinding {
                command: "workbench.view.explorer".to_string(),
                key: format!("{}+Shift+E", modifier),
                when: None,
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            Keybinding {
                command: "workbench.action.findInFiles".to_string(),
                key: format!("{}+Shift+F", modifier),
                when: None,
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            Keybinding {
                command: "workbench.view.scm".to_string(),
                key: format!("{}+Shift+G", modifier),
                when: None,
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            Keybinding {
                command: "workbench.view.extensions".to_string(),
                key: format!("{}+Shift+X", modifier),
                when: None,
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            Keybinding {
                command: "workbench.action.openSettings".to_string(),
                key: format!("{}+,", modifier),
                when: None,
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            Keybinding {
                command: "dscode.toggleResourceMetrics".to_string(),
                key: format!("{}+Shift+M", modifier),
                when: None,
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            // Developer
            Keybinding {
                command: "workbench.action.reloadWindow".to_string(),
                key: format!("{}+R", modifier),
                when: None,
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            // Find and replace
            Keybinding {
                command: "editor.action.startFindReplaceAction".to_string(),
                key: format!("{}+H", modifier),
                when: Some("editorTextFocus".to_string()),
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            Keybinding {
                command: "actions.find".to_string(),
                key: format!("{}+F", modifier),
                when: Some("editorTextFocus".to_string()),
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            // Navigation
            Keybinding {
                command: "workbench.action.navigateBack".to_string(),
                key: format!("{}+Alt+Left", modifier),
                when: None,
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
            Keybinding {
                command: "workbench.action.navigateForward".to_string(),
                key: format!("{}+Alt+Right", modifier),
                when: None,
                platform: None,
                owner: "__builtin__".to_string(),
                args: None,
            },
        ];

        for binding in builtins {
            let _ = self.register_keybinding(binding);
        }
    }

    /// Register a keybinding
    pub fn register_keybinding(&self, keybinding: Keybinding) -> Result<(), String> {
        // Check if keybinding applies to current platform
        if !keybinding.applies_to_platform(&self.platform) {
            // Skip keybindings for other platforms
            return Ok(());
        }

        let normalized_key = keybinding.normalized_key();
        let mut keybindings = self.keybindings.write().unwrap_or_else(|e| {
            warn!("keybinding_registry write lock poisoned, recovering: {}", e);
            e.into_inner()
        });

        let bindings = keybindings.entry(normalized_key).or_default();

        // Check for conflicts (same key + same when clause)
        for existing in bindings.iter_mut() {
            if existing.command == keybinding.command && existing.when == keybinding.when {
                // Same binding already exists
                if existing.owner != keybinding.owner {
                    warn!(
                        "Keybinding conflict for '{}': '{}' (owner: {}) vs '{}' (owner: {})",
                        keybinding.key,
                        existing.command,
                        existing.owner,
                        keybinding.command,
                        keybinding.owner
                    );
                }
                // Replace existing
                *existing = keybinding.clone();
                drop(keybindings);

                // Emit event
                let _ = self.app_handle.emit("keybinding-registered", &keybinding);
                return Ok(());
            }
        }

        // Add new binding
        bindings.push(keybinding.clone());

        drop(keybindings);

        // Emit event to frontend
        let _ = self.app_handle.emit("keybinding-registered", &keybinding);

        Ok(())
    }

    /// Register multiple keybindings
    pub fn register_keybindings(&self, keybindings: Vec<Keybinding>) -> Result<(), String> {
        for keybinding in keybindings {
            self.register_keybinding(keybinding)?;
        }
        Ok(())
    }

    /// Get all keybindings for a specific key
    pub fn get_keybindings_for_key(&self, key: &str) -> Vec<Keybinding> {
        let normalized_key = Keybinding::normalize_key(key);
        let keybindings = self.keybindings.read().unwrap_or_else(|e| {
            warn!("keybinding_registry read lock poisoned, recovering: {}", e);
            e.into_inner()
        });

        keybindings.get(&normalized_key).cloned().unwrap_or_default()
    }

    /// Get all keybindings
    pub fn get_all_keybindings(&self) -> Vec<Keybinding> {
        let keybindings = self.keybindings.read().unwrap_or_else(|e| {
            warn!("keybinding_registry read lock poisoned, recovering: {}", e);
            e.into_inner()
        });
        keybindings.values().flatten().cloned().collect()
    }

    /// Get keybindings for a specific command
    pub fn get_keybindings_for_command(&self, command: &str) -> Vec<Keybinding> {
        let keybindings = self.keybindings.read().unwrap_or_else(|e| {
            warn!("keybinding_registry read lock poisoned, recovering: {}", e);
            e.into_inner()
        });
        keybindings.values().flatten().filter(|kb| kb.command == command).cloned().collect()
    }

    /// Remove all keybindings from a specific owner
    pub fn clear_keybindings_by_owner(&self, owner: &str) {
        let mut keybindings = self.keybindings.write().unwrap_or_else(|e| {
            warn!("keybinding_registry write lock poisoned, recovering: {}", e);
            e.into_inner()
        });

        for bindings in keybindings.values_mut() {
            bindings.retain(|kb| kb.owner != owner);
        }

        // Remove empty entries
        keybindings.retain(|_, bindings| !bindings.is_empty());

        // Emit event to frontend
        let _ = self.app_handle.emit("keybindings-cleared", owner);
    }

    /// Get current platform
    pub fn get_platform(&self) -> String {
        self.platform.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_key() {
        assert_eq!(Keybinding::normalize_key("Ctrl+S"), "ctrl+s");
        assert_eq!(Keybinding::normalize_key("Cmd+Shift+P"), "cmd+shift+p");
        assert_eq!(Keybinding::normalize_key("Control+Alt+Delete"), "ctrl+alt+delete");
    }

    #[test]
    fn test_platform_key() {
        let kb = Keybinding {
            command: "test".to_string(),
            key: "Ctrl+S".to_string(),
            when: None,
            platform: None,
            owner: "test".to_string(),
            args: None,
        };

        assert_eq!(kb.platform_key("macos"), "Cmd+S");
        assert_eq!(kb.platform_key("windows"), "Ctrl+S");
    }
}
