use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarCommand {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusBarAlignment {
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarItem {
    pub id: String,
    pub owner: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<StatusBarCommand>,
    pub alignment: StatusBarAlignment,
    pub priority: i32,
    pub visible: bool,
}

/// Registry for status bar items contributed by extensions
pub struct StatusBarRegistry {
    items: Arc<RwLock<HashMap<String, StatusBarItem>>>,
    app_handle: AppHandle,
}

impl StatusBarRegistry {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { items: Arc::new(RwLock::new(HashMap::new())), app_handle }
    }

    /// Create a unique key for a status bar item
    fn item_key(owner: &str, id: &str) -> String {
        format!("{}:{}", owner, id)
    }

    /// Create a new status bar item
    pub fn create_item(
        &self, owner: String, id: String, alignment: StatusBarAlignment, priority: Option<i32>,
    ) -> Result<String, String> {
        let key = Self::item_key(&owner, &id);

        let item = StatusBarItem {
            id: id.clone(),
            owner,
            text: String::new(),
            tooltip: None,
            color: None,
            background_color: None,
            command: None,
            alignment,
            priority: priority.unwrap_or(0),
            visible: false, // Hidden by default until show() is called
        };

        {
            let mut items = self.items.write().map_err(|e| e.to_string())?;
            items.insert(key.clone(), item);
        }

        // Don't emit event yet - item is hidden
        Ok(key)
    }

    /// Update an existing status bar item
    pub fn update_item(
        &self, key: String, text: Option<String>, tooltip: Option<String>, color: Option<String>,
        background_color: Option<String>, command: Option<StatusBarCommand>,
    ) -> Result<(), String> {
        let mut items = self.items.write().map_err(|e| e.to_string())?;

        let item =
            items.get_mut(&key).ok_or_else(|| format!("Status bar item not found: {}", key))?;

        if let Some(text) = text {
            item.text = text;
        }
        if let Some(tooltip) = tooltip {
            item.tooltip = Some(tooltip);
        }
        if let Some(color) = color {
            item.color = Some(color);
        }
        if let Some(bg_color) = background_color {
            item.background_color = Some(bg_color);
        }
        if let Some(cmd) = command {
            item.command = Some(cmd);
        }

        // Clone the item for event emission
        let updated_item = item.clone();
        drop(items); // Release lock before emitting

        // Emit update event if item is visible
        if updated_item.visible {
            self.emit_items_changed();
        }

        Ok(())
    }

    /// Show a status bar item
    pub fn show_item(&self, key: String) -> Result<(), String> {
        let mut items = self.items.write().map_err(|e| e.to_string())?;

        let item =
            items.get_mut(&key).ok_or_else(|| format!("Status bar item not found: {}", key))?;

        if !item.visible {
            item.visible = true;
            drop(items); // Release lock before emitting
            self.emit_items_changed();
        }

        Ok(())
    }

    /// Hide a status bar item
    pub fn hide_item(&self, key: String) -> Result<(), String> {
        let mut items = self.items.write().map_err(|e| e.to_string())?;

        let item =
            items.get_mut(&key).ok_or_else(|| format!("Status bar item not found: {}", key))?;

        if item.visible {
            item.visible = false;
            drop(items); // Release lock before emitting
            self.emit_items_changed();
        }

        Ok(())
    }

    /// Dispose (remove) a status bar item
    pub fn dispose_item(&self, key: String) -> Result<(), String> {
        let mut items = self.items.write().map_err(|e| e.to_string())?;

        let item =
            items.remove(&key).ok_or_else(|| format!("Status bar item not found: {}", key))?;

        drop(items); // Release lock before emitting

        // Emit update event if item was visible
        if item.visible {
            self.emit_items_changed();
        }

        Ok(())
    }

    /// Get all visible status bar items sorted by priority
    pub fn get_visible_items(&self) -> Vec<StatusBarItem> {
        let items = self.items.read().expect("status_bar_registry read lock poisoned");

        let mut visible_items: Vec<StatusBarItem> =
            items.values().filter(|item| item.visible).cloned().collect();

        // Sort by priority (higher priority first)
        visible_items.sort_by(|a, b| b.priority.cmp(&a.priority));

        visible_items
    }

    /// Get a specific status bar item
    pub fn get_item(&self, key: &str) -> Option<StatusBarItem> {
        let items = self.items.read().expect("status_bar_registry read lock poisoned");
        items.get(key).cloned()
    }

    /// Clear all status bar items from a specific owner (for cleanup)
    pub fn clear_owner_items(&self, owner: &str) -> Result<(), String> {
        let mut items = self.items.write().map_err(|e| e.to_string())?;

        let keys_to_remove: Vec<String> = items
            .iter()
            .filter(|(_, item)| item.owner == owner)
            .map(|(key, _)| key.clone())
            .collect();

        let had_visible =
            keys_to_remove.iter().any(|key| items.get(key).map(|i| i.visible).unwrap_or(false));

        for key in keys_to_remove {
            items.remove(&key);
        }

        drop(items); // Release lock before emitting

        // Emit update event if any visible items were removed
        if had_visible {
            self.emit_items_changed();
        }

        Ok(())
    }

    /// Emit event to notify frontend of changes
    fn emit_items_changed(&self) {
        let visible_items = self.get_visible_items();

        if let Err(e) = self.app_handle.emit("status-bar-items-changed", &visible_items) {
            eprintln!("[StatusBarRegistry] Failed to emit event: {}", e);
        }
    }

    /// Get the AppHandle (for creating items from commands)
    pub fn app_handle(&self) -> &AppHandle {
        &self.app_handle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would require a Tauri AppHandle mock
    // For now, they serve as documentation of expected behavior

    #[test]
    fn test_item_key() {
        let key = StatusBarRegistry::item_key("my-extension", "status-item-1");
        assert_eq!(key, "my-extension:status-item-1");
    }
}
