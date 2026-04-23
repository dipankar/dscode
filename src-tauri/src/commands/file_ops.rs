use dscode_core::CoreError;
use dscode_extension_host::PathValidator;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub node_type: String,
    pub children: Option<Vec<FileNode>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub name: String,
    pub path: String,
    pub node_type: String,
}

fn validate_path_sync(
    path: &str, validator: &PathValidator,
) -> Result<std::path::PathBuf, CoreError> {
    validator.validate_file_path(path).map_err(CoreError::PathResolution)
}

#[tauri::command]
pub async fn read_file(
    path: String, path_validator: tauri::State<'_, Arc<RwLock<PathValidator>>>,
) -> Result<String, String> {
    let validator = path_validator.read().await;
    let validated = validate_path_sync(&path, &validator).map_err(|e| e.to_string())?;
    let path_str = validated.to_string_lossy().to_string();
    drop(validator);
    tokio::task::spawn_blocking(move || {
        fs::read_to_string(&path_str).map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn write_file(
    path: String, content: String, path_validator: tauri::State<'_, Arc<RwLock<PathValidator>>>,
) -> Result<(), String> {
    let validator = path_validator.read().await;
    let validated = validate_path_sync(&path, &validator).map_err(|e| e.to_string())?;
    let path_str = validated.to_string_lossy().to_string();
    drop(validator);
    tokio::task::spawn_blocking(move || {
        fs::write(&path_str, content).map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn list_directory(
    path: String, path_validator: tauri::State<'_, Arc<RwLock<PathValidator>>>,
) -> Result<Vec<String>, String> {
    let validator = path_validator.read().await;
    let validated = validate_path_sync(&path, &validator).map_err(|e| e.to_string())?;
    let path_str = validated.to_string_lossy().to_string();
    drop(validator);
    tokio::task::spawn_blocking(move || {
        let entries = fs::read_dir(&path_str)
            .map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))?;
        let mut files = Vec::new();
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                files.push(name.to_string());
            }
        }
        Ok(files)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn read_directory_tree(
    path: String, max_depth: Option<u32>,
    path_validator: tauri::State<'_, Arc<RwLock<PathValidator>>>,
) -> Result<Vec<FileNode>, String> {
    let validator = path_validator.read().await;
    let validated = validate_path_sync(&path, &validator).map_err(|e| e.to_string())?;
    let path_str = validated.to_string_lossy().to_string();
    drop(validator);
    tokio::task::spawn_blocking(move || {
        let max_depth = max_depth.unwrap_or(3);
        read_dir_recursive(&path_str, 0, max_depth).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn read_directory(
    path: String, path_validator: tauri::State<'_, Arc<RwLock<PathValidator>>>,
) -> Result<Vec<FileNode>, String> {
    let validator = path_validator.read().await;
    let validated = validate_path_sync(&path, &validator).map_err(|e| e.to_string())?;
    let path_str = validated.to_string_lossy().to_string();
    drop(validator);
    tokio::task::spawn_blocking(move || {
        let entries = fs::read_dir(&path_str)
            .map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))?;
        let mut nodes = Vec::new();
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let path_str_inner = entry_path.to_string_lossy().to_string();
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with('.') || name == "node_modules" || name == "target" {
                    continue;
                }
                let is_dir = entry_path.is_dir();
                let node = FileNode {
                    name: name.to_string(),
                    path: path_str_inner,
                    node_type: if is_dir { "directory".to_string() } else { "file".to_string() },
                    children: None,
                };
                nodes.push(node);
            }
        }
        nodes.sort_by(|a, b| match (a.node_type.as_str(), b.node_type.as_str()) {
            ("directory", "file") => std::cmp::Ordering::Less,
            ("file", "directory") => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
        Ok(nodes)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

fn read_dir_recursive(
    path: &str, current_depth: u32, max_depth: u32,
) -> Result<Vec<FileNode>, CoreError> {
    if current_depth >= max_depth {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(path).map_err(CoreError::from)?;

    let mut nodes = Vec::new();

    for entry in entries.flatten() {
        let entry_path = entry.path();
        let path_str = entry_path.to_string_lossy().to_string();

        if let Some(name) = entry.file_name().to_str() {
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }

            let is_dir = entry_path.is_dir();
            let node = FileNode {
                name: name.to_string(),
                path: path_str.clone(),
                node_type: if is_dir { "directory".to_string() } else { "file".to_string() },
                children: if is_dir {
                    Some(
                        read_dir_recursive(&path_str, current_depth + 1, max_depth)
                            .unwrap_or_default(),
                    )
                } else {
                    None
                },
            };

            nodes.push(node);
        }
    }

    nodes.sort_by(|a, b| match (a.node_type.as_str(), b.node_type.as_str()) {
        ("directory", "file") => std::cmp::Ordering::Less,
        ("file", "directory") => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(nodes)
}

#[tauri::command]
pub async fn get_file_language(path: String) -> Result<String, String> {
    let extension = Path::new(&path).extension().and_then(|ext| ext.to_str()).unwrap_or("");

    let language = match extension {
        "rs" => "rust",
        "js" | "mjs" => "javascript",
        "ts" => "typescript",
        "jsx" => "javascript",
        "tsx" => "typescript",
        "py" => "python",
        "go" => "go",
        "java" => "java",
        "c" => "c",
        "cpp" | "cc" | "cxx" => "cpp",
        "h" | "hpp" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "kt" => "kotlin",
        "scala" => "scala",
        "sh" | "bash" => "shell",
        "json" => "json",
        "xml" => "xml",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" | "sass" => "scss",
        "md" | "markdown" => "markdown",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "sql" => "sql",
        _ => "plaintext",
    };

    Ok(language.to_string())
}

#[tauri::command]
pub async fn start_watching_file(app_handle: tauri::AppHandle, path: String) -> Result<(), String> {
    crate::watcher::watch_file(app_handle, path)
}

#[tauri::command]
pub async fn stop_watching_file(app_handle: tauri::AppHandle, path: String) -> Result<(), String> {
    crate::watcher::unwatch_file(app_handle, path)
}

#[tauri::command]
pub async fn create_file(
    path: String, path_validator: tauri::State<'_, Arc<RwLock<PathValidator>>>,
) -> Result<(), String> {
    let validator = path_validator.read().await;
    let validated = validate_path_sync(&path, &validator).map_err(|e| e.to_string())?;
    let path_str = validated.to_string_lossy().to_string();
    drop(validator);
    tokio::task::spawn_blocking(move || {
        if Path::new(&path_str).exists() {
            return Err("File already exists".to_string());
        }
        if let Some(parent) = Path::new(&path_str).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))?;
        }
        fs::write(&path_str, "").map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn create_folder(
    path: String, path_validator: tauri::State<'_, Arc<RwLock<PathValidator>>>,
) -> Result<(), String> {
    let validator = path_validator.read().await;
    let validated = validate_path_sync(&path, &validator).map_err(|e| e.to_string())?;
    let path_str = validated.to_string_lossy().to_string();
    drop(validator);
    tokio::task::spawn_blocking(move || {
        if Path::new(&path_str).exists() {
            return Err("Folder already exists".to_string());
        }
        fs::create_dir_all(&path_str).map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn delete_file(
    path: String, path_validator: tauri::State<'_, Arc<RwLock<PathValidator>>>,
) -> Result<(), String> {
    let validator = path_validator.read().await;
    let validated = validate_path_sync(&path, &validator).map_err(|e| e.to_string())?;
    let path_str = validated.to_string_lossy().to_string();
    drop(validator);
    tokio::task::spawn_blocking(move || {
        if !Path::new(&path_str).exists() {
            return Err("File does not exist".to_string());
        }
        if Path::new(&path_str).is_dir() {
            return Err("Path is a directory, not a file".to_string());
        }
        fs::remove_file(&path_str).map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn delete_folder(
    path: String, path_validator: tauri::State<'_, Arc<RwLock<PathValidator>>>,
) -> Result<(), String> {
    let validator = path_validator.read().await;
    let validated = validate_path_sync(&path, &validator).map_err(|e| e.to_string())?;
    let path_str = validated.to_string_lossy().to_string();
    drop(validator);
    tokio::task::spawn_blocking(move || {
        if !Path::new(&path_str).exists() {
            return Err("Folder does not exist".to_string());
        }
        if !Path::new(&path_str).is_dir() {
            return Err("Path is not a directory".to_string());
        }
        fs::remove_dir_all(&path_str).map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn rename_path(
    old_path: String, new_path: String,
    path_validator: tauri::State<'_, Arc<RwLock<PathValidator>>>,
) -> Result<(), String> {
    let validator = path_validator.read().await;
    let validated_old = validate_path_sync(&old_path, &validator).map_err(|e| e.to_string())?;
    let validated_new = validate_path_sync(&new_path, &validator).map_err(|e| e.to_string())?;
    let old_str = validated_old.to_string_lossy().to_string();
    let new_str = validated_new.to_string_lossy().to_string();
    drop(validator);
    tokio::task::spawn_blocking(move || {
        if !Path::new(&old_str).exists() {
            return Err("Path does not exist".to_string());
        }
        if Path::new(&new_str).exists() {
            return Err("Destination path already exists".to_string());
        }
        fs::rename(&old_str, &new_str).map_err(|e| PathValidator::sanitize_error(&format!("{}", e)))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn search_files(
    query: String, root_path: String, max_results: Option<usize>,
) -> Result<Vec<FileSearchResult>, String> {
    let max = max_results.unwrap_or(100);
    let query_lower = query.to_lowercase();
    tokio::task::spawn_blocking(move || {
        let mut results = Vec::new();

        fn walk_dir(
            dir: &Path, query_lower: &str, results: &mut Vec<FileSearchResult>, max: usize,
        ) {
            if results.len() >= max {
                return;
            }

            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if results.len() >= max {
                        return;
                    }

                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') || name == "node_modules" || name == "target" {
                        continue;
                    }

                    let path = entry.path().to_string_lossy().to_string();
                    let is_dir = entry.path().is_dir();

                    if name.to_lowercase().contains(query_lower) {
                        results.push(FileSearchResult {
                            name,
                            path,
                            node_type: if is_dir {
                                "directory".to_string()
                            } else {
                                "file".to_string()
                            },
                        });
                    }

                    if is_dir {
                        walk_dir(&entry.path(), query_lower, results, max);
                    }
                }
            }
        }

        walk_dir(Path::new(&root_path), &query_lower, &mut results, max);
        Ok(results)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}
