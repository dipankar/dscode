use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub node_type: String,
    pub children: Option<Vec<FileNode>>,
}

#[tauri::command]
pub async fn read_file(path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read file {}: {}", path, e))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn write_file(path: String, content: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write file {}: {}", path, e))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn list_directory(path: String) -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(move || {
        let entries = fs::read_dir(&path)
            .map_err(|e| format!("Failed to read directory {}: {}", path, e))?;

        let mut files = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(name) = entry.file_name().to_str() {
                    files.push(name.to_string());
                }
            }
        }

        Ok(files)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn read_directory_tree(path: String, max_depth: Option<u32>) -> Result<Vec<FileNode>, String> {
    tokio::task::spawn_blocking(move || {
        let max_depth = max_depth.unwrap_or(3);
        read_dir_recursive(&path, 0, max_depth)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

fn read_dir_recursive(path: &str, current_depth: u32, max_depth: u32) -> Result<Vec<FileNode>, String> {
    if current_depth >= max_depth {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory {}: {}", path, e))?;

    let mut nodes = Vec::new();

    for entry in entries {
        if let Ok(entry) = entry {
            let entry_path = entry.path();
            let path_str = entry_path.to_string_lossy().to_string();

            // Skip hidden files and common ignore patterns
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
                        Some(read_dir_recursive(&path_str, current_depth + 1, max_depth).unwrap_or_default())
                    } else {
                        None
                    },
                };

                nodes.push(node);
            }
        }
    }

    // Sort: directories first, then files, both alphabetically
    nodes.sort_by(|a, b| {
        match (a.node_type.as_str(), b.node_type.as_str()) {
            ("directory", "file") => std::cmp::Ordering::Less,
            ("file", "directory") => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(nodes)
}

#[tauri::command]
pub async fn get_file_language(path: String) -> Result<String, String> {
    let extension = Path::new(&path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

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
pub async fn create_file(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        // Check if file already exists
        if Path::new(&path).exists() {
            return Err(format!("File already exists: {}", path));
        }

        // Create parent directories if they don't exist
        if let Some(parent) = Path::new(&path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directories: {}", e))?;
        }

        // Create empty file
        fs::write(&path, "")
            .map_err(|e| format!("Failed to create file {}: {}", path, e))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn create_folder(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        // Check if folder already exists
        if Path::new(&path).exists() {
            return Err(format!("Folder already exists: {}", path));
        }

        fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create folder {}: {}", path, e))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn delete_file(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        if !Path::new(&path).exists() {
            return Err(format!("File does not exist: {}", path));
        }

        if Path::new(&path).is_dir() {
            return Err(format!("Path is a directory, not a file: {}", path));
        }

        fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete file {}: {}", path, e))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn delete_folder(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        if !Path::new(&path).exists() {
            return Err(format!("Folder does not exist: {}", path));
        }

        if !Path::new(&path).is_dir() {
            return Err(format!("Path is not a directory: {}", path));
        }

        fs::remove_dir_all(&path)
            .map_err(|e| format!("Failed to delete folder {}: {}", path, e))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn rename_path(old_path: String, new_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        if !Path::new(&old_path).exists() {
            return Err(format!("Path does not exist: {}", old_path));
        }

        if Path::new(&new_path).exists() {
            return Err(format!("Destination path already exists: {}", new_path));
        }

        fs::rename(&old_path, &new_path)
            .map_err(|e| format!("Failed to rename {} to {}: {}", old_path, new_path, e))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}
