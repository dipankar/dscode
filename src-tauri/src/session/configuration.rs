use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_FILE_NAME: &str = "settings.json";

pub struct ConfigurationStore {
    path: PathBuf,
    data: Value,
}

impl ConfigurationStore {
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

    pub fn empty(path: PathBuf) -> Self {
        Self {
            path,
            data: Value::Object(Map::new()),
        }
    }

    pub fn default_in_dir(dir: &Path) -> Result<Self, String> {
        let path = dir.join(DEFAULT_FILE_NAME);
        Self::new(path)
    }

    pub fn snapshot(&self, section: Option<&str>) -> Value {
        self.get_section(section)
            .cloned()
            .unwrap_or_else(|| Value::Object(Map::new()))
    }

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

    fn ensure_path<'a>(&'a mut self, section: Option<&str>, key: &'a str) -> (&'a mut Map<String, Value>, &'a str) {
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
            let map = current.as_object_mut().unwrap();
            let entry = map
                .entry(segment.to_string())
                .or_insert_with(|| Value::Object(Map::new()));
            if !entry.is_object() {
                *entry = Value::Object(Map::new());
            }
            current = entry;
        }

        let map = current.as_object_mut().unwrap();
        (map, final_key)
    }

    fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create configuration directory {:?}: {}", parent, e))?;
        }

        let content = serde_json::to_string_pretty(&self.data)
            .map_err(|e| format!("Failed to serialize configuration: {}", e))?;

        fs::write(&self.path, content)
            .map_err(|e| format!("Failed to write configuration file {:?}: {}", self.path, e))
    }
}
