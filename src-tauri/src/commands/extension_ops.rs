use tauri::State;
use std::sync::Mutex;
use crate::extension_host::{ExtensionHostManager, NngIpcManager};
use crate::marketplace;
use std::fs;
use std::io;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use zip::ZipArchive;

#[tauri::command]
pub async fn start_extension_host(
    ext_host: State<'_, Mutex<ExtensionHostManager>>,
    nng_manager: State<'_, NngIpcManager>,
) -> Result<(), String> {
    // Generate unique IPC URL for this extension host
    let ext_host_id = {
        let host = ext_host.lock().unwrap();
        host.extension_id.clone()
    };
    let ipc_url = format!("ipc:///tmp/dscode-ext-host-{}.ipc", ext_host_id);

    println!("[ExtensionHost] Using IPC URL: {}", ipc_url);

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

    // Start extension host with IPC URL as environment variable
    {
        let mut host = ext_host.lock().unwrap();
        host.start_with_ipc(&extension_host_path, &ipc_url)?;
    }

    // Wait for extension host to bind to the socket (give it 2 seconds)
    println!("[ExtensionHost] Waiting for extension host to bind to socket...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Connect to the extension host from Tauri side
    println!("[ExtensionHost] Connecting to extension host via NNG...");
    nng_manager.connect(&ext_host_id, &ipc_url).await?;

    println!("[ExtensionHost] Successfully connected to extension host");
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

// Extension manifest structure (package.json)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifest {
    pub name: String,
    pub display_name: Option<String>,
    pub version: String,
    pub publisher: String,
    pub description: Option<String>,
    pub main: Option<String>,
    pub activation_events: Option<Vec<String>>,
    pub engines: Option<serde_json::Value>,
    pub contributes: Option<serde_json::Value>,
    pub dependencies: Option<serde_json::Value>,
    pub dev_dependencies: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct InstalledExtension {
    pub id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub description: Option<String>,
    pub path: String,
}

/// Install a .vsix extension package
#[tauri::command]
pub async fn install_extension(vsix_path: String) -> Result<InstalledExtension, String> {
    tokio::task::spawn_blocking(move || {
        // Open the .vsix file (which is a ZIP archive)
        let file = fs::File::open(&vsix_path)
            .map_err(|e| format!("Failed to open .vsix file: {}", e))?;

        let mut archive = ZipArchive::new(file)
            .map_err(|e| format!("Failed to read .vsix archive: {}", e))?;

        // Find and read the package.json (manifest) from the extension folder
        let mut manifest: Option<ExtensionManifest> = None;
        let mut extension_folder: Option<String> = None;

        // Look for extension/package.json
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("Failed to read archive entry: {}", e))?;

            let file_path = file.name().to_string();

            // VS Code extensions have their contents in an "extension" folder
            if file_path == "extension/package.json" {
                let mut contents = String::new();
                io::Read::read_to_string(&mut file, &mut contents)
                    .map_err(|e| format!("Failed to read package.json: {}", e))?;

                manifest = Some(serde_json::from_str(&contents)
                    .map_err(|e| format!("Failed to parse package.json: {}", e))?);
                extension_folder = Some("extension".to_string());
                break;
            }
        }

        let manifest = manifest.ok_or("No valid package.json found in .vsix file")?;
        let extension_folder = extension_folder.ok_or("No extension folder found")?;

        // Validate manifest
        validate_manifest(&manifest)?;

        // Get extensions directory
        let extensions_dir = get_extensions_dir()?;
        let extension_id = format!("{}.{}", manifest.publisher, manifest.name);
        let install_path = PathBuf::from(&extensions_dir).join(&extension_id);

        // Check if extension already exists
        if install_path.exists() {
            return Err(format!("Extension {} is already installed", extension_id));
        }

        // Create extension directory
        fs::create_dir_all(&install_path)
            .map_err(|e| format!("Failed to create extension directory: {}", e))?;

        // Extract extension files
        let file = fs::File::open(&vsix_path)
            .map_err(|e| format!("Failed to reopen .vsix file: {}", e))?;

        let mut archive = ZipArchive::new(file)
            .map_err(|e| format!("Failed to read .vsix archive: {}", e))?;

        // Extract all files from the extension folder
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("Failed to read archive entry: {}", e))?;

            let file_path = file.name().to_string();

            // Only extract files from the extension folder
            if let Some(relative_path) = file_path.strip_prefix(&format!("{}/", extension_folder)) {
                let outpath = install_path.join(relative_path);

                if file.is_dir() {
                    fs::create_dir_all(&outpath)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                } else {
                    if let Some(parent) = outpath.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                    }

                    let mut outfile = fs::File::create(&outpath)
                        .map_err(|e| format!("Failed to create file: {}", e))?;

                    io::copy(&mut file, &mut outfile)
                        .map_err(|e| format!("Failed to extract file: {}", e))?;
                }
            }
        }

        Ok(InstalledExtension {
            id: extension_id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            publisher: manifest.publisher.clone(),
            description: manifest.description.clone(),
            path: install_path.to_string_lossy().to_string(),
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Uninstall an extension
#[tauri::command]
pub async fn uninstall_extension(extension_id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let extensions_dir = get_extensions_dir()?;
        let extension_path = PathBuf::from(&extensions_dir).join(&extension_id);

        if !extension_path.exists() {
            return Err(format!("Extension {} is not installed", extension_id));
        }

        fs::remove_dir_all(&extension_path)
            .map_err(|e| format!("Failed to uninstall extension: {}", e))?;

        Ok(())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// List all installed extensions
#[tauri::command]
pub async fn list_extensions() -> Result<Vec<InstalledExtension>, String> {
    tokio::task::spawn_blocking(move || {
        let extensions_dir = get_extensions_dir()?;
        let extensions_path = PathBuf::from(&extensions_dir);

        if !extensions_path.exists() {
            return Ok(Vec::new());
        }

        let mut extensions = Vec::new();

        for entry in fs::read_dir(&extensions_path)
            .map_err(|e| format!("Failed to read extensions directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                let manifest_path = path.join("package.json");

                if manifest_path.exists() {
                    match fs::read_to_string(&manifest_path) {
                        Ok(contents) => {
                            match serde_json::from_str::<ExtensionManifest>(&contents) {
                                Ok(manifest) => {
                                    let extension_id = format!("{}.{}", manifest.publisher, manifest.name);
                                    extensions.push(InstalledExtension {
                                        id: extension_id,
                                        name: manifest.name.clone(),
                                        version: manifest.version.clone(),
                                        publisher: manifest.publisher.clone(),
                                        description: manifest.description.clone(),
                                        path: path.to_string_lossy().to_string(),
                                    });
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse manifest for {}: {}", path.display(), e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read manifest for {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(extensions)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get extension contributions (for activity bar, commands, views, etc.)
#[derive(Debug, Serialize)]
pub struct ExtensionContribution {
    pub extension_id: String,
    pub extension_name: String,
    pub contributes: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn get_extension_contributions() -> Result<Vec<ExtensionContribution>, String> {
    tokio::task::spawn_blocking(move || {
        let extensions_dir = get_extensions_dir()?;
        let extensions_path = PathBuf::from(&extensions_dir);

        if !extensions_path.exists() {
            return Ok(Vec::new());
        }

        let mut contributions = Vec::new();

        for entry in fs::read_dir(&extensions_path)
            .map_err(|e| format!("Failed to read extensions directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                let manifest_path = path.join("package.json");

                if manifest_path.exists() {
                    match fs::read_to_string(&manifest_path) {
                        Ok(contents) => {
                            match serde_json::from_str::<ExtensionManifest>(&contents) {
                                Ok(manifest) => {
                                    if manifest.contributes.is_some() {
                                        let extension_id = format!("{}.{}", manifest.publisher, manifest.name);
                                        contributions.push(ExtensionContribution {
                                            extension_id,
                                            extension_name: manifest.display_name.clone().unwrap_or(manifest.name.clone()),
                                            contributes: manifest.contributes.clone(),
                                        });
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse manifest for {}: {}", path.display(), e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read manifest for {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(contributions)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Validate extension manifest
fn validate_manifest(manifest: &ExtensionManifest) -> Result<(), String> {
    // Required fields
    if manifest.name.is_empty() {
        return Err("Extension name is required".to_string());
    }

    if manifest.version.is_empty() {
        return Err("Extension version is required".to_string());
    }

    if manifest.publisher.is_empty() {
        return Err("Extension publisher is required".to_string());
    }

    // Validate name format (lowercase alphanumeric with hyphens)
    if !manifest.name.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '-') {
        return Err("Extension name must be lowercase alphanumeric with hyphens".to_string());
    }

    // Validate version format (semantic versioning)
    let version_parts: Vec<&str> = manifest.version.split('.').collect();
    if version_parts.len() != 3 {
        return Err("Extension version must follow semantic versioning (x.y.z)".to_string());
    }

    for part in version_parts {
        if part.parse::<u32>().is_err() {
            return Err("Extension version parts must be numbers".to_string());
        }
    }

    // Validate engines.vscode if present
    if let Some(engines) = &manifest.engines {
        if engines.get("vscode").is_none() {
            return Err("Extension must specify vscode engine version".to_string());
        }
    } else {
        return Err("Extension must specify engines field with vscode version".to_string());
    }

    Ok(())
}

// ============ Marketplace Integration ============

/// Search the VS Code Marketplace
#[tauri::command]
pub async fn search_marketplace(
    query: String,
    page: Option<i32>,
    page_size: Option<i32>,
) -> Result<Vec<marketplace::MarketplaceExtension>, String> {
    marketplace::search_marketplace(
        query,
        page.unwrap_or(1),
        page_size.unwrap_or(50),
    ).await
}

/// Get extension details from marketplace
#[tauri::command]
pub async fn get_marketplace_extension(
    publisher: String,
    name: String,
) -> Result<marketplace::MarketplaceExtension, String> {
    marketplace::get_extension_details(publisher, name).await
}

/// Download and install extension from marketplace
#[tauri::command]
pub async fn install_from_marketplace(
    publisher: String,
    name: String,
    version: String,
) -> Result<InstalledExtension, String> {
    // Download the extension
    let vsix_path = marketplace::download_extension(publisher.clone(), name.clone(), version.clone())
        .await?;

    // Install it
    let result = install_extension(vsix_path.to_string_lossy().to_string()).await;

    // Clean up the downloaded file
    if let Err(e) = std::fs::remove_file(&vsix_path) {
        eprintln!("Failed to clean up downloaded .vsix file: {}", e);
    }

    result
}

// ============ Extension IPC Commands (via NNG) ============

/// Get children from extension tree view
#[tauri::command]
pub async fn extension_tree_get_children(
    ext_host: State<'_, Mutex<ExtensionHostManager>>,
    nng_manager: State<'_, NngIpcManager>,
    view_id: String,
    element: Option<Value>,
) -> Result<Vec<Value>, String> {
    // Get extension host ID
    let ext_host_id = {
        let host = ext_host.lock().unwrap();
        host.extension_id.clone()
    };

    // Get IPC connection
    let ipc = nng_manager.get(&ext_host_id).await
        .ok_or("Extension host not connected via NNG")?;

    let payload = serde_json::json!({
        "viewId": view_id,
        "element": element,
    });

    let response = ipc.request("treeView:getChildren", payload).await?;

    // Response should be an array of tree items
    let items = response.as_array()
        .ok_or("Invalid response: expected array")?
        .clone();

    Ok(items)
}

/// Execute an extension command
#[tauri::command]
pub async fn extension_execute_command(
    ext_host: State<'_, Mutex<ExtensionHostManager>>,
    nng_manager: State<'_, NngIpcManager>,
    command: String,
    args: Vec<Value>,
) -> Result<Value, String> {
    // Get extension host ID
    let ext_host_id = {
        let host = ext_host.lock().unwrap();
        host.extension_id.clone()
    };

    // Get IPC connection
    let ipc = nng_manager.get(&ext_host_id).await
        .ok_or("Extension host not connected via NNG")?;

    let payload = serde_json::json!({
        "command": command,
        "args": args,
    });

    ipc.request("executeCommand", payload).await
}

// Helper trait for pipe operations
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}
