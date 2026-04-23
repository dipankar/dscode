//! Extension lifecycle management: scanning, loading, unloading, installing, and deleting.
//!
//! This module provides both standalone functions for extension management
//! and types for extension metadata.

#![allow(dead_code)]

use crate::contributions::ExtensionContributes;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use zip::ZipArchive;

/// Information about an extension visible to the session layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub active: bool,
    pub categories: Vec<String>,
    pub dependencies: Vec<String>,
    pub repository: Option<String>,
    pub activation_events: Vec<String>,
    pub commands: Vec<String>,
    #[serde(default)]
    pub contributes: Option<ExtensionContributes>,
}

/// Installed extension metadata.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
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

/// Extension contribution summary.
#[derive(Debug, Serialize, Clone)]
pub struct ExtensionContribution {
    pub extension_id: String,
    pub extension_name: String,
    pub contributes: Option<ExtensionContributes>,
}

// Internal manifest struct for VSIX parsing
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

/// Read an extension manifest from disk and return an `ExtensionInfo`.
pub(crate) fn read_extension_manifest(extension_path: &Path) -> Result<ExtensionInfo, String> {
    let manifest_path = extension_path.join("package.json");

    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;

    let manifest: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse manifest: {}", e))?;

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

    let contributes: Option<ExtensionContributes> = manifest
        .get("contributes")
        .map(ExtensionContributes::from_json);

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

/// Scan an extension directory and return an `InstalledExtension`.
pub(crate) fn scan_extension_dir(extension_path: &Path) -> Result<InstalledExtension, String> {
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
        contributes: manifest
            .contributes
            .as_ref()
            .map(ExtensionContributes::from_json),
        dependencies,
        categories: manifest.categories.unwrap_or_default(),
        repository: manifest
            .repository
            .as_ref()
            .and_then(extract_repository_url),
    })
}

/// List all installed extensions from a given extensions directory.
pub(crate) fn list_installed_extensions(
    extensions_dir: &Path,
) -> Result<Vec<InstalledExtension>, String> {
    if !extensions_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(extensions_dir)
        .map_err(|e| format!("Failed to read extensions directory: {}", e))?;
    let mut extensions = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        match scan_extension_dir(&path) {
            Ok(extension) => extensions.push(extension),
            Err(err) => {
                warn!(path = ?path, error = %err, "Failed to read installed extension");
            }
        }
    }

    extensions.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(extensions)
}

