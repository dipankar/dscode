use notify::{RecommendedWatcher, Watcher, RecursiveMode, Event, EventKind};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tauri::{AppHandle, Manager, Emitter};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FileChangeEvent {
    pub path: String,
    pub event_type: String,
}

pub struct FileWatcherState {
    pub watchers: Arc<Mutex<HashMap<String, RecommendedWatcher>>>,
}

impl FileWatcherState {
    pub fn new() -> Self {
        Self {
            watchers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

pub fn watch_file(app_handle: AppHandle, file_path: String) -> Result<(), String> {
    let path = PathBuf::from(&file_path);

    let app_handle_clone = app_handle.clone();
    let file_path_clone = file_path.clone();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        match res {
            Ok(event) => {
                // Only handle modify events (file content changed)
                if matches!(event.kind, EventKind::Modify(_)) {
                    let change_event = FileChangeEvent {
                        path: file_path_clone.clone(),
                        event_type: "modified".to_string(),
                    };

                    // Emit event to frontend
                    let _ = app_handle_clone.emit("file-changed", change_event);
                }
            }
            Err(e) => eprintln!("File watch error: {:?}", e),
        }
    })
    .map_err(|e| format!("Failed to create watcher: {}", e))?;

    // Watch the file
    watcher.watch(&path, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Failed to watch file: {}", e))?;

    // Store watcher in state
    let state: tauri::State<FileWatcherState> = app_handle.state();
    let mut watchers = state.watchers.lock().unwrap();
    watchers.insert(file_path.clone(), watcher);

    Ok(())
}

pub fn unwatch_file(app_handle: AppHandle, file_path: String) -> Result<(), String> {
    let state: tauri::State<FileWatcherState> = app_handle.state();
    let mut watchers = state.watchers.lock().unwrap();

    if let Some(_watcher) = watchers.remove(&file_path) {
        // Watcher will be dropped automatically, stopping the watch
        Ok(())
    } else {
        Err("File not being watched".to_string())
    }
}
