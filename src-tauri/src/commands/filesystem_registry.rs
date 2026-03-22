use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter};

/// File system provider for custom file systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemProvider {
    pub id: String,
    pub owner: String,
    pub scheme: String,
    pub is_case_sensitive: bool,
    pub is_readonly: bool,
}

/// File watcher with glob pattern support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWatcher {
    pub id: String,
    pub owner: String,
    pub glob_pattern: String,
    pub ignore_create_events: bool,
    pub ignore_change_events: bool,
    pub ignore_delete_events: bool,
}

/// File change event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileChangeType {
    Created,
    Changed,
    Deleted,
}

/// File change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeEvent {
    pub uri: String,
    #[serde(rename = "type")]
    pub change_type: FileChangeType,
}

/// File stat information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStat {
    pub uri: String,
    #[serde(rename = "type")]
    pub file_type: FileType,
    pub ctime: u64,
    pub mtime: u64,
    pub size: u64,
    pub permissions: Option<u32>,
}

/// File type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Unknown,
    File,
    Directory,
    SymbolicLink,
}

/// Directory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub name: String,
    #[serde(rename = "type")]
    pub file_type: FileType,
}

/// Workspace edit for batch file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceEdit {
    pub id: String,
    pub edits: Vec<FileEdit>,
}

/// File edit operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FileEdit {
    #[serde(rename = "create")]
    CreateFile {
        uri: String,
        options: Option<CreateFileOptions>,
    },
    #[serde(rename = "delete")]
    DeleteFile {
        uri: String,
        options: Option<DeleteFileOptions>,
    },
    #[serde(rename = "rename")]
    RenameFile {
        old_uri: String,
        new_uri: String,
        options: Option<RenameFileOptions>,
    },
    #[serde(rename = "text")]
    TextEdit {
        uri: String,
        edits: Vec<TextEditItem>,
    },
}

/// Text edit item for workspace edits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditItem {
    pub range: TextRange,
    pub new_text: String,
}

/// Text range for text edits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRange {
    pub start: TextPosition,
    pub end: TextPosition,
}

/// Text position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPosition {
    pub line: u32,
    pub character: u32,
}

/// Options for creating files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFileOptions {
    pub overwrite: bool,
    pub ignore_if_exists: bool,
}

/// Options for deleting files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFileOptions {
    pub recursive: bool,
    pub ignore_if_not_exists: bool,
}

/// Options for renaming files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameFileOptions {
    pub overwrite: bool,
}

/// File system registry
pub struct FileSystemRegistry {
    providers: Arc<RwLock<Vec<FileSystemProvider>>>,
    watchers: Arc<RwLock<HashMap<String, FileWatcher>>>,
    app_handle: AppHandle,
}

