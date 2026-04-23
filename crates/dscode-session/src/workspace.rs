//! Workspace management types and operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents a workspace folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    pub uri: String,
    pub name: String,
    pub index: usize,
}

/// Workspace configuration section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfiguration {
    pub section: String,
    pub scope: Option<String>,
    pub values: HashMap<String, serde_json::Value>,
}

/// File decoration data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDecoration {
    pub uri: String,
    pub badge: Option<String>,
    pub tooltip: Option<String>,
    pub color: Option<String>,
    pub propagate: bool,
}

/// A file decoration provider registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDecorationProvider {
    pub id: String,
    pub owner: String,
}

/// Options for finding files in the workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindFilesOptions {
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub max_results: Option<usize>,
    pub follow_symlinks: bool,
}

/// Options for text search in the workspace.
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

/// A text search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSearchResult {
    pub uri: String,
    pub matches: Vec<TextSearchMatch>,
}

/// A single match within a text search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSearchMatch {
    pub line: usize,
    pub character: usize,
    pub length: usize,
    pub line_text: String,
}

/// Manages workspace folders, configurations, and file decorations.
pub struct WorkspaceManager {
    pub(crate) workspace_configurations: Arc<RwLock<HashMap<String, WorkspaceConfiguration>>>,
    pub(crate) file_decoration_providers: Arc<RwLock<Vec<FileDecorationProvider>>>,
    pub(crate) file_decorations: Arc<RwLock<HashMap<String, Vec<FileDecoration>>>>,
}

