use std::path::{Path, PathBuf};

use directories::BaseDirs;
use serde_json::Value;
use tracing::warn;

/// Application directories derived from configuration.
///
/// Resolves platform-appropriate directories for extensions, storage, and logs.
/// Supports environment variable overrides and user configuration files.
///
/// # Environment Variables
///
/// - `DSCODE_EXTENSIONS_DIR` — Override the extensions directory
///
/// # User Configuration
///
/// Reads `~/.dscode/config.json` for custom paths:
///
/// ```json
/// {
///   "paths": {
///     "extensionsDir": "/custom/extensions",
///     "storageDir": "/custom/storage",
///     "logsDir": "/custom/logs"
///   }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AppDirectories {
    /// Directory for installed extensions
    pub extensions_dir: PathBuf,
    /// Directory for persistent storage
    pub storage_dir: PathBuf,
    /// Directory for log files
    pub logs_dir: PathBuf,
}

impl AppDirectories {
    /// Resolve directories from user configuration and environment overrides.
    ///
    /// Priority order for each directory:
    /// 1. Environment variable override (e.g., `DSCODE_EXTENSIONS_DIR`)
    /// 2. User config file (`~/.dscode/config.json`)
    /// 3. Platform default (`~/.dscode/extensions`, `~/.dscode/storage`, `~/.dscode/logs`)
    pub fn resolve() -> Result<Self, String> {
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
        warn!("Unable to resolve home directory, using temp directory");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_directories_resolve() {
        // This test verifies that AppDirectories can resolve without error
        let dirs = AppDirectories::resolve();
        assert!(dirs.is_ok(), "AppDirectories::resolve() should succeed");

        let dirs = dirs.unwrap();
        assert!(dirs.extensions_dir.to_string_lossy().contains("dscode") || dirs.extensions_dir.to_string_lossy().contains("DSCode"));
        assert!(dirs.storage_dir.to_string_lossy().contains("dscode") || dirs.storage_dir.to_string_lossy().contains("DSCode"));
        assert!(dirs.logs_dir.to_string_lossy().contains("dscode") || dirs.logs_dir.to_string_lossy().contains("DSCode"));
    }

    #[test]
    fn test_env_override_extensions_dir() {
        // Set the environment variable
        let custom_dir = std::env::temp_dir().join("dscode-test-extensions");
        std::env::set_var("DSCODE_EXTENSIONS_DIR", &custom_dir);

        let dirs = AppDirectories::resolve().unwrap();
        assert!(dirs.extensions_dir.starts_with(&custom_dir) || dirs.extensions_dir == canonicalize_or_original(custom_dir.clone()));

        // Clean up
        std::env::remove_var("DSCODE_EXTENSIONS_DIR");
        let _ = std::fs::remove_dir_all(&custom_dir);
    }
}