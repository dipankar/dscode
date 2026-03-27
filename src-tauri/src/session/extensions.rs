use super::contributions::ExtensionContributes;
use super::{ExtensionInfo, SessionEvent, SessionManager, SessionState};
use crate::marketplace;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ExtensionManifest {
    name: String,
    display_name: Option<String>,
    version: String,
    publisher: String,
    description: Option<String>,
    main: Option<String>,
    activation_events: Option<Vec<String>>,
    engines: Option<serde_json::Value>,
    contributes: Option<serde_json::Value>,
    dependencies: Option<serde_json::Value>,
    dev_dependencies: Option<serde_json::Value>,
    extension_dependencies: Option<Vec<String>>,
    extension_pack: Option<Vec<String>>,
    categories: Option<Vec<String>>,
    repository: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledExtension {
    pub id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub description: Option<String>,
    pub path: String,
    #[serde(default)]
    pub contributes: Option<ExtensionContributes>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ExtensionContribution {
    pub extension_id: String,
    pub extension_name: String,
    pub contributes: Option<ExtensionContributes>,
}

impl SessionManager {
    pub(super) async fn scan_extensions(&self) -> Result<(), String> {
        println!("[SessionManager] Scanning extensions directory...");

        if !self.app_dirs.extensions_dir.exists() {
            std::fs::create_dir_all(&self.app_dirs.extensions_dir)
                .map_err(|e| format!("Failed to create extensions directory: {}", e))?;
        }

        let entries = std::fs::read_dir(&self.app_dirs.extensions_dir)
            .map_err(|e| format!("Failed to read extensions directory: {}", e))?;

        let mut extensions = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                match self.read_extension_manifest(&path) {
                    Ok(info) => extensions.push(info),
                    Err(e) => eprintln!(
                        "[SessionManager] Failed to read extension manifest at {:?}: {}",
                        path, e
                    ),
                }
            }
        }

        {
            let mut state = self.state.write().await;
            state.installed_extensions = extensions.clone();
        }

        self.rebuild_command_index(&extensions).await;
        self.publish_command_list().await;

        println!("[SessionManager] Found {} installed extensions", extensions.len());

        self.emit_event(SessionEvent::ExtensionsChanged { extensions });

        Ok(())
    }

    pub async fn scan_and_emit_extensions(&self) -> Result<(), String> {
        self.scan_extensions().await
    }

    async fn rebuild_command_index(&self, extensions: &[ExtensionInfo]) {
        let mut map = self.command_map.write().await;
        map.clear();
        for ext in extensions {
            for command in &ext.commands {
                let owners = map.entry(command.clone()).or_insert_with(Vec::new);
                if !owners.contains(&ext.id) {
                    owners.push(ext.id.clone());
                }
            }
        }
    }

    pub(super) async fn publish_command_list(&self) {
        let commands = {
            let map = self.command_map.read().await;
            let mut commands: Vec<String> = map.keys().cloned().collect();
            commands.sort();
            commands
        };

        {
            let mut state = self.state.write().await;
            state.available_commands = commands.clone();
        }

        self.emit_event(SessionEvent::CommandsChanged { commands });
    }

    pub(super) async fn add_command_owner(&self, command: &str, owner: &str) {
        if owner.is_empty() {
            return;
        }
        let mut map = self.command_map.write().await;
        let owners = map.entry(command.to_string()).or_insert_with(Vec::new);
        if !owners.contains(&owner.to_string()) {
            owners.push(owner.to_string());
        }
    }

    pub(super) async fn remove_command_owner(&self, command: &str, owner: &str) {
        let mut map = self.command_map.write().await;
        if let Some(owners) = map.get_mut(command) {
            owners.retain(|entry| entry != owner);
            if owners.is_empty() {
                map.remove(command);
            }
        }
    }

    async fn remove_extension_commands(&self, extension_id: &str) {
        let mut map = self.command_map.write().await;
        map.retain(|_, owners| {
            owners.retain(|owner| owner != extension_id);
            !owners.is_empty()
        });
    }

    async fn is_extension_active(&self, extension_id: &str) -> bool {
        self.state.read().await.active_extensions.iter().any(|ext| ext.id == extension_id)
    }

    async fn owners_from_activation_events(&self, command: &str) -> Vec<String> {
        let pattern = format!("onCommand:{}", command);
        let state = self.state.read().await;
        state
            .installed_extensions
            .iter()
            .filter(|ext| ext.activation_events.iter().any(|evt| evt == &pattern))
            .map(|ext| ext.id.clone())
            .collect()
    }

    pub async fn ensure_command_ready(&self, command: &str) -> Result<(), String> {
        let owners_opt = {
            let map = self.command_map.read().await;
            map.get(command).cloned()
        };

        let mut owners = owners_opt.unwrap_or_default();

        if owners.is_empty() {
            owners = self.owners_from_activation_events(command).await;
        }

        if owners.is_empty() {
            return Ok(());
        }

        owners.sort();
        owners.dedup();

        for owner in owners {
            if owner == Self::CORE_COMMAND_OWNER {
                continue;
            }

            if !self.is_extension_active(&owner).await {
                self.load_extension(&owner).await?;
            }
        }

        Ok(())
    }

    fn read_extension_manifest(&self, extension_path: &Path) -> Result<ExtensionInfo, String> {
        let manifest_path = extension_path.join("package.json");

        let content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;

        let manifest: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?;

        let name = manifest
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'name' field")?
            .to_string();

        let version =
            manifest.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();

        let publisher =
            manifest.get("publisher").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();

        let description =
            manifest.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());

        let extension_id = format!("{}.{}", publisher, name);

        let categories = manifest
            .get("categories")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let mut dependencies = manifest
            .get("extensionDependencies")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        if let Some(pack) = manifest.get("extensionPack").and_then(|v| v.as_array()) {
            for item in pack {
                if let Some(value) = item.as_str() {
                    dependencies.push(value.to_string());
                }
            }
        }

        dependencies.sort();
        dependencies.dedup();

        let activation_events = manifest
            .get("activationEvents")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let commands = manifest
            .get("contributes")
            .and_then(|v| v.get("commands"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        item.get("command").and_then(|c| c.as_str()).map(|s| s.to_string())
                    })
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let repository = manifest.get("repository").and_then(|value| {
            if let Some(url) = value.as_str() {
                Some(url.to_string())
            } else if let Some(obj) = value.as_object() {
                obj.get("url").and_then(|u| u.as_str()).map(|s| s.to_string())
            } else {
                None
            }
        });

        let contributes: Option<ExtensionContributes> =
            manifest.get("contributes").map(|v| ExtensionContributes::from_json(v));

        Ok(ExtensionInfo {
            id: extension_id,
            name,
            version,
            publisher,
            description,
            enabled: true,
            active: false,
            categories,
            dependencies,
            repository,
            activation_events,
            commands,
            contributes,
        })
    }

    pub(super) async fn load_auto_start_extensions(&self) -> Result<(), String> {
        let state = self.state.read().await;
        let extensions: Vec<_> =
            state.installed_extensions.iter().filter(|ext| ext.enabled).cloned().collect();
        drop(state);

        for ext_info in extensions {
            if let Err(e) = self.load_extension_internal(&ext_info.id).await {
                eprintln!("[SessionManager] Failed to load extension {}: {}", ext_info.id, e);
            }
        }

        Ok(())
    }

    pub async fn load_extension(&self, extension_id: &str) -> Result<(), String> {
        self.load_extension_internal(extension_id).await?;

        self.mark_extension_active(extension_id, true).await;
        self.emit_event(SessionEvent::ExtensionLoaded { extension_id: extension_id.to_string() });

        Ok(())
    }

    async fn load_extension_internal(&self, extension_id: &str) -> Result<(), String> {
        let extension_path = self.app_dirs.extensions_dir.join(extension_id);

        if !extension_path.exists() {
            return Err(format!("Extension directory not found: {:?}", extension_path));
        }

        println!("[SessionManager] Loading extension: {}", extension_id);

        let payload = json!({ "extensionId": extension_id });
        self.ipc_manager.request("main", "activate-extension", payload).await?;

        Ok(())
    }

    pub async fn unload_extension(&self, extension_id: &str) -> Result<(), String> {
        println!("[SessionManager] Unloading extension: {}", extension_id);

        let payload = json!({ "extensionId": extension_id });
        self.ipc_manager.request("main", "deactivate-extension", payload).await?;

        self.mark_extension_active(extension_id, false).await;
        self.emit_event(SessionEvent::ExtensionUnloaded { extension_id: extension_id.to_string() });
        self.remove_extension_status_bar_items(extension_id).await;

        Ok(())
    }

    pub async fn delete_extension(&self, extension_id: &str) -> Result<(), String> {
        println!("[SessionManager] Deleting extension: {}", extension_id);

        let _ = self.unload_extension(extension_id).await;

        let payload = json!({ "extensionId": extension_id });
        let response = self.ipc_manager.request("main", "uninstall-extension", payload).await?;

        let success = response.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        if !success {
            let error = response
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error returned from extension host");
            return Err(format!("Extension host failed to uninstall {}: {}", extension_id, error));
        }

        let extension_path = self.app_dirs.extensions_dir.join(extension_id);
        if let Err(e) = remove_dir_if_exists(&extension_path) {
            eprintln!(
                "[SessionManager] Failed to delete extension directory {:?}: {}",
                extension_path, e
            );
        }

        let storage_dir = self.app_dirs.storage_dir.join(extension_id);
        if let Err(e) = remove_dir_if_exists(&storage_dir) {
            eprintln!(
                "[SessionManager] Failed to delete storage directory {:?}: {}",
                storage_dir, e
            );
        }

        let logs_dir = self.app_dirs.logs_dir.join(extension_id);
        if let Err(e) = remove_dir_if_exists(&logs_dir) {
            eprintln!("[SessionManager] Failed to delete log directory {:?}: {}", logs_dir, e);
        }

        let mut state = self.state.write().await;
        state.installed_extensions.retain(|ext| ext.id != extension_id);
        state.active_extensions.retain(|ext| ext.id != extension_id);
        let extensions = state.installed_extensions.clone();
        drop(state);

        self.remove_extension_commands(extension_id).await;
        self.publish_command_list().await;
        self.remove_extension_status_bar_items(extension_id).await;

        self.emit_event(SessionEvent::ExtensionDeleted { extension_id: extension_id.to_string() });
        self.emit_event(SessionEvent::ExtensionsChanged { extensions });

        Ok(())
    }

    async fn mark_extension_active(&self, extension_id: &str, active: bool) {
        let mut state = self.state.write().await;

        if let Some(ext) = state.installed_extensions.iter_mut().find(|e| e.id == extension_id) {
            ext.active = active;
        }

        if active {
            if let Some(ext) = state.installed_extensions.iter().find(|e| e.id == extension_id) {
                let ext_clone = ext.clone();
                if !state.active_extensions.iter().any(|e| e.id == extension_id) {
                    state.active_extensions.push(ext_clone);
                }
            }
        } else {
            state.active_extensions.retain(|e| e.id != extension_id);
        }

        let state_clone = state.clone();
        drop(state);

        self.emit_event(SessionEvent::StateChanged { state: state_clone });
    }

    pub async fn get_state(&self) -> SessionState {
        self.state.read().await.clone()
    }

    pub async fn get_installed_extensions(&self) -> Vec<ExtensionInfo> {
        self.state.read().await.installed_extensions.clone()
    }

    pub async fn get_active_extensions(&self) -> Vec<ExtensionInfo> {
        self.state.read().await.active_extensions.clone()
    }

    pub async fn install_vsix_package(
        &self, vsix_path: String,
    ) -> Result<InstalledExtension, String> {
        let extensions_dir = self.app_dirs.extensions_dir.clone();
        let installed =
            tokio::task::spawn_blocking(move || install_vsix(vsix_path, extensions_dir))
                .await
                .map_err(|e| format!("Task failed: {}", e))??;

        let dependency_installs =
            ensure_extension_dependencies(&installed.dependencies, &self.app_dirs).await?;

        if !dependency_installs.is_empty() {
            let ids: Vec<String> = dependency_installs.iter().map(|ext| ext.id.clone()).collect();
            println!("[Extensions] Installed dependencies for {}: {:?}", installed.id, ids);
        }

        self.reload_extensions_after_install().await?;
        self.emit_event(SessionEvent::ExtensionInstalled { extension_id: installed.id.clone() });

        Ok(installed)
    }

    pub async fn install_marketplace_extension(
        &self, publisher: String, name: String, version: String,
    ) -> Result<InstalledExtension, String> {
        let vsix_path = marketplace::download_extension(publisher, name, version).await?;
        let vsix_string = vsix_path.to_string_lossy().to_string();
        let installed = self.install_vsix_package(vsix_string).await;

        if let Err(err) = fs::remove_file(&vsix_path) {
            if err.kind() != ErrorKind::NotFound {
                eprintln!("[Extensions] Failed to delete temporary VSIX {:?}: {}", vsix_path, err);
            }
        }

        installed
    }

    pub async fn list_extensions_detailed(&self) -> Result<Vec<InstalledExtension>, String> {
        let extensions_dir = self.app_dirs.extensions_dir.clone();
        tokio::task::spawn_blocking(move || {
            if !extensions_dir.exists() {
                return Ok(Vec::new());
            }

            let entries = fs::read_dir(&extensions_dir)
                .map_err(|e| format!("Failed to read extensions directory: {}", e))?;
            let mut extensions = Vec::new();

            for entry in entries {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();

                if !path.is_dir() {
                    continue;
                }

                match Self::scan_extension_dir(&path) {
                    Ok(extension) => extensions.push(extension),
                    Err(err) => eprintln!(
                        "[SessionManager] Failed to read installed extension at {:?}: {}",
                        path, err
                    ),
                }
            }

            extensions.sort_by(|left, right| left.id.cmp(&right.id));
            Ok(extensions)
        })
        .await
        .map_err(|e| format!("Failed to list extensions: {}", e))?
    }

    pub async fn get_extension_contributions_detailed(
        &self,
    ) -> Result<Vec<ExtensionContribution>, String> {
        let extensions = self.list_extensions_detailed().await?;

        Ok(extensions
            .into_iter()
            .filter_map(|ext| {
                let contributes = ext.contributes.clone()?;
                Some(ExtensionContribution {
                    extension_id: ext.id,
                    extension_name: ext.name,
                    contributes: Some(contributes),
                })
            })
            .collect())
    }

    pub async fn get_extension_tree_children(
        &self, view_id: String, element: Option<serde_json::Value>,
    ) -> Result<Vec<serde_json::Value>, String> {
        let response = self
            .ipc_manager
            .request(
                "main",
                "treeView:getChildren",
                json!({
                    "viewId": view_id,
                    "element": element,
                }),
            )
            .await?;

        let children_value = if let Some(obj) = response.as_object() {
            let success = obj.get("success").and_then(|v| v.as_bool()).unwrap_or(true);
            let error = obj.get("error").and_then(|v| v.as_str()).map(|s| s.to_string());
            let children =
                obj.get("children").cloned().unwrap_or(serde_json::Value::Array(Vec::new()));
            if success {
                children
            } else {
                return Err(error.unwrap_or_else(|| "Tree provider error".to_string()));
            }
        } else {
            response
        };

        let elements = match children_value {
            serde_json::Value::Array(arr) => arr,
            serde_json::Value::Null => Vec::new(),
            other => {
                return Err(format!("Invalid tree view response format: {:?}", other));
            }
        };

        let mut results = Vec::with_capacity(elements.len());

        for element_value in elements {
            let element_clone = element_value.clone();
            let item_value =
                match self.get_extension_tree_item(view_id.clone(), element_value).await {
                    Ok(item) => item,
                    Err(_) => element_clone.clone(),
                };

            results.push(json!({
                "element": element_clone,
                "item": item_value,
            }));
        }

        Ok(results)
    }

    pub async fn get_extension_tree_item(
        &self, view_id: String, element: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let response = self
            .ipc_manager
            .request(
                "main",
                "treeView:getTreeItem",
                json!({
                    "viewId": view_id,
                    "element": element,
                }),
            )
            .await?;

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

    pub async fn execute_extension_command(
        &self, command: String, args: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        self.ensure_command_ready(&command).await?;
        self.ipc_manager
            .request(
                "main",
                "executeCommand",
                json!({
                    "command": command,
                    "args": args,
                }),
            )
            .await
    }

    pub async fn notify_extension_tree_event(
        &self, view_id: String, event: String, element: Option<serde_json::Value>,
        selection: Option<Vec<serde_json::Value>>, visible: Option<bool>,
    ) -> Result<(), String> {
        let mut payload = serde_json::Map::new();
        payload.insert("viewId".to_string(), serde_json::Value::String(view_id));
        payload.insert("event".to_string(), serde_json::Value::String(event));

        if let Some(element) = element {
            payload.insert("element".to_string(), element);
        }

        if let Some(selection) = selection {
            payload.insert("selection".to_string(), serde_json::Value::Array(selection));
        }

        if let Some(visible) = visible {
            payload.insert("visible".to_string(), serde_json::Value::Bool(visible));
        }

        let response = self
            .ipc_manager
            .request("main", "treeView:event", serde_json::Value::Object(payload))
            .await?;

        if response.get("success").and_then(|v| v.as_bool()).unwrap_or(true) {
            Ok(())
        } else {
            let err =
                response.get("error").and_then(|v| v.as_str()).unwrap_or("Tree view event failed");
            Err(err.to_string())
        }
    }

    async fn reload_extensions_after_install(&self) -> Result<(), String> {
        if let Err(err) = self.ipc_manager.request("main", "reload-extensions", json!({})).await {
            eprintln!("[Extensions] Failed to request extension host reload: {}", err);
        }

        self.scan_and_emit_extensions().await
    }

    fn scan_extension_dir(extension_path: &Path) -> Result<InstalledExtension, String> {
        let manifest = read_extension_manifest_from_disk(extension_path)?;
        let mut dependencies = manifest.extension_dependencies.clone().unwrap_or_default();

        if let Some(pack) = manifest.extension_pack.clone() {
            dependencies.extend(pack);
        }

        dependencies.sort();
        dependencies.dedup();

        Ok(InstalledExtension {
            id: format!("{}.{}", manifest.publisher, manifest.name),
            name: manifest.name,
            version: manifest.version,
            publisher: manifest.publisher,
            description: manifest.description,
            path: extension_path.to_string_lossy().to_string(),
            contributes: manifest.contributes.as_ref().map(|v| ExtensionContributes::from_json(v)),
            dependencies,
            categories: manifest.categories.unwrap_or_default(),
            repository: manifest.repository.as_ref().and_then(extract_repository_url),
        })
    }
}

