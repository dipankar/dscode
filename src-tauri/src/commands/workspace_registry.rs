use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    pub uri: String,
    pub name: String,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfiguration {
    pub section: String,
    pub scope: Option<String>,
    pub values: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDecoration {
    pub uri: String,
    pub badge: Option<String>,
    pub tooltip: Option<String>,
    pub color: Option<String>,
    pub propagate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDecorationProvider {
    pub id: String,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindFilesOptions {
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub max_results: Option<usize>,
    pub follow_symlinks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSearchOptions {
    pub pattern: String,
    pub is_regex: bool,
    pub is_case_sensitive: bool,
    pub is_word_match: bool,
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub max_results: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSearchResult {
    pub uri: String,
    pub matches: Vec<TextSearchMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSearchMatch {
    pub line: usize,
    pub character: usize,
    pub length: usize,
    pub line_text: String,
}

/// Registry for workspace-related features
pub struct WorkspaceRegistry {
    folders: Arc<RwLock<Vec<WorkspaceFolder>>>,
    configurations: Arc<RwLock<HashMap<String, WorkspaceConfiguration>>>,
    file_decoration_providers: Arc<RwLock<Vec<FileDecorationProvider>>>,
    file_decorations: Arc<RwLock<HashMap<String, Vec<FileDecoration>>>>, // provider_id -> decorations
    app_handle: AppHandle,
}

impl WorkspaceRegistry {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            folders: Arc::new(RwLock::new(Vec::new())),
            configurations: Arc::new(RwLock::new(HashMap::new())),
            file_decoration_providers: Arc::new(RwLock::new(Vec::new())),
            file_decorations: Arc::new(RwLock::new(HashMap::new())),
            app_handle,
        }
    }

    /// Get all workspace folders
    pub fn get_workspace_folders(&self) -> Vec<WorkspaceFolder> {
        let folders = self.folders.read().unwrap();
        folders.clone()
    }

    /// Add a workspace folder
    pub fn add_workspace_folder(&self, folder: WorkspaceFolder) -> Result<(), String> {
        {
            let mut folders = self.folders.write().map_err(|e| e.to_string())?;

            // Check if folder already exists
            if folders.iter().any(|f| f.uri == folder.uri) {
                return Err("Workspace folder already exists".to_string());
            }

            folders.push(folder.clone());
            folders.sort_by_key(|f| f.index);
        }

        // Emit event to frontend
        if let Err(e) = self.app_handle.emit("workspace-folders-changed", &self.get_workspace_folders()) {
            eprintln!("[Workspace] Failed to emit workspace folders changed event: {}", e);
        }

        println!("[Workspace] Added workspace folder: {}", folder.name);
        Ok(())
    }

    /// Remove a workspace folder
    pub fn remove_workspace_folder(&self, uri: &str) -> Result<(), String> {
        {
            let mut folders = self.folders.write().map_err(|e| e.to_string())?;
            let initial_len = folders.len();
            folders.retain(|f| f.uri != uri);

            if folders.len() == initial_len {
                return Err("Workspace folder not found".to_string());
            }

            // Reindex remaining folders
            for (i, folder) in folders.iter_mut().enumerate() {
                folder.index = i;
            }
        }

        // Emit event to frontend
        if let Err(e) = self.app_handle.emit("workspace-folders-changed", &self.get_workspace_folders()) {
            eprintln!("[Workspace] Failed to emit workspace folders changed event: {}", e);
        }

        println!("[Workspace] Removed workspace folder: {}", uri);
        Ok(())
    }

    /// Get workspace configuration
    pub fn get_configuration(&self, section: &str, scope: Option<&str>) -> Option<WorkspaceConfiguration> {
        let configs = self.configurations.read().unwrap();
        let key = self.config_key(section, scope);
        configs.get(&key).cloned()
    }

    /// Update workspace configuration
    pub fn update_configuration(
        &self,
        section: String,
        scope: Option<String>,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), String> {
        {
            let mut configs = self.configurations.write().map_err(|e| e.to_string())?;
            let config_key = self.config_key(&section, scope.as_deref());

            let config = configs.entry(config_key).or_insert_with(|| WorkspaceConfiguration {
                section: section.clone(),
                scope: scope.clone(),
                values: HashMap::new(),
            });

            config.values.insert(key.clone(), value);
        }

        // Emit event to frontend
        if let Err(e) = self.app_handle.emit("configuration-changed", &(section.clone(), scope.clone())) {
            eprintln!("[Workspace] Failed to emit configuration changed event: {}", e);
        }

        println!("[Workspace] Updated configuration: {}.{}", section, key);
        Ok(())
    }

    /// Register a file decoration provider
    pub fn register_file_decoration_provider(&self, provider: FileDecorationProvider) -> Result<String, String> {
        let mut providers = self.file_decoration_providers.write().map_err(|e| e.to_string())?;
        let id = provider.id.clone();
        providers.push(provider);

        println!("[Workspace] Registered file decoration provider: {}", id);
        Ok(id)
    }

    /// Update file decorations for a provider
    pub fn update_file_decorations(
        &self,
        provider_id: String,
        decorations: Vec<FileDecoration>,
    ) -> Result<(), String> {
        {
            let mut all_decorations = self.file_decorations.write().map_err(|e| e.to_string())?;
            all_decorations.insert(provider_id.clone(), decorations.clone());
        }

        // Emit event to frontend
        if let Err(e) = self.app_handle.emit("file-decorations-changed", &provider_id) {
            eprintln!("[Workspace] Failed to emit file decorations changed event: {}", e);
        }

        Ok(())
    }

    /// Get file decorations for a URI
    pub fn get_file_decorations(&self, uri: &str) -> Vec<FileDecoration> {
        let all_decorations = self.file_decorations.read().unwrap();
        let mut result = Vec::new();

        for decorations in all_decorations.values() {
            for decoration in decorations {
                if decoration.uri == uri {
                    result.push(decoration.clone());
                }
            }
        }

        result
    }

    /// Clear all providers and configurations for an owner
    pub fn clear_owner_data(&self, owner: &str) -> Result<(), String> {
        {
            let mut providers = self.file_decoration_providers.write().map_err(|e| e.to_string())?;
            providers.retain(|p| p.owner != owner);
        }

        {
            let mut decorations = self.file_decorations.write().map_err(|e| e.to_string())?;
            decorations.retain(|provider_id, _| {
                let providers = self.file_decoration_providers.read().unwrap();
                providers.iter().any(|p| &p.id == provider_id)
            });
        }

        println!("[Workspace] Cleared all data for owner: {}", owner);
        Ok(())
    }

    /// Helper to generate configuration key
    fn config_key(&self, section: &str, scope: Option<&str>) -> String {
        match scope {
            Some(s) => format!("{}:{}", section, s),
            None => section.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here
}
