use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter};

/// Menu contribution locations (matches VS Code menu IDs)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MenuLocation {
    /// Editor context menu
    EditorContext,
    /// Editor title menu (top-right of editor)
    EditorTitle,
    /// Editor title context menu
    EditorTitleContext,
    /// Explorer context menu
    ExplorerContext,
    /// SCM title menu
    ScmTitle,
    /// SCM resource context menu
    ScmResourceContext,
    /// View title menu
    ViewTitle,
    /// View item context menu
    ViewItemContext,
    /// Command palette
    CommandPalette,
    /// Custom menu location
    Custom(String),
}

impl MenuLocation {
    pub fn from_str(s: &str) -> Self {
        match s {
            "editor/context" => Self::EditorContext,
            "editor/title" => Self::EditorTitle,
            "editor/title/context" => Self::EditorTitleContext,
            "explorer/context" => Self::ExplorerContext,
            "scm/title" => Self::ScmTitle,
            "scm/resourceState/context" => Self::ScmResourceContext,
            "view/title" => Self::ViewTitle,
            "view/item/context" => Self::ViewItemContext,
            "commandPalette" => Self::CommandPalette,
            other => Self::Custom(other.to_string()),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::EditorContext => "editor/context".to_string(),
            Self::EditorTitle => "editor/title".to_string(),
            Self::EditorTitleContext => "editor/title/context".to_string(),
            Self::ExplorerContext => "explorer/context".to_string(),
            Self::ScmTitle => "scm/title".to_string(),
            Self::ScmResourceContext => "scm/resourceState/context".to_string(),
            Self::ViewTitle => "view/title".to_string(),
            Self::ViewItemContext => "view/item/context".to_string(),
            Self::CommandPalette => "commandPalette".to_string(),
            Self::Custom(s) => s.clone(),
        }
    }
}

/// A single menu item contribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    /// Command ID to execute
    pub command: String,

    /// Menu location (where this item appears)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// When clause (conditional visibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,

    /// Group within the menu (e.g., "navigation", "1_modification")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,

    /// Title (overrides command title if provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Icon (for graphical menus)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    /// Owner (extension ID that contributed this)
    pub owner: String,

    /// Alt command (command to run when Alt is pressed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
}

impl MenuItem {
    /// Get the sort key for menu ordering
    pub fn sort_key(&self) -> (String, String) {
        let group = self.group.clone().unwrap_or_else(|| "z_other".to_string());
        let title = self.title.clone().unwrap_or_else(|| self.command.clone());
        (group, title)
    }
}

/// Registry for all menu contributions
#[derive(Clone)]
pub struct MenuRegistry {
    /// Map from MenuLocation to list of menu items
    menus: Arc<RwLock<HashMap<String, Vec<MenuItem>>>>,
    app_handle: AppHandle,
}

impl MenuRegistry {
    pub fn new(app_handle: AppHandle) -> Self {
        let registry = Self { menus: Arc::new(RwLock::new(HashMap::new())), app_handle };

        // Register built-in menu items
        registry.register_builtin_menus();

        registry
    }

    fn register_builtin_menus(&self) {
        let builtin_items = vec![
            // Editor context menu
            MenuItem {
                command: "editor.action.formatDocument".to_string(),
                location: Some("editor/context".to_string()),
                when: Some("editorHasSelection".to_string()),
                group: Some("1_modification".to_string()),
                title: Some("Format Document".to_string()),
                icon: None,
                owner: "__builtin__".to_string(),
                alt: None,
            },
            MenuItem {
                command: "editor.action.commentLine".to_string(),
                location: Some("editor/context".to_string()),
                when: Some("editorTextFocus".to_string()),
                group: Some("1_modification".to_string()),
                title: Some("Toggle Line Comment".to_string()),
                icon: None,
                owner: "__builtin__".to_string(),
                alt: None,
            },
            // Explorer context menu
            MenuItem {
                command: "explorer.newFile".to_string(),
                location: Some("explorer/context".to_string()),
                when: Some("explorerResourceIsDirectory".to_string()),
                group: Some("1_new".to_string()),
                title: Some("New File".to_string()),
                icon: None,
                owner: "__builtin__".to_string(),
                alt: None,
            },
            MenuItem {
                command: "explorer.newFolder".to_string(),
                location: Some("explorer/context".to_string()),
                when: Some("explorerResourceIsDirectory".to_string()),
                group: Some("1_new".to_string()),
                title: Some("New Folder".to_string()),
                icon: None,
                owner: "__builtin__".to_string(),
                alt: None,
            },
            MenuItem {
                command: "file.delete".to_string(),
                location: Some("explorer/context".to_string()),
                when: None,
                group: Some("2_workspace".to_string()),
                title: Some("Delete".to_string()),
                icon: None,
                owner: "__builtin__".to_string(),
                alt: None,
            },
            MenuItem {
                command: "file.rename".to_string(),
                location: Some("explorer/context".to_string()),
                when: None,
                group: Some("2_workspace".to_string()),
                title: Some("Rename".to_string()),
                icon: None,
                owner: "__builtin__".to_string(),
                alt: None,
            },
            MenuItem {
                command: "copyFilePath".to_string(),
                location: Some("explorer/context".to_string()),
                when: None,
                group: Some("5_cutcopypaste".to_string()),
                title: Some("Copy Path".to_string()),
                icon: None,
                owner: "__builtin__".to_string(),
                alt: None,
            },
            MenuItem {
                command: "copyRelativeFilePath".to_string(),
                location: Some("explorer/context".to_string()),
                when: None,
                group: Some("5_cutcopypaste".to_string()),
                title: Some("Copy Relative Path".to_string()),
                icon: None,
                owner: "__builtin__".to_string(),
                alt: None,
            },
        ];

        for item in builtin_items {
            let _ = self.register_menu_item(item);
        }
    }