fn remove_dir_if_exists(path: &Path) -> Result<(), std::io::Error> {
    if !path.exists() {
        return Ok(());
    }

    match fs::remove_dir_all(path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

fn install_vsix(vsix_path: String, extensions_root: PathBuf) -> Result<InstalledExtension, String> {
    let file =
        fs::File::open(&vsix_path).map_err(|e| format!("Failed to open .vsix file: {}", e))?;

    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read .vsix archive: {}", e))?;

    let mut manifest: Option<ExtensionManifest> = None;
    let mut extension_folder: Option<String> = None;

    for i in 0..archive.len() {
        let mut file =
            archive.by_index(i).map_err(|e| format!("Failed to read archive entry: {}", e))?;

        let file_path = file.name().to_string();

        if file_path == "extension/package.json" {
            let mut contents = String::new();
            io::Read::read_to_string(&mut file, &mut contents)
                .map_err(|e| format!("Failed to read package.json: {}", e))?;

            manifest = Some(
                serde_json::from_str(&contents)
                    .map_err(|e| format!("Failed to parse package.json: {}", e))?,
            );
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

    let file =
        fs::File::open(&vsix_path).map_err(|e| format!("Failed to reopen .vsix file: {}", e))?;

    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read .vsix archive: {}", e))?;

    for i in 0..archive.len() {
        let mut file =
            archive.by_index(i).map_err(|e| format!("Failed to read archive entry: {}", e))?;

        let path = file.name().to_string();

        // Security: validate that the path does not contain directory traversal
        if path.contains("..") {
            continue; // Skip paths with parent directory references
        }

        if let Some(relative_path) = path.strip_prefix(&format!("{}/", extension_folder)) {
            let outpath = install_path.join(relative_path);

            // Security: verify the canonicalized output path is within the install directory
            // This prevents Zip Slip attacks where archive entries contain paths like "../../../etc/passwd"
            if let Some(parent) = outpath.parent() {
                // Canonicalize parent if it exists; for new dirs, verify manually
                let canonical_install =
                    fs::canonicalize(&install_path).unwrap_or_else(|_| install_path.clone());
                if parent.exists() {
                    if let Ok(canonical_parent) = fs::canonicalize(parent) {
                        if !canonical_parent.starts_with(&canonical_install) {
                            eprintln!(
                                "[Extensions] Skipping path outside install dir: {}",
                                relative_path
                            );
                            continue;
                        }
                    }
                } else {
                    // Parent doesn't exist yet — verify the path itself doesn't escape
                    let clean_relative = Path::new(relative_path);
                    let mut depth = 0isize;
                    for component in clean_relative.components() {
                        match component {
                            std::path::Component::ParentDir => {
                                depth -= 1;
                                if depth < 0 {
                                    eprintln!(
                                        "[Extensions] Skipping path with traversal: {}",
                                        relative_path
                                    );
                                    continue;
                                }
                            }
                            std::path::Component::Normal(_) => depth += 1,
                            _ => {}
                        }
                    }
                }

                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }

            if file.is_dir() {
                fs::create_dir_all(&outpath)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            } else {
                // Enforce maximum single file size of 500MB
                const MAX_SINGLE_FILE_SIZE: u64 = 500 * 1024 * 1024;
                if file.size() > MAX_SINGLE_FILE_SIZE {
                    eprintln!(
                        "[Extensions] Skipping oversized file: {} ({} bytes)",
                        relative_path,
                        file.size()
                    );
                    continue;
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
    let repository = manifest.repository.as_ref().and_then(extract_repository_url);

    Ok(InstalledExtension {
        id: extension_id,
        name: manifest.name,
        version: manifest.version,
        publisher: manifest.publisher,
        description: manifest.description,
        path: install_path.to_string_lossy().to_string(),
        contributes: manifest.contributes.as_ref().map(|v| ExtensionContributes::from_json(v)),
        dependencies: dependency_list,
        categories,
        repository,
    })
}

fn validate_manifest(manifest: &ExtensionManifest) -> Result<(), String> {
    if manifest.name.is_empty() {
        return Err("Extension name is required".to_string());
    }

    if manifest.version.is_empty() {
        return Err("Extension version is required".to_string());
    }

    if manifest.publisher.is_empty() {
        return Err("Extension publisher is required".to_string());
    }

    if !manifest.name.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '-') {
        return Err("Extension name must be lowercase alphanumeric with hyphens".to_string());
    }

    if Version::parse(&manifest.version).is_err() {
        return Err("Extension version must follow semantic versioning".to_string());
    }

    Ok(())
}

fn read_existing_extension_version(path: &Path) -> Option<Version> {
    let manifest = read_extension_manifest_from_disk(path).ok()?;
    Version::parse(&manifest.version).ok()
}

fn read_extension_manifest_from_disk(path: &Path) -> Result<ExtensionManifest, String> {
    let manifest_path = path.join("package.json");
    let contents =
        fs::read_to_string(manifest_path).map_err(|e| format!("Failed to read manifest: {}", e))?;
    serde_json::from_str(&contents).map_err(|e| format!("Failed to parse manifest: {}", e))
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
    dependencies: &[String], app_dirs: &crate::config::AppDirectories,
) -> Result<Vec<InstalledExtension>, String> {
    if dependencies.is_empty() {
        return Ok(Vec::new());
    }

    let extensions_root = app_dirs.extensions_dir.clone();
    let mut visited = HashSet::new();
    let mut queue: VecDeque<String> = dependencies.iter().cloned().collect();
    let mut installed = Vec::new();

    while let Some(dep_id) = queue.pop_front() {
        if !visited.insert(dep_id.clone()) || dep_id.trim().is_empty() {
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
        let vsix_path =
            marketplace::download_extension(publisher, name, details.version.clone()).await?;
        let vsix_string = vsix_path.to_string_lossy().to_string();
        let extensions_root_clone = extensions_root.clone();

        let installed_dep =
            tokio::task::spawn_blocking(move || install_vsix(vsix_string, extensions_root_clone))
                .await
                .map_err(|e| format!("Task failed: {}", e))??;

        if let Err(err) = fs::remove_file(&vsix_path) {
            if err.kind() != ErrorKind::NotFound {
                eprintln!("[Extensions] Failed to delete temporary VSIX {:?}: {}", vsix_path, err);
            }
        }

        for nested in &installed_dep.dependencies {
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
    let manifest = match read_extension_manifest_from_disk(path) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let mut deps = manifest.extension_dependencies.unwrap_or_default();

    if let Some(pack) = manifest.extension_pack {
        deps.extend(pack);
    }

    deps.sort();
    deps.dedup();
    deps
}
