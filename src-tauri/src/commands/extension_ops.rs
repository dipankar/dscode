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

// Note: Extension host is now started by SessionManager on app initialization.
// This command is kept for backward compatibility but is now a no-op.
#[tauri::command]
pub async fn start_extension_host(
    _ext_host: State<'_, Mutex<ExtensionHostManager>>,
    _nng_manager: State<'_, NngIpcManager>,
) -> Result<(), String> {
    println!("[ExtensionHost] start_extension_host called - extension host is managed by SessionManager");
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

#[derive(Debug, Serialize, Deserialize)]
pub struct InstalledExtension {
    pub id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub description: Option<String>,
    pub path: String,
    #[serde(default)]
    pub contributes: Option<serde_json::Value>,
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

        // Check if extension already exists - if so, just return it (reload will pick it up)
        if install_path.exists() {
            return Ok(InstalledExtension {
                id: extension_id.clone(),
                name: manifest.name.clone(),
                version: manifest.version.clone(),
                publisher: manifest.publisher.clone(),
                description: manifest.description.clone(),
                path: install_path.to_string_lossy().to_string(),
                contributes: manifest.contributes.clone().map(|c| serde_json::to_value(c).unwrap_or(serde_json::Value::Null)),
            });
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
            contributes: manifest.contributes.clone().map(|c| serde_json::to_value(c).unwrap_or(serde_json::Value::Null)),
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Uninstall an extension
#[tauri::command]
pub async fn uninstall_extension(
    nng_manager: State<'_, NngIpcManager>,
    extension_id: String
) -> Result<(), String> {
    // Request Extension Host to uninstall the extension
    let response = nng_manager.request("main", "uninstall-extension", serde_json::json!({
        "extensionId": extension_id
    })).await?;

    let success = response.get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !success {
        let error = response.get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        return Err(error.to_string());
    }

    Ok(())
}

/// List all installed extensions
#[tauri::command]
pub async fn list_extensions(
    nng_manager: State<'_, NngIpcManager>
) -> Result<Vec<InstalledExtension>, String> {
    // Request extension list from Extension Host via IPC
    let ipc = nng_manager.get_outgoing("main").await
        .ok_or("Extension host not connected via NNG")?;

    let response = ipc.request("list-extensions", serde_json::json!({})).await?;

    let extensions_data = response.get("extensions")
        .ok_or("Invalid response: missing extensions field")?;

    let extensions: Vec<InstalledExtension> = serde_json::from_value(extensions_data.clone())
        .map_err(|e| format!("Failed to parse extensions: {}", e))?;

    Ok(extensions)
}

/// Get extension contributions (for activity bar, commands, views, etc.)
#[derive(Debug, Serialize)]
pub struct ExtensionContribution {
    pub extension_id: String,
    pub extension_name: String,
    pub contributes: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn get_extension_contributions(
    nng_manager: State<'_, NngIpcManager>
) -> Result<Vec<ExtensionContribution>, String> {
    // Request extension list from Extension Host via IPC
    let ipc = nng_manager.get_outgoing("main").await
        .ok_or("Extension host not connected via NNG")?;

    let response = ipc.request("list-extensions", serde_json::json!({})).await?;

    let extensions_data = response.get("extensions")
        .ok_or("Invalid response: missing extensions field")?;

    let extensions: Vec<InstalledExtension> = serde_json::from_value(extensions_data.clone())
        .map_err(|e| format!("Failed to parse extensions: {}", e))?;

    // Extract contributions from extensions
    let contributions = extensions
        .into_iter()
        .filter_map(|ext| {
            // Only include extensions that have contributions
            let contributes = ext.contributes.clone()?;
            Some(ExtensionContribution {
                extension_id: ext.id,
                extension_name: ext.name,
                contributes: Some(contributes),
            })
        })
        .collect();

    Ok(contributions)
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
    nng_manager: State<'_, NngIpcManager>,
    publisher: String,
    name: String,
    version: String,
) -> Result<InstalledExtension, String> {
    // Download the extension
    let vsix_path = marketplace::download_extension(publisher.clone(), name.clone(), version.clone())
        .await?;

    // Install it
    let result = install_extension(vsix_path.to_string_lossy().to_string()).await;

    // Clean up the downloaded file (non-blocking)
    let vsix_path_clone = vsix_path.clone();
    tokio::task::spawn_blocking(move || {
        if let Err(e) = std::fs::remove_file(&vsix_path_clone) {
            eprintln!("Failed to clean up downloaded .vsix file: {}", e);
        }
    });

    // Notify Extension Host to reload extensions
    if result.is_ok() {
        if let Some(ipc) = nng_manager.get_outgoing("main").await {
            let _ = ipc.request("reload-extensions", serde_json::json!({})).await;
        }
    }

    result
}

// ============ Extension IPC Commands (via NNG) ============

/// Get children from extension tree view
#[tauri::command]
pub async fn extension_tree_get_children(
    _ext_host: State<'_, Mutex<ExtensionHostManager>>,
    nng_manager: State<'_, NngIpcManager>,
    view_id: String,
    element: Option<Value>,
) -> Result<Vec<Value>, String> {
    // Get IPC connection for the main extension host
    let ipc = nng_manager.get_outgoing("main").await
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
    _ext_host: State<'_, Mutex<ExtensionHostManager>>,
    nng_manager: State<'_, NngIpcManager>,
    command: String,
    args: Vec<Value>,
) -> Result<Value, String> {
    // Get IPC connection for the main extension host
    let ipc = nng_manager.get_outgoing("main").await
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
