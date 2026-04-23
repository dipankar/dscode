use std::path::{Path, PathBuf};

use directories::BaseDirs;
use serde_json::Value;
use tracing::warn;

/// Application directories derived from configuration.
///
/// `AppDirectories` holds the resolved filesystem paths that DSCode uses to
/// store extensions, persistent data, and log files. Paths are resolved at
/// construction time from a priority chain of environment variables, user
/// configuration, and platform defaults.
///
/// # Resolution priority (highest to lowest)
///
/// 1. **Environment variable override** — e.g. `DSCODE_EXTENSIONS_DIR` for the
///    extensions directory.
/// 2. **User config file** — `~/.dscode/config.json`, which may specify
///    custom paths under a `"paths"` object.
/// 3. **Platform default** — `~/.dscode/extensions`, `~/.dscode/storage`, and
///    `~/.dscode/logs` respectively.
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
///
/// Both `camelCase` and `snake_case` keys are accepted.
///
/// # Invariants
///
/// - All three directory paths exist on disk after [`AppDirectories::resolve`]
///   returns `Ok`. Directories are created automatically if they do not exist.
/// - Tilde (`~`) prefixes in user-configured paths are expanded to the home
///   directory.
/// - Paths are canonicalised when possible; if canonicalisation fails (e.g.
///   the path does not yet exist on some platforms), the original path is
///   kept unchanged.
#[derive(Debug, Clone)]
pub struct AppDirectories {
    /// Directory where installed extensions are stored.
    ///
    /// Can be overridden by the `DSCODE_EXTENSIONS_DIR` environment variable.
    pub extensions_dir: PathBuf,
    /// Directory for persistent application storage (e.g. workspace state,
    /// user preferences).
    pub storage_dir: PathBuf,
    /// Directory where log files are written.
    pub logs_dir: PathBuf,
}

