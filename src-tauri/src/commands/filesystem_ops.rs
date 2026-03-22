use super::filesystem_registry::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tauri::State;

/// Read file as binary
#[tauri::command]
pub async fn fs_read_file(uri: String) -> Result<Vec<u8>, String> {
    let path = uri_to_path(&uri)?;

    fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))
}

/// Write file as binary
#[tauri::command]
pub async fn fs_write_file(uri: String, content: Vec<u8>) -> Result<(), String> {
    let path = uri_to_path(&uri)?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
    }

    fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))
}

/// Get file stat information
#[tauri::command]
pub async fn fs_stat(uri: String) -> Result<FileStat, String> {
    let path = uri_to_path(&uri)?;

    let metadata = fs::metadata(&path).map_err(|e| format!("Failed to get file stat: {}", e))?;

    let file_type = if metadata.is_dir() {
        FileType::Directory
    } else if metadata.is_symlink() {
        FileType::SymbolicLink
    } else if metadata.is_file() {
        FileType::File
    } else {
        FileType::Unknown
    };

    let ctime = metadata
        .created()
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mtime = metadata
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let size = metadata.len();

    #[cfg(unix)]
    let permissions = {
        use std::os::unix::fs::PermissionsExt;
        Some(metadata.permissions().mode())
    };

    #[cfg(not(unix))]
    let permissions = None;

    Ok(FileStat {
        uri,
        file_type,
        ctime,
        mtime,
        size,
        permissions,
    })
}

/// Read directory contents
#[tauri::command]
pub async fn fs_read_directory(uri: String) -> Result<Vec<DirectoryEntry>, String> {
    let path = uri_to_path(&uri)?;

    let entries = fs::read_dir(&path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    let mut result = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let metadata = entry
            .metadata()
            .map_err(|e| format!("Failed to get entry metadata: {}", e))?;

        let file_type = if metadata.is_dir() {
            FileType::Directory
        } else if metadata.is_symlink() {
            FileType::SymbolicLink
        } else if metadata.is_file() {
            FileType::File
        } else {
            FileType::Unknown
        };

        result.push(DirectoryEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            file_type,
        });
    }

    // Sort by name for consistency
    result.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(result)
}

/// Create directory
#[tauri::command]
pub async fn fs_create_directory(uri: String) -> Result<(), String> {
    let path = uri_to_path(&uri)?;

    fs::create_dir_all(&path).map_err(|e| format!("Failed to create directory: {}", e))
}

/// Delete file or directory
#[tauri::command]
pub async fn fs_delete(uri: String, recursive: bool) -> Result<(), String> {
    let path = uri_to_path(&uri)?;

    let metadata = fs::metadata(&path).map_err(|e| format!("Failed to get file metadata: {}", e))?;

    if metadata.is_dir() {
        if recursive {
            fs::remove_dir_all(&path).map_err(|e| format!("Failed to delete directory: {}", e))
        } else {
            fs::remove_dir(&path).map_err(|e| format!("Failed to delete directory: {}", e))
        }
    } else {
        fs::remove_file(&path).map_err(|e| format!("Failed to delete file: {}", e))
    }
}

/// Rename file or directory
#[tauri::command]
pub async fn fs_rename(old_uri: String, new_uri: String) -> Result<(), String> {
    let old_path = uri_to_path(&old_uri)?;
    let new_path = uri_to_path(&new_uri)?;

    // Create parent directory for new path if needed
    if let Some(parent) = new_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
    }

    fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename: {}", e))
}

/// Copy file or directory
#[tauri::command]
pub async fn fs_copy(source_uri: String, destination_uri: String) -> Result<(), String> {
    let source = uri_to_path(&source_uri)?;
    let destination = uri_to_path(&destination_uri)?;

    let metadata = fs::metadata(&source).map_err(|e| format!("Failed to get source metadata: {}", e))?;

    if metadata.is_dir() {
        copy_dir_recursive(&source, &destination)
    } else {
        // Create parent directory if needed
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directory: {}", e))?;
        }

        fs::copy(&source, &destination)
            .map_err(|e| format!("Failed to copy file: {}", e))?;
        Ok(())
    }
}

/// Register file system provider
#[tauri::command]
pub async fn register_file_system_provider(
    provider: FileSystemProvider,
    registry: State<'_, FileSystemRegistry>,
) -> Result<String, String> {
    registry.register_file_system_provider(provider)
}

/// Unregister file system provider
#[tauri::command]
pub async fn unregister_file_system_provider(
    scheme: String,
    registry: State<'_, FileSystemRegistry>,
) -> Result<(), String> {
    registry.unregister_file_system_provider(&scheme)
}

/// Get file system provider
#[tauri::command]
pub async fn get_file_system_provider(
    scheme: String,
    registry: State<'_, FileSystemRegistry>,
) -> Result<FileSystemProvider, String> {
    registry.get_file_system_provider(&scheme)
}

/// Get all file system providers
#[tauri::command]
pub async fn get_all_file_system_providers(
    registry: State<'_, FileSystemRegistry>,
) -> Result<Vec<FileSystemProvider>, String> {
    Ok(registry.get_all_file_system_providers())
}

/// Create file watcher
#[tauri::command]
pub async fn create_file_watcher(
    watcher: FileWatcher,
    registry: State<'_, FileSystemRegistry>,
) -> Result<String, String> {
    registry.create_file_watcher(watcher)
}

/// Dispose file watcher
#[tauri::command]
pub async fn dispose_file_watcher(
    watcher_id: String,
    registry: State<'_, FileSystemRegistry>,
) -> Result<(), String> {
    registry.dispose_file_watcher(&watcher_id)
}

