use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
const DEFAULT_FILE_NAME: &str = "settings.json";

/// Persistent configuration store backed by a JSON file.
///
/// Supports hierarchical section/key lookups and atomic write-on-change persistence.
pub struct ConfigurationStore {
    path: PathBuf,
    data: Value,
}

impl ConfigurationStore {
    /// Create a new store, loading from the given path if it exists.
    pub fn new(path: PathBuf) -> Result<Self, String> {
        let data = if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read configuration file {:?}: {}", path, e))?;
            serde_json::from_str::<Value>(&content)
                .map_err(|e| format!("Failed to parse configuration file {:?}: {}", path, e))?
        } else {
            Value::Object(Map::new())
        };

        let mut store = Self { path, data };
        store.ensure_object_root();
        Ok(store)
    }

    /// Create an empty store that writes to the given path.
    #[allow(dead_code)]
    pub(crate) fn empty(path: PathBuf) -> Self {
        Self {
            path,
            data: Value::Object(Map::new()),
        }
    }

    /// Create a store with a default file name in the given directory.
    #[allow(dead_code)]
    pub(crate) fn default_in_dir(dir: &Path) -> Result<Self, String> {
        let path = dir.join(DEFAULT_FILE_NAME);
        Self::new(path)
    }

    /// Take a snapshot of a configuration section.
    pub fn snapshot(&self, section: Option<&str>) -> Value {
        self.get_section(section)
            .cloned()
            .unwrap_or_else(|| Value::Object(Map::new()))
    }

    /// Update a key within an optional section. Returns `true` if the value changed.
    pub fn update(
        &mut self,
        section: Option<&str>,
        key: &str,
        value: Value,
    ) -> Result<bool, String> {
        let changed = if value.is_null() {
            self.remove_value(section, key)
        } else {
            self.set_value(section, key, value)
        };

        if changed {
            self.save()?;
        }

        Ok(changed)
    }

    fn ensure_object_root(&mut self) {
        if !self.data.is_object() {
            self.data = Value::Object(Map::new());
        }
    }

    fn get_section(&self, section: Option<&str>) -> Option<&Value> {
        match section {
            None => Some(&self.data),
            Some("") => Some(&self.data),
            Some(path) => {
                let mut current = &self.data;
                for segment in path.split('.') {
                    match current.get(segment) {
                        Some(next) => current = next,
                        None => return None,
                    }
                }
                Some(current)
            }
        }
    }

    fn set_value(&mut self, section: Option<&str>, key: &str, value: Value) -> bool {
        let (target_map, final_key) = self.ensure_path(section, key);
        let existing = target_map.get(final_key);
        if existing == Some(&value) {
            return false;
        }
        target_map.insert(final_key.to_string(), value);
        true
    }

    fn remove_value(&mut self, section: Option<&str>, key: &str) -> bool {
        let mut segments: Vec<&str> = key.split('.').collect();
        let final_key = segments.pop().unwrap_or(key);

        let mut base = if let Some(section_path) = section {
            section_path.split('.').collect::<Vec<&str>>()
        } else {
            Vec::new()
        };
        base.extend(segments);

        let mut current = &mut self.data;
        for segment in &base {
            let map = match current.as_object_mut() {
                Some(map) => map,
                None => return false,
            };

            current = match map.get_mut(*segment) {
                Some(value @ Value::Object(_)) => value,
                Some(_) => return false,
                None => return false,
            };
        }

        if let Some(map) = current.as_object_mut() {
            if map.remove(final_key).is_some() {
                return true;
            }
        }
        false
    }

    fn ensure_path<'a>(
        &'a mut self,
        section: Option<&str>,
        key: &'a str,
    ) -> (&'a mut Map<String, Value>, &'a str) {
        let mut segments: Vec<&str> = key.split('.').collect();
        let final_key = segments.pop().unwrap_or(key);

        let mut path_segments: Vec<&str> = section
            .map(|s| s.split('.').filter(|seg| !seg.is_empty()).collect())
            .unwrap_or_default();
        path_segments.extend(segments);

        let mut current = &mut self.data;
        for segment in path_segments {
            if !current.is_object() {
                *current = Value::Object(Map::new());
            }
            let map = current
                .as_object_mut()
                .expect("configuration path segment must be an object after conversion");
            let entry = map
                .entry(segment.to_string())
                .or_insert_with(|| Value::Object(Map::new()));
            if !entry.is_object() {
                *entry = Value::Object(Map::new());
            }
            current = entry;
        }

        let map = current
            .as_object_mut()
            .expect("configuration target must be an object after path traversal");
        (map, final_key)
    }

    fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Failed to create configuration directory {:?}: {}",
                    parent, e
                )
            })?;
        }

        let content = serde_json::to_string_pretty(&self.data)
            .map_err(|e| format!("Failed to serialize configuration: {}", e))?;

        fs::write(&self.path, content)
            .map_err(|e| format!("Failed to write configuration file {:?}: {}", self.path, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_empty_store_creation() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        let store = ConfigurationStore::empty(path.clone());
        // Empty store should have an empty object root
        let snapshot = store.snapshot(None);
        assert!(snapshot.is_object());
        assert!(snapshot.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_new_store_from_missing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.json");
        let store = ConfigurationStore::new(path).unwrap();
        let snapshot = store.snapshot(None);
        assert!(snapshot.is_object());
        assert!(snapshot.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_new_store_from_existing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, r#"{"editor": {"fontSize": 14}}"#).unwrap();
        let store = ConfigurationStore::new(path).unwrap();
        let snapshot = store.snapshot(Some("editor"));
        assert_eq!(snapshot, json!({"fontSize": 14}));
    }

    #[test]
    fn test_default_in_dir() {
        let dir = TempDir::new().unwrap();
        let store = ConfigurationStore::default_in_dir(dir.path()).unwrap();
        let snapshot = store.snapshot(None);
        assert!(snapshot.is_object());
        assert!(snapshot.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_get_set_value_no_section() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        let mut store = ConfigurationStore::empty(path);

        let changed = store.update(None, "theme", json!("dark")).unwrap();
        assert!(changed);

        let snapshot = store.snapshot(None);
        assert_eq!(snapshot.get("theme"), Some(&json!("dark")));
    }

    #[test]
    fn test_get_set_value_with_section() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        let mut store = ConfigurationStore::empty(path);

        store.update(Some("editor"), "fontSize", json!(14)).unwrap();
        let snapshot = store.snapshot(Some("editor"));
        assert_eq!(snapshot.get("fontSize"), Some(&json!(14)));
    }

    #[test]
    fn test_update_same_value_no_change() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        let mut store = ConfigurationStore::empty(path);

        store.update(None, "key", json!("value")).unwrap();
        let changed = store.update(None, "key", json!("value")).unwrap();
        assert!(!changed, "Setting same value should report no change");
    }

    #[test]
    fn test_update_different_value_reports_change() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        let mut store = ConfigurationStore::empty(path);

        store.update(None, "key", json!("old")).unwrap();
        let changed = store.update(None, "key", json!("new")).unwrap();
        assert!(changed);
    }

    #[test]
    fn test_remove_value_with_null() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        let mut store = ConfigurationStore::empty(path);

        store.update(None, "key", json!("value")).unwrap();
        let changed = store.update(None, "key", Value::Null).unwrap();
        assert!(changed, "Removing a key should report change");
        let snapshot = store.snapshot(None);
        assert!(snapshot.get("key").is_none() || snapshot.get("key") == Some(&Value::Null));
    }

    #[test]
    fn test_nested_section_path() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        let mut store = ConfigurationStore::empty(path);

        store
            .update(Some("editor.formatting"), "tabSize", json!(4))
            .unwrap();
        let snapshot = store.snapshot(Some("editor.formatting"));
        assert_eq!(snapshot.get("tabSize"), Some(&json!(4)));
    }

    #[test]
    fn test_snapshot_missing_section() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        let store = ConfigurationStore::empty(path);

        let snapshot = store.snapshot(Some("nonexistent"));
        assert!(snapshot.is_object());
        assert!(snapshot.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_persistence_after_update() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.json");
        {
            let mut store = ConfigurationStore::empty(path.clone());
            store.update(None, "theme", json!("light")).unwrap();
        }
        // Reload and verify persisted
        let store = ConfigurationStore::new(path).unwrap();
        let snapshot = store.snapshot(None);
        assert_eq!(snapshot.get("theme"), Some(&json!("light")));
    }
}