impl AppDirectories {
    /// Resolve directories from user configuration and environment overrides.
    ///
    /// The method applies the priority chain (environment variable > user
    /// config > platform default) for each of the three directory paths,
    /// expands `~` prefixes, creates directories that do not yet exist, and
    /// canonicalises the resulting paths.
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` if:
    /// - A `~` path cannot be expanded because the home directory is
    ///   unavailable.
    /// - A directory cannot be created due to a filesystem permission error.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use dscode_core::AppDirectories;
    ///
    /// let dirs = AppDirectories::resolve().expect("failed to resolve dirs");
    /// println!("Extensions: {:?}", dirs.extensions_dir);
    /// ```
    pub fn resolve() -> Result<Self, String> {
        let user_paths = resolve_user_config_dirs();

        let extensions_pref = resolve_env_extensions_dir()
            .or_else(|| user_paths.extensions.clone())
            .unwrap_or_else(default_extensions_dir);

        let storage_pref = user_paths
            .storage
            .clone()
            .unwrap_or_else(default_storage_dir);

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
    std::env::var("DSCODE_EXTENSIONS_DIR")
        .ok()
        .map(PathBuf::from)
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
                return UserConfiguredPaths {
                    extensions,
                    storage,
                    logs,
                };
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
    use std::sync::Mutex;

    /// Mutex to serialize env-override tests since std::env::set_var is process-global.
    static ENV_TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_app_directories_resolve() {
        // This test verifies that AppDirectories can resolve without error
        let dirs = AppDirectories::resolve();
        assert!(dirs.is_ok(), "AppDirectories::resolve() should succeed");

        let dirs = dirs.unwrap();
        assert!(
            dirs.extensions_dir.to_string_lossy().contains("dscode")
                || dirs.extensions_dir.to_string_lossy().contains("DSCode")
        );
        assert!(
            dirs.storage_dir.to_string_lossy().contains("dscode")
                || dirs.storage_dir.to_string_lossy().contains("DSCode")
        );
        assert!(
            dirs.logs_dir.to_string_lossy().contains("dscode")
                || dirs.logs_dir.to_string_lossy().contains("DSCode")
        );
    }

    #[test]
    fn test_env_override_extensions_dir() {
        let _lock = ENV_TEST_MUTEX.lock().unwrap();
        let custom_dir =
            std::env::temp_dir().join(format!("dscode-test-ext-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&custom_dir);

        std::env::set_var("DSCODE_EXTENSIONS_DIR", &custom_dir);
        let dirs = AppDirectories::resolve().expect("resolve should work with env override");
        assert!(
            dirs.extensions_dir.starts_with(&custom_dir)
                || dirs.extensions_dir == canonicalize_or_original(custom_dir.clone())
        );

        std::env::remove_var("DSCODE_EXTENSIONS_DIR");
        let _ = std::fs::remove_dir_all(&custom_dir);
    }

    #[test]
    fn test_env_override_set_resolve_clear() {
        let _lock = ENV_TEST_MUTEX.lock().unwrap();
        let custom_dir =
            std::env::temp_dir().join(format!("dscode-test-cycle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&custom_dir);

        std::env::set_var("DSCODE_EXTENSIONS_DIR", &custom_dir);
        let dirs_with_env =
            AppDirectories::resolve().expect("resolve should work with env override");
        assert!(
            dirs_with_env.extensions_dir.starts_with(&custom_dir)
                || dirs_with_env.extensions_dir == canonicalize_or_original(custom_dir.clone()),
            "Extensions dir should use env override"
        );

        std::env::remove_var("DSCODE_EXTENSIONS_DIR");
        let dirs_without_env =
            AppDirectories::resolve().expect("resolve should work without env override");
        assert!(
            !dirs_without_env.extensions_dir.starts_with(&custom_dir),
            "Extensions dir should revert to default after clearing env"
        );

        let _ = std::fs::remove_dir_all(&custom_dir);
    }

    #[test]
    fn test_resolve_creates_directories() {
        // resolve() should create the directories if they don't exist
        let dirs = AppDirectories::resolve().unwrap();
        assert!(
            dirs.extensions_dir.exists(),
            "extensions_dir should exist after resolve"
        );
        assert!(
            dirs.storage_dir.exists(),
            "storage_dir should exist after resolve"
        );
        assert!(
            dirs.logs_dir.exists(),
            "logs_dir should exist after resolve"
        );
    }

    #[test]
    fn test_normalize_path_tilde() {
        // Tilde should expand to the home directory
        let expanded = normalize_path(PathBuf::from("~")).unwrap();
        assert!(
            !expanded.to_string_lossy().starts_with('~'),
            "Tilde should be expanded"
        );

        let expanded_prefix = normalize_path(PathBuf::from("~/subdir")).unwrap();
        assert!(
            !expanded_prefix.to_string_lossy().starts_with('~'),
            "Tilde prefix should be expanded"
        );
        assert!(expanded_prefix.to_string_lossy().contains("subdir"));
    }

    #[test]
    fn test_extract_path_from_nested_json() {
        // extract_path should find keys in scoped objects
        let value: Value =
            serde_json::from_str(r#"{"paths": {"extensionsDir": "/custom/ext"}}"#).unwrap();
        let result = extract_path(&value, &["extensionsDir", "extensions_dir"], Some("paths"));
        assert_eq!(result, Some(PathBuf::from("/custom/ext")));

        // Fallback to top-level key if not in scope
        let value2: Value = serde_json::from_str(r#"{"extensionsDir": "/top/ext"}"#).unwrap();
        let result2 = extract_path(&value2, &["extensionsDir", "extensions_dir"], Some("paths"));
        assert_eq!(result2, Some(PathBuf::from("/top/ext")));

        // Missing key returns None
        let value3: Value = serde_json::from_str(r#"{"paths": {}}"#).unwrap();
        let result3 = extract_path(&value3, &["extensionsDir", "extensions_dir"], Some("paths"));
        assert_eq!(result3, None);
    }

    #[test]
    fn test_default_base_dir_contains_dscode() {
        let base = default_base_dir();
        let base_str = base.to_string_lossy();
        assert!(
            base_str.contains("dscode"),
            "Default base dir should contain 'dscode': {:?}",
            base
        );
    }
}
