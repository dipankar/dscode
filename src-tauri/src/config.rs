use std::path::{Path, PathBuf};

use directories::BaseDirs;
use serde_json::Value;

/// Application directories derived from configuration.
#[derive(Debug, Clone)]
pub struct AppDirectories {
    pub extensions_dir: PathBuf,
    pub storage_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AppDirectories {
    /// Resolve directories from Tauri configuration and environment overrides.
    pub fn from_app_config(_config: &tauri::Config) -> Result<Self, String> {
        let user_paths = resolve_user_config_dirs();

        let extensions_pref = resolve_env_extensions_dir()
            .or_else(|| user_paths.extensions.clone())
            .unwrap_or_else(default_extensions_dir);

        let storage_pref = user_paths.storage.clone().unwrap_or_else(default_storage_dir);

        let logs_pref = user_paths.logs.clone().unwrap_or_else(default_logs_dir);

        let extensions_dir = normalize_path(extensions_pref)?;
        let storage_dir = normalize_path(storage_pref)?;
        let logs_dir = normalize_path(logs_pref)?;

        create_dir_if_needed(&extensions_dir, "extensions")?;
        create_dir_if_needed(&storage_dir, "storage")?;
        create_dir_if_needed(&logs_dir, "logs")?;

        Ok(Self {
            extensions_dir: canonicalize_or_original(extensions_dir),
            storage_dir: canonicalize_or_original(storage_dir),
            logs_dir: canonicalize_or_original(logs_dir),
        })
    }
}

fn resolve_env_extensions_dir() -> Option<PathBuf> {
    std::env::var("DSCODE_EXTENSIONS_DIR").ok().map(PathBuf::from)
}

fn resolve_user_config_dirs() -> UserConfiguredPaths {
    if let Some(base) = BaseDirs::new() {
        let config_path = base.home_dir().join(".dscode").join("config.json");
        if let Ok(contents) = std::fs::read_to_string(config_path) {
            if let Ok(value) = serde_json::from_str::<Value>(&contents) {
                let extensions =
                    extract_path(&value, &["extensionsDir", "extensions_dir"], Some("paths"));
                let storage = extract_path(&value, &["storageDir", "storage_dir"], Some("paths"));
                let logs = extract_path(&value, &["logsDir", "logs_dir"], Some("paths"));
                return UserConfiguredPaths { extensions, storage, logs };
            }
        }
    }

    UserConfiguredPaths::default()
}

fn default_extensions_dir() -> PathBuf {
    default_base_dir().join("extensions")
}

fn default_storage_dir() -> PathBuf {
    default_base_dir().join("storage")
}

fn default_logs_dir() -> PathBuf {
    default_base_dir().join("logs")
}

fn default_base_dir() -> PathBuf {
    if let Some(base) = BaseDirs::new() {
        base.home_dir().join(".dscode")
    } else if let Ok(home) = std::env::var("HOME") {
        Path::new(&home).join(".dscode")
    } else {
        std::env::temp_dir().join("dscode")
    }
}

fn normalize_path(path: PathBuf) -> Result<PathBuf, String> {
    let as_str = path.to_string_lossy();
    if as_str == "~" {
        if let Some(base) = BaseDirs::new() {
            return Ok(base.home_dir().to_path_buf());
        }
        if let Ok(home) = std::env::var("HOME") {
            return Ok(Path::new(&home).to_path_buf());
        }
        return Err("Unable to resolve home directory for '~' path".to_string());
    }

    if let Some(stripped) = as_str.strip_prefix("~/") {
        if let Some(base) = BaseDirs::new() {
            return Ok(base.home_dir().join(stripped));
        }
        if let Ok(home) = std::env::var("HOME") {
            return Ok(Path::new(&home).join(stripped));
        }
        return Err("Unable to resolve home directory for '~/...' path".to_string());
    }

    Ok(path)
}

fn create_dir_if_needed(path: &Path, label: &str) -> Result<(), String> {
    std::fs::create_dir_all(path)
        .map_err(|e| format!("Failed to create {} directory {:?}: {}", label, path, e))
}

fn canonicalize_or_original(path: PathBuf) -> PathBuf {
    std::fs::canonicalize(&path).unwrap_or(path)
}

#[derive(Default)]
struct UserConfiguredPaths {
    extensions: Option<PathBuf>,
    storage: Option<PathBuf>,
    logs: Option<PathBuf>,
}

fn extract_path(value: &Value, keys: &[&str], scoped: Option<&str>) -> Option<PathBuf> {
    for key in keys {
        if let Some(path) = value.get(*key).and_then(|v| v.as_str()) {
            return Some(PathBuf::from(path));
        }
    }

    if let Some(scope_key) = scoped {
        if let Some(scope) = value.get(scope_key) {
            for key in keys {
                if let Some(path) = scope.get(*key).and_then(|v| v.as_str()) {
                    return Some(PathBuf::from(path));
                }
            }
        }
    }

    None
}