/// Get file watcher
#[tauri::command]
pub async fn get_file_watcher(
    watcher_id: String,
    registry: State<'_, FileSystemRegistry>,
) -> Result<FileWatcher, String> {
    registry.get_file_watcher(&watcher_id)
}

/// Get all file watchers
#[tauri::command]
pub async fn get_all_file_watchers(
    registry: State<'_, FileSystemRegistry>,
) -> Result<Vec<FileWatcher>, String> {
    Ok(registry.get_all_file_watchers())
}

/// Apply workspace edit
#[tauri::command]
pub async fn apply_workspace_edit(
    edit: WorkspaceEdit,
    registry: State<'_, FileSystemRegistry>,
) -> Result<(), String> {
    for file_edit in edit.edits {
        match file_edit {
            FileEdit::CreateFile { uri, options } => {
                let path = uri_to_path(&uri)?;

                // Check if file exists
                if path.exists() {
                    if let Some(opts) = options {
                        if opts.ignore_if_exists {
                            continue;
                        }
                        if !opts.overwrite {
                            return Err(format!("File already exists: {}", uri));
                        }
                    } else {
                        return Err(format!("File already exists: {}", uri));
                    }
                }

                // Create parent directory
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }

                // Create empty file
                fs::write(&path, b"").map_err(|e| format!("Failed to create file: {}", e))?;

                // Emit event
                registry.emit_file_change_event(FileChangeEvent {
                    uri: uri.clone(),
                    change_type: FileChangeType::Created,
                })?;
            }
            FileEdit::DeleteFile { uri, options } => {
                let path = uri_to_path(&uri)?;

                // Check if file exists
                if !path.exists() {
                    if let Some(opts) = options {
                        if opts.ignore_if_not_exists {
                            continue;
                        }
                    }
                    return Err(format!("File not found: {}", uri));
                }

                let recursive = options.map(|o| o.recursive).unwrap_or(false);

                let metadata = fs::metadata(&path)
                    .map_err(|e| format!("Failed to get file metadata: {}", e))?;

                if metadata.is_dir() {
                    if recursive {
                        fs::remove_dir_all(&path)
                            .map_err(|e| format!("Failed to delete directory: {}", e))?;
                    } else {
                        fs::remove_dir(&path)
                            .map_err(|e| format!("Failed to delete directory: {}", e))?;
                    }
                } else {
                    fs::remove_file(&path).map_err(|e| format!("Failed to delete file: {}", e))?;
                }

                // Emit event
                registry.emit_file_change_event(FileChangeEvent {
                    uri: uri.clone(),
                    change_type: FileChangeType::Deleted,
                })?;
            }
            FileEdit::RenameFile {
                old_uri,
                new_uri,
                options,
            } => {
                let old_path = uri_to_path(&old_uri)?;
                let new_path = uri_to_path(&new_uri)?;

                // Check if new file exists
                if new_path.exists() {
                    if let Some(opts) = options {
                        if !opts.overwrite {
                            return Err(format!("File already exists: {}", new_uri));
                        }
                    } else {
                        return Err(format!("File already exists: {}", new_uri));
                    }
                }

                // Create parent directory for new path
                if let Some(parent) = new_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }

                fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename: {}", e))?;

                // Emit events
                registry.emit_file_change_event(FileChangeEvent {
                    uri: old_uri.clone(),
                    change_type: FileChangeType::Deleted,
                })?;

                registry.emit_file_change_event(FileChangeEvent {
                    uri: new_uri.clone(),
                    change_type: FileChangeType::Created,
                })?;
            }
            FileEdit::TextEdit { uri, edits } => {
                // Text edits are applied through the frontend (Monaco editor)
                // This is just logged here for tracking purposes
                println!(
                    "[FileSystem] Text edit requested for {}: {} edit(s)",
                    uri,
                    edits.len()
                );

                // Emit change event
                registry.emit_file_change_event(FileChangeEvent {
                    uri: uri.clone(),
                    change_type: FileChangeType::Changed,
                })?;
            }
        }
    }

    Ok(())
}

/// Clear file system data for owner
#[tauri::command]
pub async fn clear_filesystem_data(
    owner: String,
    registry: State<'_, FileSystemRegistry>,
) -> Result<(), String> {
    registry.clear_filesystem_data(&owner);
    Ok(())
}

// Helper functions

/// Convert URI to file path
fn uri_to_path(uri: &str) -> Result<PathBuf, String> {
    let path_str = uri.trim_start_matches("file://");

    // Handle Windows paths (C:/ instead of /C:/)
    #[cfg(windows)]
    let path_str = if path_str.starts_with('/') && path_str.len() > 2 && path_str.chars().nth(2) == Some(':') {
        &path_str[1..]
    } else {
        path_str
    };

    Ok(PathBuf::from(path_str))
}

/// Copy directory recursively
fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), String> {
    // Create destination directory
    fs::create_dir_all(destination)
        .map_err(|e| format!("Failed to create destination directory: {}", e))?;

    // Copy all entries
    for entry in fs::read_dir(source).map_err(|e| format!("Failed to read source directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let source_path = entry.path();
        let dest_path = destination.join(entry.file_name());

        let metadata = entry
            .metadata()
            .map_err(|e| format!("Failed to get entry metadata: {}", e))?;

        if metadata.is_dir() {
            copy_dir_recursive(&source_path, &dest_path)?;
        } else {
            fs::copy(&source_path, &dest_path)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }

    Ok(())
}