    /// Register a menu item
    pub fn register_menu_item(&self, item: MenuItem) -> Result<(), String> {
        let location = item.location.clone().unwrap_or_else(|| "commandPalette".to_string());

        let mut menus = self.menus.write().expect("menu_registry write lock poisoned");

        let items = menus.entry(location.clone()).or_insert_with(Vec::new);
        items.push(item.clone());

        // Sort by group and title
        items.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));

        drop(menus);

        // Emit event to frontend
        let _ = self.app_handle.emit("menu-item-registered", &item);

        Ok(())
    }

    /// Register multiple menu items from an extension
    pub fn register_menu_items(&self, items: Vec<MenuItem>) -> Result<(), String> {
        for item in items {
            self.register_menu_item(item)?;
        }
        Ok(())
    }

    /// Get menu items for a specific location
    pub fn get_menu_items(&self, location: &str) -> Vec<MenuItem> {
        let menus = self.menus.read().expect("menu_registry read lock poisoned");
        menus.get(location).cloned().unwrap_or_default()
    }

    /// Get menu items for a location, filtered by when clause context
    pub fn get_menu_items_filtered(&self, location: &str, context: &MenuContext) -> Vec<MenuItem> {
        let items = self.get_menu_items(location);

        items
            .into_iter()
            .filter(|item| {
                if let Some(when_clause) = &item.when {
                    self.evaluate_when_clause(when_clause, context)
                } else {
                    true
                }
            })
            .collect()
    }

    /// Evaluate a when clause against context
    /// TODO: Implement full when clause parser
    fn evaluate_when_clause(&self, when: &str, context: &MenuContext) -> bool {
        // Simple implementation for now - just check for basic conditions
        match when {
            "editorHasSelection" => context.has_selection,
            "editorTextFocus" => context.editor_focused,
            "explorerViewletFocus" => context.explorer_focused,
            "resourceExtname == .rs" => context.resource_extension.as_deref() == Some("rs"),
            "resourceExtname == .ts" => context.resource_extension.as_deref() == Some("ts"),
            "resourceExtname == .js" => context.resource_extension.as_deref() == Some("js"),
            _ => {
                // Default to true for unknown clauses (permissive)
                // TODO: Implement proper when clause parser
                true
            }
        }
    }

    /// Remove all menu items from a specific owner (when extension deactivates)
    pub fn clear_menu_items_by_owner(&self, owner: &str) {
        let mut menus = self.menus.write().expect("menu_registry write lock poisoned");

        for items in menus.values_mut() {
            items.retain(|item| item.owner != owner);
        }

        // Emit event to frontend
        let _ = self.app_handle.emit("menu-items-cleared", owner);
    }

    /// Get all locations that have menu items
    pub fn get_locations(&self) -> Vec<String> {
        let menus = self.menus.read().expect("menu_registry read lock poisoned");
        menus.keys().cloned().collect()
    }
}

/// Context for evaluating when clauses
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MenuContext {
    /// Whether the editor has a selection
    pub has_selection: bool,

    /// Whether the editor is focused
    pub editor_focused: bool,

    /// Whether the explorer is focused
    pub explorer_focused: bool,

    /// File extension of the selected resource
    pub resource_extension: Option<String>,

    /// Full path of the selected resource
    pub resource_path: Option<String>,

    /// Language ID of the active editor
    pub language_id: Option<String>,

    /// Whether in debug mode
    pub in_debug_mode: bool,

    /// Custom context variables from extensions
    pub custom: HashMap<String, serde_json::Value>,
}

impl MenuContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_selection(mut self, has_selection: bool) -> Self {
        self.has_selection = has_selection;
        self
    }

    pub fn with_editor_focus(mut self, focused: bool) -> Self {
        self.editor_focused = focused;
        self
    }

    pub fn with_resource(mut self, path: String) -> Self {
        // Extract extension from path
        if let Some(ext) = std::path::Path::new(&path).extension().and_then(|e| e.to_str()) {
            self.resource_extension = Some(ext.to_string());
        }
        self.resource_path = Some(path);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_location_parsing() {
        assert_eq!(MenuLocation::from_str("editor/context"), MenuLocation::EditorContext);
        assert_eq!(MenuLocation::from_str("explorer/context"), MenuLocation::ExplorerContext);
    }

    #[test]
    fn test_menu_item_sort_key() {
        let item1 = MenuItem {
            command: "test1".to_string(),
            location: None,
            when: None,
            group: Some("1_group".to_string()),
            title: Some("B Title".to_string()),
            icon: None,
            owner: "test".to_string(),
            alt: None,
        };

        let item2 = MenuItem {
            command: "test2".to_string(),
            location: None,
            when: None,
            group: Some("1_group".to_string()),
            title: Some("A Title".to_string()),
            icon: None,
            owner: "test".to_string(),
            alt: None,
        };

        assert!(item2.sort_key() < item1.sort_key());
    }
}
