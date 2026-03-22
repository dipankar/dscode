use tauri::State;
use crate::commands::{
    WorkspaceRegistry, WorkspaceFolder, WorkspaceConfiguration,
    FileDecoration, FileDecorationProvider, FindFilesOptions,
    TextSearchOptions, TextSearchResult, TextSearchMatch,
};
use std::path::PathBuf;
use walkdir::WalkDir;
use regex::Regex;
use std::fs;

/// Get all workspace folders
#[tauri::command]
pub async fn get_workspace_folders(
    registry: State<'_, WorkspaceRegistry>,
) -> Result<Vec<WorkspaceFolder>, String> {
    Ok(registry.get_workspace_folders())
}

/// Add a workspace folder
#[tauri::command]
pub async fn workspace_add_folder(
    folder: WorkspaceFolder,
    registry: State<'_, WorkspaceRegistry>,
) -> Result<(), String> {
    registry.add_workspace_folder(folder)
}

/// Remove a workspace folder
#[tauri::command]
pub async fn workspace_remove_folder(
    uri: String,
    registry: State<'_, WorkspaceRegistry>,
) -> Result<(), String> {
    registry.remove_workspace_folder(&uri)
}

/// Get workspace configuration
#[tauri::command]
pub async fn get_workspace_configuration(
    section: String,
    scope: Option<String>,
    registry: State<'_, WorkspaceRegistry>,
) -> Result<Option<WorkspaceConfiguration>, String> {
    Ok(registry.get_configuration(&section, scope.as_deref()))
}

/// Update workspace configuration
#[tauri::command]
pub async fn update_workspace_configuration(
    section: String,
    scope: Option<String>,
    key: String,
    value: serde_json::Value,
    registry: State<'_, WorkspaceRegistry>,
) -> Result<(), String> {
    registry.update_configuration(section, scope, key, value)
}

/// Register a file decoration provider
#[tauri::command]
pub async fn register_file_decoration_provider(
    provider: FileDecorationProvider,
    registry: State<'_, WorkspaceRegistry>,
) -> Result<String, String> {
    registry.register_file_decoration_provider(provider)
}

/// Update file decorations
#[tauri::command]
pub async fn update_file_decorations(
    provider_id: String,
    decorations: Vec<FileDecoration>,
    registry: State<'_, WorkspaceRegistry>,
) -> Result<(), String> {
    registry.update_file_decorations(provider_id, decorations)
}

/// Get file decorations for a URI
#[tauri::command]
pub async fn get_file_decorations(
    uri: String,
    registry: State<'_, WorkspaceRegistry>,
) -> Result<Vec<FileDecoration>, String> {
    Ok(registry.get_file_decorations(&uri))
}

/// Find files in workspace
#[tauri::command]
pub async fn workspace_find_files(
    options: FindFilesOptions,
    folders: Vec<WorkspaceFolder>,
) -> Result<Vec<String>, String> {
    let mut results = Vec::new();
    let max_results = options.max_results.unwrap_or(10000);

    // Build include pattern
    let include_pattern = options.include.as_ref().map(|p| {
        glob::Pattern::new(p).map_err(|e| format!("Invalid include pattern: {}", e))
    }).transpose()?;

    // Build exclude pattern
    let exclude_pattern = options.exclude.as_ref().map(|p| {
        glob::Pattern::new(p).map_err(|e| format!("Invalid exclude pattern: {}", e))
    }).transpose()?;

    for folder in folders {
        // Convert URI to path
        let path = PathBuf::from(folder.uri.trim_start_matches("file://"));

        if !path.exists() {
            continue;
        }

        let walker = WalkDir::new(&path)
            .follow_links(options.follow_symlinks)
            .into_iter()
            .filter_entry(|e| {
                // Skip hidden files/folders unless explicitly included
                let file_name = e.file_name().to_string_lossy();
                !file_name.starts_with('.')
            });

        for entry in walker {
            if results.len() >= max_results {
                break;
            }

            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            let path_str = path.to_string_lossy();

            // Check include pattern
            if let Some(ref pattern) = include_pattern {
                if !pattern.matches(&path_str) {
                    continue;
                }
            }

            // Check exclude pattern
            if let Some(ref pattern) = exclude_pattern {
                if pattern.matches(&path_str) {
                    continue;
                }
            }

            results.push(format!("file://{}", path_str));
        }
    }

    Ok(results)
}

/// Search for text in workspace files
#[tauri::command]
pub async fn workspace_find_text(
    options: TextSearchOptions,
    folders: Vec<WorkspaceFolder>,
) -> Result<Vec<TextSearchResult>, String> {
    let mut results = Vec::new();
    let max_results = options.max_results.unwrap_or(10000);
    let mut match_count = 0;

    // Build search pattern
    let pattern = if options.is_regex {
        Regex::new(&options.pattern).map_err(|e| format!("Invalid regex pattern: {}", e))?
    } else {
        let escaped = regex::escape(&options.pattern);
        let pattern_str = if options.is_word_match {
            format!(r"\b{}\b", escaped)
        } else {
            escaped
        };

        let regex_str = if options.is_case_sensitive {
            pattern_str
        } else {
            format!("(?i){}", pattern_str)
        };

        Regex::new(&regex_str).map_err(|e| format!("Failed to create regex: {}", e))?
    };

    // Build include pattern
    let include_pattern = options.include.as_ref().map(|p| {
        glob::Pattern::new(p).map_err(|e| format!("Invalid include pattern: {}", e))
    }).transpose()?;

    // Build exclude pattern
    let exclude_pattern = options.exclude.as_ref().map(|p| {
        glob::Pattern::new(p).map_err(|e| format!("Invalid exclude pattern: {}", e))
    }).transpose()?;

    for folder in folders {
        if match_count >= max_results {
            break;
        }

        // Convert URI to path
        let path = PathBuf::from(folder.uri.trim_start_matches("file://"));

        if !path.exists() {
            continue;
        }

        let walker = WalkDir::new(&path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let file_name = e.file_name().to_string_lossy();
                !file_name.starts_with('.')
            });

        for entry in walker {
            if match_count >= max_results {
                break;
            }

            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let entry_path = entry.path();

            // Skip directories
            if entry_path.is_dir() {
                continue;
            }

            let path_str = entry_path.to_string_lossy();

            // Check include pattern
            if let Some(ref pattern) = include_pattern {
                if !pattern.matches(&path_str) {
                    continue;
                }
            }

            // Check exclude pattern
            if let Some(ref pattern) = exclude_pattern {
                if pattern.matches(&path_str) {
                    continue;
                }
            }

            // Read file content
            let content = match fs::read_to_string(entry_path) {
                Ok(c) => c,
                Err(_) => continue, // Skip binary files
            };

            let mut file_matches = Vec::new();

            // Search in file
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
                    uri: format!("file://{}", path_str),
                    matches: file_matches,
                });
            }
        }
    }

    Ok(results)
}

/// Clear workspace data for an owner
#[tauri::command]
pub async fn clear_workspace_data(
    owner: String,
    registry: State<'_, WorkspaceRegistry>,
) -> Result<(), String> {
    registry.clear_owner_data(&owner)
}