/// Install a VSIX package from a file path.
pub(crate) fn install_vsix(
    vsix_path: String,
    extensions_root: PathBuf,
) -> Result<InstalledExtension, String> {
    let file =
        fs::File::open(&vsix_path).map_err(|e| format!("Failed to open .vsix file: {}", e))?;

    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read .vsix archive: {}", e))?;

    let mut manifest: Option<ExtensionManifest> = None;
    let mut extension_folder: Option<String> = None;

    for i in 0..archive.len() {
        let mut zip_file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;

        let file_path = zip_file.name().to_string();

        if file_path == "extension/package.json" {
            let mut contents = String::new();
            io::Read::read_to_string(&mut zip_file, &mut contents)
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
        let mut zip_file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;

        let path = zip_file.name().to_string();

        // Security: validate that the path does not contain directory traversal
        if path.contains("..") {
            continue;
        }

        if let Some(relative_path) = path.strip_prefix(&format!("{}/", extension_folder)) {
            let outpath = install_path.join(relative_path);

            // Security: verify the canonicalized output path is within the install directory
            if let Some(parent) = outpath.parent() {
                let canonical_install =
                    fs::canonicalize(&install_path).unwrap_or_else(|_| install_path.clone());
                if parent.exists() {
                    if let Ok(canonical_parent) = fs::canonicalize(parent) {
                        if !canonical_parent.starts_with(&canonical_install) {
                            warn!(path = %relative_path, "Skipping path outside install dir");
                            continue;
                        }
                    }
                } else {
                    let clean_relative = Path::new(relative_path);
                    let mut depth: isize = 0;
                    for component in clean_relative.components() {
                        match component {
                            std::path::Component::ParentDir => {
                                depth -= 1;
                                if depth < 0 {
                                    warn!(path = %relative_path, "Skipping path with traversal");
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

            if zip_file.is_dir() {
                fs::create_dir_all(&outpath)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            } else {
                // Enforce maximum single file size of 500MB
                const MAX_SINGLE_FILE_SIZE: u64 = 500 * 1024 * 1024;
                if zip_file.size() > MAX_SINGLE_FILE_SIZE {
                    warn!(
                        path = %relative_path,
                        size = zip_file.size(),
                        "Skipping oversized file"
                    );
                    continue;
                }

                let mut outfile = fs::File::create(&outpath)
                    .map_err(|e| format!("Failed to create file: {}", e))?;

                io::copy(&mut zip_file, &mut outfile)
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
        id: extension_id,
        name: manifest.name,
        version: manifest.version,
        publisher: manifest.publisher,
        description: manifest.description,
        path: install_path.to_string_lossy().to_string(),
        contributes: manifest
            .contributes
            .as_ref()
            .map(ExtensionContributes::from_json),
        dependencies: dependency_list,
        categories,
        repository,
    })
}

/// Ensure extension dependencies are installed.
pub(crate) async fn ensure_extension_dependencies(
    dependencies: &[String],
    extensions_dir: &Path,
) -> Result<Vec<InstalledExtension>, String> {
    if dependencies.is_empty() {
        return Ok(Vec::new());
    }

    let extensions_root = extensions_dir.to_path_buf();
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
                warn!(id = %dep_id, "Invalid dependency identifier");
                continue;
            }
        };

        info!(id = %dep_id, "Auto-installing dependency");

        let details =
            crate::marketplace::get_extension_details(publisher.clone(), name.clone()).await?;
        let vsix_path =
            crate::marketplace::download_extension(publisher, name, details.version.clone())
                .await?;
        let vsix_string = vsix_path.to_string_lossy().to_string();
        let extensions_root_clone = extensions_root.clone();

        let installed_dep =
            tokio::task::spawn_blocking(move || install_vsix(vsix_string, extensions_root_clone))
                .await
                .map_err(|e| format!("Task failed: {}", e))??;

        if let Err(err) = fs::remove_file(&vsix_path) {
            if err.kind() != io::ErrorKind::NotFound {
                warn!(path = ?vsix_path, error = %err, "Failed to delete temporary VSIX");
            }
        }

        for nested in &installed_dep.dependencies {
            queue.push_back(nested.clone());
        }

        installed.push(installed_dep);
    }

    Ok(installed)
}

fn remove_dir_if_exists(path: &Path) -> Result<(), io::Error> {
    if !path.exists() {
        return Ok(());
    }

    match fs::remove_dir_all(path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
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

    if !manifest
        .name
        .chars()
        .all(|c| c.is_lowercase() || c.is_numeric() || c == '-')
    {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contributions::ExtensionContributes;

    #[test]
    fn test_extension_contribution_creation() {
        let contrib = ExtensionContribution {
            extension_id: "publisher.my-extension".to_string(),
            extension_name: "My Extension".to_string(),
            contributes: None,
        };
        assert_eq!(contrib.extension_id, "publisher.my-extension");
        assert_eq!(contrib.extension_name, "My Extension");
        assert!(contrib.contributes.is_none());
    }

    #[test]
    fn test_extension_contribution_with_contributes() {
        let contributes = ExtensionContributes::default();
        let contrib = ExtensionContribution {
            extension_id: "pub.ext".to_string(),
            extension_name: "Ext".to_string(),
            contributes: Some(contributes),
        };
        assert!(contrib.contributes.is_some());
        let c = contrib.contributes.unwrap();
        assert!(c.commands.is_empty());
    }

    #[test]
    fn test_extension_info_serde_roundtrip() {
        let info = ExtensionInfo {
            id: "pub.test".to_string(),
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            publisher: "pub".to_string(),
            description: Some("A test extension".to_string()),
            enabled: true,
            active: false,
            categories: vec!["Programming Languages".to_string()],
            dependencies: vec![],
            repository: None,
            activation_events: vec![],
            commands: vec![],
            contributes: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: ExtensionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, info.id);
        assert_eq!(deserialized.name, info.name);
        assert_eq!(deserialized.version, info.version);
        assert_eq!(deserialized.publisher, info.publisher);
        assert_eq!(deserialized.description, info.description);
        assert_eq!(deserialized.enabled, info.enabled);
        assert_eq!(deserialized.active, info.active);
        assert_eq!(deserialized.categories, info.categories);
    }

    #[test]
    fn test_installed_extension_serde_roundtrip() {
        let ext = InstalledExtension {
            id: "pub.tool".to_string(),
            name: "tool".to_string(),
            version: "2.0.0".to_string(),
            publisher: "pub".to_string(),
            description: None,
            path: "/extensions/pub.tool".to_string(),
            contributes: None,
            dependencies: vec![],
            categories: vec![],
            repository: Some("https://github.com/pub/tool".to_string()),
        };
        let json = serde_json::to_string(&ext).unwrap();
        let deserialized: InstalledExtension = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, ext.id);
        assert_eq!(deserialized.name, ext.name);
        assert_eq!(deserialized.version, ext.version);
        assert_eq!(deserialized.path, ext.path);
        assert_eq!(deserialized.repository, ext.repository);
    }

    #[test]
    fn test_validate_manifest_valid() {
        let manifest = ExtensionManifest {
            name: "my-ext".to_string(),
            display_name: None,
            version: "1.0.0".to_string(),
            publisher: "pub".to_string(),
            description: None,
            main: None,
            activation_events: None,
            engines: None,
            contributes: None,
            dependencies: None,
            dev_dependencies: None,
            extension_dependencies: None,
            extension_pack: None,
            categories: None,
            repository: None,
        };
        assert!(validate_manifest(&manifest).is_ok());
    }

    #[test]
    fn test_validate_manifest_empty_name() {
        let manifest = ExtensionManifest {
            name: "".to_string(),
            display_name: None,
            version: "1.0.0".to_string(),
            publisher: "pub".to_string(),
            description: None,
            main: None,
            activation_events: None,
            engines: None,
            contributes: None,
            dependencies: None,
            dev_dependencies: None,
            extension_dependencies: None,
            extension_pack: None,
            categories: None,
            repository: None,
        };
        assert!(validate_manifest(&manifest).is_err());
    }

    #[test]
    fn test_validate_manifest_empty_version() {
        let manifest = ExtensionManifest {
            name: "ext".to_string(),
            display_name: None,
            version: "".to_string(),
            publisher: "pub".to_string(),
            description: None,
            main: None,
            activation_events: None,
            engines: None,
            contributes: None,
            dependencies: None,
            dev_dependencies: None,
            extension_dependencies: None,
            extension_pack: None,
            categories: None,
            repository: None,
        };
        assert!(validate_manifest(&manifest).is_err());
    }

    #[test]
    fn test_validate_manifest_empty_publisher() {
        let manifest = ExtensionManifest {
            name: "ext".to_string(),
            display_name: None,
            version: "1.0.0".to_string(),
            publisher: "".to_string(),
            description: None,
            main: None,
            activation_events: None,
            engines: None,
            contributes: None,
            dependencies: None,
            dev_dependencies: None,
            extension_dependencies: None,
            extension_pack: None,
            categories: None,
            repository: None,
        };
        assert!(validate_manifest(&manifest).is_err());
    }

    #[test]
    fn test_validate_manifest_invalid_version() {
        let manifest = ExtensionManifest {
            name: "ext".to_string(),
            display_name: None,
            version: "not-semver".to_string(),
            publisher: "pub".to_string(),
            description: None,
            main: None,
            activation_events: None,
            engines: None,
            contributes: None,
            dependencies: None,
            dev_dependencies: None,
            extension_dependencies: None,
            extension_pack: None,
            categories: None,
            repository: None,
        };
        assert!(validate_manifest(&manifest).is_err());
    }

    #[test]
    fn test_parse_extension_id() {
        assert_eq!(
            parse_extension_id("pub.name"),
            Some(("pub".to_string(), "name".to_string()))
        );
        assert_eq!(parse_extension_id("single"), None);
        assert_eq!(parse_extension_id(""), None);
    }

    #[test]
    fn test_extract_repository_url_string() {
        let value = serde_json::json!("https://github.com/test/repo");
        assert_eq!(
            extract_repository_url(&value),
            Some("https://github.com/test/repo".to_string())
        );
    }

    #[test]
    fn test_extract_repository_url_object() {
        let value = serde_json::json!({ "url": "https://github.com/test/repo" });
        assert_eq!(
            extract_repository_url(&value),
            Some("https://github.com/test/repo".to_string())
        );
    }

    #[test]
    fn test_extract_repository_url_invalid() {
        let value = serde_json::json!(42);
        assert_eq!(extract_repository_url(&value), None);
    }
}
