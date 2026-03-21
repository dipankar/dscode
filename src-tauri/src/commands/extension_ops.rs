use tauri::State;
use std::sync::{Arc, Mutex};
use crate::extension_host::{ExtensionHostManager, NngIpcManager};
use crate::marketplace;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use zip::ZipArchive;
use tokio::sync::RwLock;

use semver::Version;
use std::collections::{HashSet, VecDeque};

use crate::config::AppDirectories;
use crate::session::SessionManager;

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
pub fn get_extensions_dir(app_dirs: State<'_, AppDirectories>) -> Result<String, String> {
    Ok(app_dirs
        .extensions_dir
        .to_str()
        .ok_or("Failed to convert extensions directory path to UTF-8 string")?
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
    pub extension_dependencies: Option<Vec<String>>,
    pub extension_pack: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
    pub repository: Option<serde_json::Value>,
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
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    pub repository: Option<String>,
}

/// Install a .vsix extension package
#[tauri::command]
pub async fn install_extension(
    vsix_path: String,
    app_dirs: State<'_, AppDirectories>,
    nng_manager: State<'_, NngIpcManager>,
    session: State<'_, Arc<RwLock<SessionManager>>>,
) -> Result<InstalledExtension, String> {
    let extensions_dir = app_dirs.extensions_dir.clone();
    let installed = tokio::task::spawn_blocking(move || install_vsix(vsix_path, extensions_dir))
        .await
        .map_err(|e| format!("Task failed: {}", e))??;

    let dependency_installs = ensure_extension_dependencies(&installed.dependencies, &*app_dirs)
        .await?;

    if !dependency_installs.is_empty() {
        let ids: Vec<String> = dependency_installs.iter().map(|ext| ext.id.clone()).collect();
        println!("[Extensions] Installed dependencies for {}: {:?}", installed.id, ids);
    }

    trigger_reload_and_rescan(&*nng_manager, &*session).await?;

    Ok(installed)
}

/// Uninstall an extension
#[tauri::command]
pub async fn uninstall_extension(
    session: State<'_, Arc<RwLock<SessionManager>>>,
    extension_id: String,
) -> Result<(), String> {
    let session = session.read().await;
    session.delete_extension(&extension_id).await
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
    app_dirs: State<'_, AppDirectories>,
    session: State<'_, Arc<RwLock<SessionManager>>>,
    publisher: String,
    name: String,
    version: String,
) -> Result<InstalledExtension, String> {
    // Download the extension
    let vsix_path = marketplace::download_extension(publisher.clone(), name.clone(), version.clone())
        .await?;

    // Install it
    let extensions_dir = app_dirs.extensions_dir.clone();
    let vsix_string = vsix_path.to_string_lossy().to_string();
    let installed = tokio::task::spawn_blocking(move || install_vsix(vsix_string, extensions_dir))
        .await
        .map_err(|e| format!("Task failed: {}", e))??;

    // Clean up the downloaded file (non-blocking)
    let vsix_path_clone = vsix_path.clone();
    tokio::task::spawn_blocking(move || {
        if let Err(e) = std::fs::remove_file(&vsix_path_clone) {
            eprintln!("Failed to clean up downloaded .vsix file: {}", e);
        }
    });

    // Notify Extension Host to reload extensions
    let dependency_installs = ensure_extension_dependencies(&installed.dependencies, &*app_dirs)
        .await?;

    if !dependency_installs.is_empty() {
        let ids: Vec<String> = dependency_installs.iter().map(|ext| ext.id.clone()).collect();
        println!("[Extensions] Installed transitive dependencies for {}: {:?}", installed.id, ids);
    }

    trigger_reload_and_rescan(&*nng_manager, &*session).await?;

    Ok(installed)
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

    let children_value = if let Some(obj) = response.as_object() {
        let success = obj.get("success").and_then(|v| v.as_bool()).unwrap_or(true);
        let error = obj.get("error").and_then(|v| v.as_str()).map(|s| s.to_string());
        let children = obj.get("children").cloned().unwrap_or(Value::Array(Vec::new()));
        if success {
            children
        } else {
            return Err(error.unwrap_or_else(|| "Tree provider error".to_string()));
        }
    } else {
        response
    };

    let elements = match children_value {
        Value::Array(arr) => arr,
        Value::Null => Vec::new(),
        other => {
            return Err(format!("Invalid tree view response format: {:?}", other));
        }
    };

    let mut results = Vec::with_capacity(elements.len());

    for element_value in elements {
        let element_clone = element_value.clone();

        let item_value = match ipc
            .request(
                "treeView:getTreeItem",
                serde_json::json!({
                    "viewId": view_id,
                    "element": element_value,
                }),
            )
            .await
        {
            Ok(response) => {
                if let Some(obj) = response.as_object() {
                    let success = obj.get("success").and_then(|v| v.as_bool()).unwrap_or(true);
                    if success {
                        obj.get("item").cloned().unwrap_or(Value::Null)
                    } else {
                        element_clone.clone()
                    }
                } else {
                    response
                }
            }
            Err(_) => element_clone.clone(),
        };

        let record = serde_json::json!({
            "element": element_clone,
            "item": item_value,
        });

        results.push(record);
    }

    Ok(results)
}

/// Get a specific tree item for the provided element
#[tauri::command]
pub async fn extension_tree_get_item(
    _ext_host: State<'_, Mutex<ExtensionHostManager>>,
    nng_manager: State<'_, NngIpcManager>,
    view_id: String,
    element: Value,
) -> Result<Value, String> {
    let ipc = nng_manager.get_outgoing("main").await
        .ok_or("Extension host not connected via NNG")?;

    let payload = serde_json::json!({
        "viewId": view_id,
        "element": element,
    });

    let response = ipc.request("treeView:getTreeItem", payload).await?;

    if let Some(obj) = response.as_object() {
        let success = obj.get("success").and_then(|v| v.as_bool()).unwrap_or(true);
        if success {
            if let Some(item) = obj.get("item") {
                return Ok(item.clone());
            }
        } else if let Some(error) = obj.get("error").and_then(|v| v.as_str()) {
            return Err(error.to_string());
        }
    }

    Ok(response)
}

/// Execute an extension command
#[tauri::command]
pub async fn extension_execute_command(
    _ext_host: State<'_, Mutex<ExtensionHostManager>>,
    nng_manager: State<'_, NngIpcManager>,
    session: State<'_, Arc<RwLock<SessionManager>>>,
    command: String,
    args: Vec<Value>,
) -> Result<Value, String> {
    {
        let manager = session.read().await;
        manager.ensure_command_ready(&command).await?;
    }

    // Get IPC connection for the main extension host
    let ipc = nng_manager.get_outgoing("main").await
        .ok_or("Extension host not connected via NNG")?;

    let payload = serde_json::json!({
        "command": command,
        "args": args,
    });

    ipc.request("executeCommand", payload).await
}

#[tauri::command]
pub async fn extension_tree_notify_event(
    nng_manager: State<'_, NngIpcManager>,
    view_id: String,
    event: String,
    element: Option<Value>,
    selection: Option<Vec<Value>>,
    visible: Option<bool>,
) -> Result<(), String> {
    let mut payload = serde_json::Map::new();
    payload.insert("viewId".to_string(), Value::String(view_id));
    payload.insert("event".to_string(), Value::String(event));

    if let Some(element) = element {
        payload.insert("element".to_string(), element);
    }

    if let Some(selection) = selection {
        payload.insert("selection".to_string(), Value::Array(selection));
    }

    if let Some(visible) = visible {
        payload.insert("visible".to_string(), Value::Bool(visible));
    }

    let response = nng_manager.request("main", "treeView:event", Value::Object(payload)).await?;

    if response.get("success").and_then(|v| v.as_bool()).unwrap_or(true) {
        Ok(())
    } else {
        let err = response
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Tree view event failed");
        Err(err.to_string())
    }
}

fn install_vsix(vsix_path: String, extensions_root: PathBuf) -> Result<InstalledExtension, String> {
    let file = fs::File::open(&vsix_path)
        .map_err(|e| format!("Failed to open .vsix file: {}", e))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read .vsix archive: {}", e))?;

    let mut manifest: Option<ExtensionManifest> = None;
    let mut extension_folder: Option<String> = None;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;

        let file_path = file.name().to_string();

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

    validate_manifest(&manifest)?;

    let new_version = Version::parse(&manifest.version).unwrap_or_else(|_| Version::new(0, 0, 0));
    let extension_id = format!("{}.{}", manifest.publisher, manifest.name);
    let install_path = extensions_root.join(&extension_id);

    if install_path.exists() {
        if let Some(existing_version) = read_existing_extension_version(&install_path) {
            if existing_version > new_version {
                return Err(format!(
                    "Installed version {} is newer than requested {} for {}",
                    existing_version, new_version, extension_id
                ));
            }
        }

        fs::remove_dir_all(&install_path)
            .map_err(|e| format!("Failed to replace existing extension directory: {}", e))?;
    }

    fs::create_dir_all(&install_path)
        .map_err(|e| format!("Failed to create extension directory: {}", e))?;

    let file = fs::File::open(&vsix_path)
        .map_err(|e| format!("Failed to reopen .vsix file: {}", e))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read .vsix archive: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;

        let file_path = file.name().to_string();

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

    let mut dependency_list = manifest.extension_dependencies.clone().unwrap_or_default();
    if let Some(pack) = manifest.extension_pack.clone() {
        dependency_list.extend(pack);
    }
    dependency_list.sort();
    dependency_list.dedup();

    let categories = manifest.categories.clone().unwrap_or_default();
    let repository = manifest
        .repository
        .as_ref()
        .and_then(extract_repository_url);

    Ok(InstalledExtension {
        id: extension_id.clone(),
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        publisher: manifest.publisher.clone(),
        description: manifest.description.clone(),
        path: install_path.to_string_lossy().to_string(),
        contributes: manifest.contributes.clone(),
        dependencies: dependency_list,
        categories,
        repository,
    })
}

fn read_existing_extension_version(path: &Path) -> Option<Version> {
    let manifest_path = path.join("package.json");
    let contents = fs::read_to_string(manifest_path).ok()?;
    let manifest: ExtensionManifest = serde_json::from_str(&contents).ok()?;
    Version::parse(&manifest.version).ok()
}

fn extract_repository_url(value: &serde_json::Value) -> Option<String> {
    if let Some(url) = value.as_str() {
        return Some(url.to_string());
    }

    if let Some(obj) = value.as_object() {
        if let Some(url) = obj.get("url").and_then(|v| v.as_str()) {
            return Some(url.to_string());
        }
    }

    None
}

async fn ensure_extension_dependencies(
    dependencies: &[String],
    app_dirs: &AppDirectories,
) -> Result<Vec<InstalledExtension>, String> {
    if dependencies.is_empty() {
        return Ok(Vec::new());
    }

    let extensions_root = app_dirs.extensions_dir.clone();
    let mut visited = HashSet::new();
    let mut queue: VecDeque<String> = dependencies.iter().cloned().collect();
    let mut installed = Vec::new();

    while let Some(dep_id) = queue.pop_front() {
        if !visited.insert(dep_id.clone()) {
            continue;
        }

        if dep_id.trim().is_empty() {
            continue;
        }

        let dependency_path = extensions_root.join(&dep_id);
        if dependency_path.exists() {
            for nested in read_manifest_dependencies_from_disk(&dependency_path) {
                queue.push_back(nested);
            }
            continue;
        }

        let (publisher, name) = match parse_extension_id(&dep_id) {
            Some(parts) => parts,
            None => {
                eprintln!("[Extensions] Invalid dependency identifier '{}'", dep_id);
                continue;
            }
        };

        println!("[Extensions] Auto-installing dependency {}", dep_id);

        let details = marketplace::get_extension_details(publisher.clone(), name.clone()).await?;
        let vsix_path = marketplace::download_extension(publisher.clone(), name.clone(), details.version.clone()).await?;
        let vsix_string = vsix_path.to_string_lossy().to_string();
        let extensions_root_clone = extensions_root.clone();

        let installed_dep = tokio::task::spawn_blocking(move || install_vsix(vsix_string, extensions_root_clone))
            .await
            .map_err(|e| format!("Task failed: {}", e))??;

        if let Err(e) = fs::remove_file(&vsix_path) {
            if e.kind() != ErrorKind::NotFound {
                eprintln!("[Extensions] Failed to delete temporary VSIX {:?}: {}", vsix_path, e);
            }
        }

        for nested in installed_dep.dependencies.iter() {
            queue.push_back(nested.clone());
        }

        installed.push(installed_dep);
    }

    Ok(installed)
}

fn parse_extension_id(id: &str) -> Option<(String, String)> {
    let mut parts = id.splitn(2, '.');
    let publisher = parts.next()?.trim();
    let name = parts.next()?.trim();
    if publisher.is_empty() || name.is_empty() {
        return None;
    }
    Some((publisher.to_string(), name.to_string()))
}

fn read_manifest_dependencies_from_disk(path: &Path) -> Vec<String> {
    let manifest_path = path.join("package.json");
    let contents = match fs::read_to_string(manifest_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let manifest: serde_json::Value = match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut deps = Vec::new();

    if let Some(arr) = manifest.get("extensionDependencies").and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(id) = item.as_str() {
                deps.push(id.to_string());
            }
        }
    }

    if let Some(arr) = manifest.get("extensionPack").and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(id) = item.as_str() {
                deps.push(id.to_string());
            }
        }
    }

    deps.sort();
    deps.dedup();
    deps
}

async fn trigger_reload_and_rescan(
    nng_manager: &NngIpcManager,
    session: &Arc<RwLock<SessionManager>>,
) -> Result<(), String> {
    if let Some(ipc) = nng_manager.get_outgoing("main").await {
        if let Err(err) = ipc.request("reload-extensions", serde_json::json!({})).await {
            eprintln!("[Extensions] Failed to request extension host reload: {}", err);
        }
    } else {
        eprintln!("[Extensions] Extension host not connected via NNG during reload request");
    }

    session.read().await.scan_and_emit_extensions().await
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
