use super::{SessionEvent, SessionManager};
use crate::commands::{
    FileDecoration, FileDecorationProvider, FindFilesOptions, TextSearchMatch, TextSearchOptions,
    TextSearchResult, WorkspaceConfiguration, WorkspaceFolder,
};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Emitter;

impl SessionManager {
    pub async fn get_workspace_folders_detailed(&self) -> Vec<WorkspaceFolder> {
        self.state
            .read()
            .await
            .workspace_folders
            .iter()
            .enumerate()
            .map(|(index, path)| WorkspaceFolder {
                uri: Self::workspace_uri_from_path(path),
                name: path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string()),
                index,
            })
            .collect()
    }

    pub async fn add_workspace_folder_from_uri(&self, uri: &str) -> Result<(), String> {
        let path = Self::workspace_path_from_uri(uri);
        self.add_workspace_folder(path).await
    }

    pub async fn remove_workspace_folder_by_uri(&self, uri: &str) -> Result<(), String> {
        let path = Self::workspace_path_from_uri(uri);
        self.remove_workspace_folder(&path).await
    }

    pub async fn add_workspace_folder(&self, path: PathBuf) -> Result<(), String> {
        let state_clone = {
            let mut state = self.state.write().await;

            if state.workspace_folders.contains(&path) {
                return Ok(());
            }

            state.workspace_folders.push(path.clone());
            state.clone()
        };

        // Register the workspace folder with the path validator
        {
            let mut validator = self.path_validator.write().await;
            validator.add_workspace_folder(path.clone());
        }

        self.emit_event(SessionEvent::WorkspaceFolderAdded { path });
        self.emit_event(SessionEvent::StateChanged { state: state_clone });

        Ok(())
    }

    pub async fn remove_workspace_folder(&self, path: &PathBuf) -> Result<(), String> {
        let removed = {
            let state = self.state.read().await;
            state.workspace_folders.contains(path)
        };

        if !removed {
            return Ok(());
        }

        let state_clone = {
            let mut state = self.state.write().await;
            state.workspace_folders.retain(|p| p != path);
            state.clone()
        };

        // Note: PathValidator doesn't support unregistration of individual paths.
        // This is acceptable since removed workspace folders are no longer accessible.

        self.emit_event(SessionEvent::WorkspaceFolderRemoved { path: path.clone() });
        self.emit_event(SessionEvent::StateChanged { state: state_clone });

        Ok(())
    }

    pub async fn get_workspace_configuration(
        &self, section: &str, scope: Option<&str>,
    ) -> Option<WorkspaceConfiguration> {
        let key = Self::workspace_config_key(section, scope);
        self.workspace_configurations.read().await.get(&key).cloned()
    }

    pub async fn update_workspace_configuration(
        &self, section: String, scope: Option<String>, key: String, value: Value,
    ) -> Result<(), String> {
        {
            let mut configs = self.workspace_configurations.write().await;
            let config_key = Self::workspace_config_key(&section, scope.as_deref());

            let config = configs.entry(config_key).or_insert_with(|| WorkspaceConfiguration {
                section: section.clone(),
                scope: scope.clone(),
                values: HashMap::new(),
            });

            config.values.insert(key.clone(), value);
        }

        self.emit_event(SessionEvent::ConfigurationChanged {
            section: Some(section.clone()),
            key: Some(key.clone()),
        });

        if let Err(err) =
            self.app_handle.emit("configuration-changed", &(section.clone(), scope.clone()))
        {
            eprintln!("[SessionManager] Failed to emit workspace configuration change: {}", err);
        }

        Ok(())
    }

    pub async fn register_file_decoration_provider(
        &self, provider: FileDecorationProvider,
    ) -> Result<String, String> {
        let id = provider.id.clone();
        let mut providers = self.file_decoration_providers.write().await;
        providers.push(provider);
        Ok(id)
    }

    pub async fn update_file_decorations(
        &self, provider_id: String, decorations: Vec<FileDecoration>,
    ) -> Result<(), String> {
        {
            let mut all_decorations = self.file_decorations.write().await;
            all_decorations.insert(provider_id.clone(), decorations);
        }

        if let Err(err) = self.app_handle.emit("file-decorations-changed", &provider_id) {
            eprintln!("[SessionManager] Failed to emit file decorations change: {}", err);
        }

        Ok(())
    }

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

    pub async fn find_workspace_files_with_options(
        &self, options: &FindFilesOptions,
    ) -> Result<Vec<String>, String> {
        let include = options.include.as_deref().unwrap_or("**/*");
        let include_patterns = Self::split_patterns(include, "**/*");
        let exclude_patterns = Self::split_patterns_opt(options.exclude.as_deref());
        let max_results = options.max_results.unwrap_or(10000);

        self.find_workspace_files(&include_patterns, &exclude_patterns, max_results).await.map(
            |paths| {
                paths
                    .into_iter()
                    .map(|path| Self::workspace_uri_from_path(Path::new(&path)))
                    .collect()
            },
        )
    }

    pub async fn find_workspace_text(
        &self, options: &TextSearchOptions,
    ) -> Result<Vec<TextSearchResult>, String> {
        let mut results = Vec::new();
        let max_results = options.max_results.unwrap_or(10000);
        let mut match_count = 0;

        let pattern = if options.is_regex {
            Regex::new(&options.pattern).map_err(|e| format!("Invalid regex pattern: {}", e))?
        } else {
            let escaped = regex::escape(&options.pattern);
            let pattern_str =
                if options.is_word_match { format!(r"\b{}\b", escaped) } else { escaped };

            let regex_str = if options.is_case_sensitive {
                pattern_str
            } else {
                format!("(?i){}", pattern_str)
            };

            Regex::new(&regex_str).map_err(|e| format!("Failed to create regex: {}", e))?
        };

        let include = options.include.as_deref().unwrap_or("**/*");
        let include_patterns = Self::split_patterns(include, "**/*");
        let exclude_patterns = Self::split_patterns_opt(options.exclude.as_deref());
        let files =
            self.find_workspace_files(&include_patterns, &exclude_patterns, max_results).await?;

        for file in files {
            if match_count >= max_results {
                break;
            }

            let content = match fs::read_to_string(&file) {
                Ok(content) => content,
                Err(_) => continue,
            };

            let mut file_matches = Vec::new();

            for (line_num, line) in content.lines().enumerate() {
                for mat in pattern.find_iter(line) {
                    if match_count >= max_results {
                        break;
                    }

                    file_matches.push(TextSearchMatch {
                        line: line_num,
                        character: mat.start(),
                        length: mat.end() - mat.start(),
                        line_text: line.to_string(),
                    });

                    match_count += 1;
                }

                if match_count >= max_results {
                    break;
                }
            }

            if !file_matches.is_empty() {
                results.push(TextSearchResult {
                    uri: Self::workspace_uri_from_path(Path::new(&file)),
                    matches: file_matches,
                });
            }
        }

        Ok(results)
    }

    fn workspace_config_key(section: &str, scope: Option<&str>) -> String {
        match scope {
            Some(scope) => format!("{}:{}", section, scope),
            None => section.to_string(),
        }
    }

    fn workspace_path_from_uri(uri: &str) -> PathBuf {
        PathBuf::from(uri.trim_start_matches("file://"))
    }

    fn workspace_uri_from_path(path: &Path) -> String {
        format!("file://{}", path.to_string_lossy())
    }
}