impl WorkspaceManager {
    /// Create a new workspace manager.
    pub fn new() -> Self {
        Self {
            workspace_configurations: Arc::new(RwLock::new(HashMap::new())),
            file_decoration_providers: Arc::new(RwLock::new(Vec::new())),
            file_decorations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a workspace configuration section.
    pub async fn get_workspace_configuration(
        &self,
        section: &str,
        scope: Option<&str>,
    ) -> Option<WorkspaceConfiguration> {
        let key = workspace_config_key(section, scope);
        self.workspace_configurations
            .read()
            .await
            .get(&key)
            .cloned()
    }

    /// Update a workspace configuration value.
    pub async fn update_workspace_configuration(
        &self,
        section: String,
        scope: Option<String>,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), String> {
        {
            let mut configs = self.workspace_configurations.write().await;
            let config_key = workspace_config_key(&section, scope.as_deref());

            let config = configs
                .entry(config_key)
                .or_insert_with(|| WorkspaceConfiguration {
                    section: section.clone(),
                    scope: scope.clone(),
                    values: HashMap::new(),
                });

            config.values.insert(key.clone(), value);
        }

        Ok(())
    }

    /// Register a file decoration provider.
    pub async fn register_file_decoration_provider(
        &self,
        provider: FileDecorationProvider,
    ) -> Result<String, String> {
        let id = provider.id.clone();
        let mut providers = self.file_decoration_providers.write().await;
        providers.push(provider);
        Ok(id)
    }

    /// Update file decorations for a provider.
    pub async fn update_file_decorations(
        &self,
        provider_id: String,
        decorations: Vec<FileDecoration>,
    ) -> Result<(), String> {
        {
            let mut all_decorations = self.file_decorations.write().await;
            all_decorations.insert(provider_id.clone(), decorations);
        }

        Ok(())
    }

    /// Get file decorations for a URI.
    pub async fn get_file_decorations(&self, uri: &str) -> Vec<FileDecoration> {
        let all_decorations = self.file_decorations.read().await;
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

    /// Clear all workspace data for a given owner.
    pub async fn clear_workspace_data(&self, owner: &str) -> Result<(), String> {
        {
            let mut providers = self.file_decoration_providers.write().await;
            providers.retain(|provider| provider.owner != owner);
        }

        {
            let provider_ids: Vec<String> = self
                .file_decoration_providers
                .read()
                .await
                .iter()
                .map(|provider| provider.id.clone())
                .collect();

            let mut decorations = self.file_decorations.write().await;
            decorations.retain(|provider_id, _| provider_ids.contains(provider_id));
        }

        Ok(())
    }
}

fn workspace_config_key(section: &str, scope: Option<&str>) -> String {
    match scope {
        Some(scope) => format!("{}:{}", section, scope),
        None => section.to_string(),
    }
}

/// Convert a file URI to a path.
pub fn workspace_path_from_uri(uri: &str) -> PathBuf {
    PathBuf::from(uri.trim_start_matches("file://"))
}

/// Convert a path to a file URI.
pub fn workspace_uri_from_path(path: &std::path::Path) -> String {
    format!("file://{}", path.to_string_lossy())
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_workspace_folder_serde_roundtrip() {
        let folder = WorkspaceFolder {
            uri: "file:///home/user/project".to_string(),
            name: "project".to_string(),
            index: 0,
        };
        let json = serde_json::to_string(&folder).unwrap();
        let deserialized: WorkspaceFolder = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.uri, folder.uri);
        assert_eq!(deserialized.name, folder.name);
        assert_eq!(deserialized.index, folder.index);
    }

    #[test]
    fn test_workspace_folder_path_handling() {
        let uri = "file:///home/user/myproject";
        let path = workspace_path_from_uri(uri);
        assert_eq!(path, PathBuf::from("/home/user/myproject"));
    }

    #[test]
    fn test_workspace_folder_path_handling_with_file_uri() {
        let path = workspace_path_from_uri("file:///Users/test/code");
        assert_eq!(path, PathBuf::from("/Users/test/code"));
    }

    #[test]
    fn test_workspace_uri_from_path() {
        let path = PathBuf::from("/home/user/project");
        let uri = workspace_uri_from_path(&path);
        assert_eq!(uri, "file:///home/user/project");
    }

    #[test]
    fn test_workspace_uri_path_roundtrip() {
        let original = "/home/user/project";
        let uri = workspace_uri_from_path(Path::new(original));
        let back = workspace_path_from_uri(&uri);
        assert_eq!(back, PathBuf::from(original));
    }

    #[test]
    fn test_workspace_configuration_serde_roundtrip() {
        let mut values = HashMap::new();
        values.insert("fontSize".to_string(), serde_json::json!(14));
        values.insert("tabSize".to_string(), serde_json::json!(4));

        let config = WorkspaceConfiguration {
            section: "editor".to_string(),
            scope: Some("workspace".to_string()),
            values: values.clone(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: WorkspaceConfiguration = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.section, config.section);
        assert_eq!(deserialized.scope, config.scope);
        assert_eq!(deserialized.values, config.values);
    }

    #[test]
    fn test_workspace_configuration_default_values() {
        let config = WorkspaceConfiguration {
            section: "test".to_string(),
            scope: None,
            values: HashMap::new(),
        };
        assert_eq!(config.section, "test");
        assert!(config.scope.is_none());
        assert!(config.values.is_empty());
    }

    #[test]
    fn test_file_decoration_serde_roundtrip() {
        let deco = FileDecoration {
            uri: "file:///test.rs".to_string(),
            badge: Some("E".to_string()),
            tooltip: Some("Error found".to_string()),
            color: Some("red".to_string()),
            propagate: true,
        };
        let json = serde_json::to_string(&deco).unwrap();
        let deserialized: FileDecoration = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.uri, deco.uri);
        assert_eq!(deserialized.badge, deco.badge);
        assert_eq!(deserialized.tooltip, deco.tooltip);
        assert_eq!(deserialized.color, deco.color);
        assert_eq!(deserialized.propagate, deco.propagate);
    }

    #[test]
    fn test_find_files_options_serde_roundtrip() {
        let opts = FindFilesOptions {
            include: Some("**/*.rs".to_string()),
            exclude: Some("target/**".to_string()),
            max_results: Some(100),
            follow_symlinks: false,
        };
        let json = serde_json::to_string(&opts).unwrap();
        let deserialized: FindFilesOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.include, opts.include);
        assert_eq!(deserialized.exclude, opts.exclude);
        assert_eq!(deserialized.max_results, opts.max_results);
        assert_eq!(deserialized.follow_symlinks, opts.follow_symlinks);
    }

    #[test]
    fn test_text_search_options_serde_roundtrip() {
        let opts = TextSearchOptions {
            pattern: "fn main".to_string(),
            is_regex: false,
            is_case_sensitive: true,
            is_word_match: false,
            include: Some("**/*.rs".to_string()),
            exclude: None,
            max_results: Some(50),
        };
        let json = serde_json::to_string(&opts).unwrap();
        let deserialized: TextSearchOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.pattern, opts.pattern);
        assert_eq!(deserialized.is_regex, opts.is_regex);
        assert_eq!(deserialized.is_case_sensitive, opts.is_case_sensitive);
    }
}
