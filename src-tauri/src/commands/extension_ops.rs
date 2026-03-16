use tauri::State;
use std::sync::Mutex;
use crate::extension_host::ExtensionHostManager;

#[tauri::command]
pub fn start_extension_host(
    ext_host: State<Mutex<ExtensionHostManager>>,
) -> Result<(), String> {
    let mut host = ext_host.lock().unwrap();

    // Get the extension host path
    // In development, it's at extension-host/dist/main.js
    // In production, we'll bundle it with the app
    let extension_host_path = if cfg!(debug_assertions) {
        // Development path
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .parent()
            .ok_or("Failed to get parent directory")?
            .join("extension-host/dist/main.js")
            .to_str()
            .ok_or("Failed to convert path to string")?
            .to_string()
    } else {
        // Production path (will need to bundle extension host with app)
        // For now, use the same path
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .parent()
            .ok_or("Failed to get parent directory")?
            .join("extension-host/dist/main.js")
            .to_str()
            .ok_or("Failed to convert path to string")?
            .to_string()
    };

    host.start(&extension_host_path)?;

    Ok(())
}

#[tauri::command]
pub fn get_extensions_dir() -> Result<String, String> {
    // Get the extensions directory
    // In development, create it in the project root
    // In production, use the app data directory
    let extensions_dir = if cfg!(debug_assertions) {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .parent()
            .ok_or("Failed to get parent directory")?
            .join("extensions")
    } else {
        // Production: use app data directory
        // TODO: Use proper app data path
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join("extensions")
    };

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&extensions_dir)
        .map_err(|e| format!("Failed to create extensions directory: {}", e))?;

    Ok(extensions_dir.to_str()
        .ok_or("Failed to convert path to string")?
        .to_string())
}
