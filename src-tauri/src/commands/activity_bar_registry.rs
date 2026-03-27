use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityBarItem {
    pub id: String,
    pub owner: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>, // SVG path or icon identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_path: Option<String>, // Path to icon file
    pub priority: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge_text: Option<String>,
    pub visible: bool,
}

/// Registry for activity bar items contributed by extensions
pub struct ActivityBarRegistry {
    items: Arc<RwLock<HashMap<String, ActivityBarItem>>>,
    app_handle: AppHandle,
}

impl ActivityBarRegistry {
    pub fn new(app_handle: AppHandle) -> Self {
        let registry = Self { items: Arc::new(RwLock::new(HashMap::new())), app_handle };

        // Register built-in activity bar items
        registry.register_built_in_items();

        registry
    }

    /// Register built-in activity bar items
    fn register_built_in_items(&self) {
        let built_in_items = vec![
            ActivityBarItem {
                id: "explorer".to_string(),
                owner: "__core__".to_string(),
                title: "Explorer".to_string(),
                icon: Some("📁".to_string()),
                icon_path: None,
                priority: 1000,
                badge_count: None,
                badge_text: None,
                visible: true,
            },
            ActivityBarItem {
                id: "search".to_string(),
                owner: "__core__".to_string(),
                title: "Search".to_string(),
                icon: Some("🔍".to_string()),
                icon_path: None,
                priority: 900,
                badge_count: None,
                badge_text: None,
                visible: true,
            },
            ActivityBarItem {
                id: "scm".to_string(),
                owner: "__core__".to_string(),
                title: "Source Control".to_string(),
                icon: Some("🔀".to_string()),
                icon_path: None,
                priority: 800,
                badge_count: None,
                badge_text: None,
                visible: true,
            },
            ActivityBarItem {
                id: "debug".to_string(),
                owner: "__core__".to_string(),
                title: "Run and Debug".to_string(),
                icon: Some("🐛".to_string()),
                icon_path: None,
                priority: 700,
                badge_count: None,
                badge_text: None,
                visible: true,
            },
            ActivityBarItem {
                id: "extensions".to_string(),
                owner: "__core__".to_string(),
                title: "Extensions".to_string(),
                icon: Some("🧩".to_string()),
                icon_path: None,
                priority: 600,
                badge_count: None,
                badge_text: None,
                visible: true,
            },
        ];

        let mut items = self.items.write().expect("activity_bar_registry write lock poisoned");
        for item in built_in_items {
            items.insert(item.id.clone(), item);
        }
    }

    /// Create a unique key for an activity bar item
    fn item_key(owner: &str, id: &str) -> String {
        format!("{}:{}", owner, id)
    }

    /// Register a new activity bar item
    pub fn register_item(
        &self, owner: String, id: String, title: String, icon: Option<String>,
        icon_path: Option<String>, priority: Option<i32>,
    ) -> Result<String, String> {
        let key = Self::item_key(&owner, &id);

        let item = ActivityBarItem {
            id: id.clone(),
            owner,
            title,
            icon,
            icon_path,
            priority: priority.unwrap_or(0),
            badge_count: None,
            badge_text: None,
            visible: true,
        };

        {
            let mut items = self.items.write().map_err(|e| e.to_string())?;
            items.insert(key.clone(), item);
        }

        self.emit_items_changed();

        Ok(key)
    }

    /// Update an activity bar item's badge
    pub fn update_badge(
        &self, key: String, badge_count: Option<i32>, badge_text: Option<String>,
    ) -> Result<(), String> {
        let is_visible = {
            let mut items = self.items.write().map_err(|e| e.to_string())?;

            let item = items
                .get_mut(&key)
                .ok_or_else(|| format!("Activity bar item not found: {}", key))?;

            item.badge_count = badge_count;
            item.badge_text = badge_text;

            item.visible
        }; // Lock released here

        if is_visible {
            self.emit_items_changed();
        }

        Ok(())
    }

    /// Show an activity bar item
    pub fn show_item(&self, key: String) -> Result<(), String> {
        let mut items = self.items.write().map_err(|e| e.to_string())?;

        let item =
            items.get_mut(&key).ok_or_else(|| format!("Activity bar item not found: {}", key))?;

        if !item.visible {
            item.visible = true;
            drop(items); // Release lock before emitting
            self.emit_items_changed();
        }

        Ok(())
    }

    /// Hide an activity bar item
    pub fn hide_item(&self, key: String) -> Result<(), String> {
        let mut items = self.items.write().map_err(|e| e.to_string())?;

        let item =
            items.get_mut(&key).ok_or_else(|| format!("Activity bar item not found: {}", key))?;

        if item.visible {
            item.visible = false;
            drop(items); // Release lock before emitting
            self.emit_items_changed();
        }

        Ok(())
    }

    /// Dispose (remove) an activity bar item
    pub fn dispose_item(&self, key: String) -> Result<(), String> {
        let mut items = self.items.write().map_err(|e| e.to_string())?;

        let item =
            items.remove(&key).ok_or_else(|| format!("Activity bar item not found: {}", key))?;

        drop(items); // Release lock before emitting

        // Emit update event if item was visible
        if item.visible {
            self.emit_items_changed();
        }

        Ok(())
    }

    /// Get all visible activity bar items sorted by priority
    pub fn get_visible_items(&self) -> Vec<ActivityBarItem> {
        let items = self.items.read().expect("activity_bar_registry read lock poisoned");

        let mut visible_items: Vec<ActivityBarItem> =
            items.values().filter(|item| item.visible).cloned().collect();

        // Sort by priority (higher priority first)
        visible_items.sort_by(|a, b| b.priority.cmp(&a.priority));

        visible_items
    }

    /// Get a specific activity bar item
    pub fn get_item(&self, key: &str) -> Option<ActivityBarItem> {
        let items = self.items.read().expect("activity_bar_registry read lock poisoned");
        items.get(key).cloned()
    }

    /// Clear all activity bar items from a specific owner (for cleanup)
    pub fn clear_owner_items(&self, owner: &str) -> Result<(), String> {
        let mut items = self.items.write().map_err(|e| e.to_string())?;

        let keys_to_remove: Vec<String> = items
            .iter()
            .filter(|(_, item)| item.owner == owner && item.owner != "__core__")
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

        if let Err(e) = self.app_handle.emit("activity-bar-items-changed", &visible_items) {
            eprintln!("[ActivityBarRegistry] Failed to emit event: {}", e);
        }
    }

    /// Get the AppHandle
    pub fn app_handle(&self) -> &AppHandle {
        &self.app_handle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_key() {
        let key = ActivityBarRegistry::item_key("my-extension", "custom-view");
        assert_eq!(key, "my-extension:custom-view");
    }
}