impl FileSystemRegistry {
    /// Create a new file system registry
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            providers: Arc::new(RwLock::new(Vec::new())),
            watchers: Arc::new(RwLock::new(HashMap::new())),
            app_handle,
        }
    }

    /// Register a file system provider
    pub fn register_file_system_provider(
        &self,
        provider: FileSystemProvider,
    ) -> Result<String, String> {
        let mut providers = self.providers.write().map_err(|e| e.to_string())?;

        // Check for duplicate scheme
        if providers.iter().any(|p| p.scheme == provider.scheme) {
            return Err(format!(
                "File system provider for scheme '{}' already registered",
                provider.scheme
            ));
        }

        let id = provider.id.clone();
        providers.push(provider.clone());

        println!(
            "[FileSystem] Registered provider: {} (scheme: {})",
            id, provider.scheme
        );

        // Emit event
        if let Err(e) = self
            .app_handle
            .emit("filesystem-provider-registered", &provider)
        {
            eprintln!("[FileSystem] Failed to emit provider registered event: {}", e);
        }

        Ok(id)
    }

    /// Unregister a file system provider
    pub fn unregister_file_system_provider(&self, scheme: &str) -> Result<(), String> {
        let mut providers = self.providers.write().map_err(|e| e.to_string())?;

        let initial_len = providers.len();
        providers.retain(|p| p.scheme != scheme);

        if providers.len() == initial_len {
            return Err(format!(
                "No file system provider found for scheme '{}'",
                scheme
            ));
        }

        println!("[FileSystem] Unregistered provider for scheme: {}", scheme);

        // Emit event
        if let Err(e) = self
            .app_handle
            .emit("filesystem-provider-unregistered", scheme)
        {
            eprintln!(
                "[FileSystem] Failed to emit provider unregistered event: {}",
                e
            );
        }

        Ok(())
    }

    /// Get file system provider for scheme
    pub fn get_file_system_provider(&self, scheme: &str) -> Result<FileSystemProvider, String> {
        let providers = self.providers.read().map_err(|e| e.to_string())?;

        providers
            .iter()
            .find(|p| p.scheme == scheme)
            .cloned()
            .ok_or_else(|| format!("No file system provider found for scheme '{}'", scheme))
    }

    /// Get all file system providers
    pub fn get_all_file_system_providers(&self) -> Vec<FileSystemProvider> {
        self.providers.read().unwrap().clone()
    }

    /// Create a file watcher
    pub fn create_file_watcher(&self, watcher: FileWatcher) -> Result<String, String> {
        let mut watchers = self.watchers.write().map_err(|e| e.to_string())?;

        let id = watcher.id.clone();
        watchers.insert(id.clone(), watcher.clone());

        println!(
            "[FileSystem] Created watcher: {} (pattern: {})",
            id, watcher.glob_pattern
        );

        // Emit event
        if let Err(e) = self.app_handle.emit("file-watcher-created", &watcher) {
            eprintln!("[FileSystem] Failed to emit watcher created event: {}", e);
        }

        Ok(id)
    }

    /// Dispose a file watcher
    pub fn dispose_file_watcher(&self, watcher_id: &str) -> Result<(), String> {
        let mut watchers = self.watchers.write().map_err(|e| e.to_string())?;

        if watchers.remove(watcher_id).is_none() {
            return Err(format!("File watcher not found: {}", watcher_id));
        }

        println!("[FileSystem] Disposed watcher: {}", watcher_id);

        // Emit event
        if let Err(e) = self
            .app_handle
            .emit("file-watcher-disposed", watcher_id)
        {
            eprintln!("[FileSystem] Failed to emit watcher disposed event: {}", e);
        }

        Ok(())
    }

    /// Get file watcher by ID
    pub fn get_file_watcher(&self, watcher_id: &str) -> Result<FileWatcher, String> {
        let watchers = self.watchers.read().map_err(|e| e.to_string())?;

        watchers
            .get(watcher_id)
            .cloned()
            .ok_or_else(|| format!("File watcher not found: {}", watcher_id))
    }

    /// Get all file watchers
    pub fn get_all_file_watchers(&self) -> Vec<FileWatcher> {
        self.watchers.read().unwrap().values().cloned().collect()
    }

    /// Emit file change event
    pub fn emit_file_change_event(&self, event: FileChangeEvent) -> Result<(), String> {
        let watchers = self.watchers.read().map_err(|e| e.to_string())?;

        // Find matching watchers
        for watcher in watchers.values() {
            // Check if event should be ignored
            let should_ignore = match event.change_type {
                FileChangeType::Created => watcher.ignore_create_events,
                FileChangeType::Changed => watcher.ignore_change_events,
                FileChangeType::Deleted => watcher.ignore_delete_events,
            };

            if should_ignore {
                continue;
            }

            // Check if URI matches glob pattern
            if self.matches_glob_pattern(&event.uri, &watcher.glob_pattern) {
                // Emit event for this watcher
                if let Err(e) = self.app_handle.emit(
                    &format!("file-watcher-event:{}", watcher.id),
                    &event,
                ) {
                    eprintln!("[FileSystem] Failed to emit file change event: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Check if URI matches glob pattern
    fn matches_glob_pattern(&self, uri: &str, pattern: &str) -> bool {
        // Simple glob matching (can be enhanced with globset crate)
        if pattern == "**/*" || pattern == "*" {
            return true;
        }

        // Remove file:// prefix
        let path = uri.trim_start_matches("file://");

        // Use glob crate for matching
        if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
            return glob_pattern.matches(path);
        }

        false
    }

    /// Clear all file system data for an owner
    pub fn clear_filesystem_data(&self, owner: &str) {
        // Clear providers
        {
            let mut providers = self.providers.write().unwrap();
            let before = providers.len();
            providers.retain(|p| p.owner != owner);
            let removed = before - providers.len();
            if removed > 0 {
                println!(
                    "[FileSystem] Cleared {} provider(s) for owner: {}",
                    removed, owner
                );
            }
        }

        // Clear watchers
        {
            let mut watchers = self.watchers.write().unwrap();
            let before = watchers.len();
            watchers.retain(|_, w| w.owner != owner);
            let removed = before - watchers.len();
            if removed > 0 {
                println!(
                    "[FileSystem] Cleared {} watcher(s) for owner: {}",
                    removed, owner
                );
            }
        }
    }
}
