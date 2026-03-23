use super::{ExtensionInfo, SessionEvent, SessionManager, SessionState};
use serde_json::json;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

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

        println!(
            "[SessionManager] Found {} installed extensions",
            extensions.len()
        );

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
        self.state
            .read()
            .await
            .active_extensions
            .iter()
            .any(|ext| ext.id == extension_id)
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

        let version = manifest
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();

        let publisher = manifest
            .get("publisher")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let description = manifest
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

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
                        item.get("command")
                            .and_then(|c| c.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let repository = manifest.get("repository").and_then(|value| {
            if let Some(url) = value.as_str() {
                Some(url.to_string())
            } else if let Some(obj) = value.as_object() {
                obj.get("url")
                    .and_then(|u| u.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        });

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
        })
    }

    pub(super) async fn load_auto_start_extensions(&self) -> Result<(), String> {
        let state = self.state.read().await;
        let extensions: Vec<_> = state
            .installed_extensions
            .iter()
            .filter(|ext| ext.enabled)
            .cloned()
            .collect();
        drop(state);

        for ext_info in extensions {
            if let Err(e) = self.load_extension_internal(&ext_info.id).await {
                eprintln!(
                    "[SessionManager] Failed to load extension {}: {}",
                    ext_info.id, e
                );
            }
        }

        Ok(())
    }

    pub async fn load_extension(&self, extension_id: &str) -> Result<(), String> {
        self.load_extension_internal(extension_id).await?;

        self.mark_extension_active(extension_id, true).await;
        self.emit_event(SessionEvent::ExtensionLoaded {
            extension_id: extension_id.to_string(),
        });

        Ok(())
    }

    async fn load_extension_internal(&self, extension_id: &str) -> Result<(), String> {
        let extension_path = self.app_dirs.extensions_dir.join(extension_id);

        if !extension_path.exists() {
            return Err(format!(
                "Extension directory not found: {:?}",
                extension_path
            ));
        }

        println!("[SessionManager] Loading extension: {}", extension_id);

        let payload = json!({ "extensionId": extension_id });
        self.nng_manager
            .request("main", "activate-extension", payload)
            .await?;

        Ok(())
    }

    pub async fn unload_extension(&self, extension_id: &str) -> Result<(), String> {
        println!("[SessionManager] Unloading extension: {}", extension_id);

        let payload = json!({ "extensionId": extension_id });
        self.nng_manager
            .request("main", "deactivate-extension", payload)
            .await?;

        self.mark_extension_active(extension_id, false).await;
        self.emit_event(SessionEvent::ExtensionUnloaded {
            extension_id: extension_id.to_string(),
        });
        self.remove_extension_status_bar_items(extension_id).await;

        Ok(())
    }

    pub async fn delete_extension(&self, extension_id: &str) -> Result<(), String> {
        println!("[SessionManager] Deleting extension: {}", extension_id);

        let _ = self.unload_extension(extension_id).await;

        let payload = json!({ "extensionId": extension_id });
        let response = self
            .nng_manager
            .request("main", "uninstall-extension", payload)
            .await?;

        let success = response
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !success {
            let error = response
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error returned from extension host");
            return Err(format!(
                "Extension host failed to uninstall {}: {}",
                extension_id, error
            ));
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
            eprintln!(
                "[SessionManager] Failed to delete log directory {:?}: {}",
                logs_dir, e
            );
        }

        let mut state = self.state.write().await;
        state
            .installed_extensions
            .retain(|ext| ext.id != extension_id);
        state.active_extensions.retain(|ext| ext.id != extension_id);
        let extensions = state.installed_extensions.clone();
        drop(state);

        self.remove_extension_commands(extension_id).await;
        self.publish_command_list().await;
        self.remove_extension_status_bar_items(extension_id).await;

        self.emit_event(SessionEvent::ExtensionDeleted {
            extension_id: extension_id.to_string(),
        });
        self.emit_event(SessionEvent::ExtensionsChanged { extensions });

        Ok(())
    }

    async fn mark_extension_active(&self, extension_id: &str, active: bool) {
        let mut state = self.state.write().await;

        if let Some(ext) = state
            .installed_extensions
            .iter_mut()
            .find(|e| e.id == extension_id)
        {
            ext.active = active;
        }

        if active {
            if let Some(ext) = state
                .installed_extensions
                .iter()
                .find(|e| e.id == extension_id)
            {
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
