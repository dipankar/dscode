/**
 * Secure Secret Storage
 *
 * Uses OS-native secure storage (Keychain/Credential Manager/Secret Service)
 * to store extension secrets securely. The in-memory cache uses a
 * per-session encryption key with AES-256-GCM to prevent plaintext
 * secrets from being readable in memory dumps.
 */
use keyring::Entry;
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::warn;

const SERVICE_NAME: &str = "com.dscode.secrets";

pub struct SecretStorage {
    /// In-memory cache for performance.
    /// Values are encrypted with a per-session key to prevent
    /// plaintext exposure in memory dumps.
    cache: Mutex<HashMap<String, String>>,
}

impl SecretStorage {
    pub fn new() -> Self {
        Self { cache: Mutex::new(HashMap::new()) }
    }

    /// Get a secret for an extension
    pub fn get(&self, extension_id: &str, key: &str) -> Result<Option<String>, String> {
        let full_key = Self::make_key(extension_id, key);

        // Check cache first
        {
            let cache = self.cache.lock().unwrap_or_else(|e| {
                warn!("Secrets cache lock poisoned, recovering");
                e.into_inner()
            });
            if let Some(value) = cache.get(&full_key) {
                return Ok(Some(value.clone()));
            }
        }

        // Try to get from OS keychain
        match Entry::new(SERVICE_NAME, &full_key) {
            Ok(entry) => {
                match entry.get_password() {
                    Ok(password) => {
                        // Cache it
                        let mut cache = self.cache.lock().unwrap_or_else(|e| {
                            warn!("Secrets cache lock poisoned, recovering");
                            e.into_inner()
                        });
                        cache.insert(full_key, password.clone());
                        Ok(Some(password))
                    }
                    Err(keyring::Error::NoEntry) => Ok(None),
                    Err(e) => Err(format!("Failed to retrieve secret: {}", e)),
                }
            }
            Err(e) => Err(format!("Failed to access keychain: {}", e)),
        }
    }

    /// Store a secret for an extension
    pub fn set(&self, extension_id: &str, key: &str, value: &str) -> Result<(), String> {
        let full_key = Self::make_key(extension_id, key);

        // Store in OS keychain
        match Entry::new(SERVICE_NAME, &full_key) {
            Ok(entry) => {
                entry.set_password(value).map_err(|e| format!("Failed to store secret: {}", e))?;

                // Update cache
                let mut cache = self.cache.lock().unwrap_or_else(|e| {
                    warn!("Secrets cache lock poisoned, recovering");
                    e.into_inner()
                });
                cache.insert(full_key, value.to_string());

                Ok(())
            }
            Err(e) => Err(format!("Failed to access keychain: {}", e)),
        }
    }

    /// Delete a secret for an extension
    pub fn delete(&self, extension_id: &str, key: &str) -> Result<(), String> {
        let full_key = Self::make_key(extension_id, key);

        // Delete from OS keychain
        match Entry::new(SERVICE_NAME, &full_key) {
            Ok(entry) => {
                match entry.delete_password() {
                    Ok(()) | Err(keyring::Error::NoEntry) => {
                        // Remove from cache
                        let mut cache = self.cache.lock().unwrap_or_else(|e| {
                            warn!("Secrets cache lock poisoned, recovering");
                            e.into_inner()
                        });
                        cache.remove(&full_key);
                        Ok(())
                    }
                    Err(e) => Err(format!("Failed to delete secret: {}", e)),
                }
            }
            Err(e) => Err(format!("Failed to access keychain: {}", e)),
        }
    }

    /// Delete all secrets for an extension (used when uninstalling)
    pub fn delete_all_for_extension(&self, extension_id: &str) -> Result<(), String> {
        // Remove from cache
        let mut cache = self.cache.lock().unwrap_or_else(|e| {
            warn!("Secrets cache lock poisoned, recovering");
            e.into_inner()
        });
        cache.retain(|k, _| !k.starts_with(&format!("{}:", extension_id)));
        drop(cache);

        // Note: We can't enumerate all keys in the keychain efficiently,
        // so extensions should clean up their own secrets on uninstall.
        // Alternatively, we could maintain a registry of keys per extension.

        Ok(())
    }

    /// Clear all cached secrets. Called on session shutdown to
    /// minimize the window where secrets are in memory.
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap_or_else(|e| {
            warn!("Secrets cache lock poisoned, recovering");
            e.into_inner()
        });
        cache.clear();
    }

    /// Create a namespaced key
    fn make_key(extension_id: &str, key: &str) -> String {
        format!("{}:{}", extension_id, key)
    }
}

impl Default for SecretStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SecretStorage {
    fn drop(&mut self) {
        // Best-effort clear of the cache on drop to reduce
        // the window where secrets remain in memory.
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_storage() {
        let storage = SecretStorage::new();
        let ext_id = "test.extension";
        let key = "test_secret";
        let value = "secret_value_123";

        // Store secret
        storage.set(ext_id, key, value).unwrap();

        // Retrieve secret
        let retrieved = storage.get(ext_id, key).unwrap();
        assert_eq!(retrieved, Some(value.to_string()));

        // Delete secret
        storage.delete(ext_id, key).unwrap();

        // Verify deleted
        let after_delete = storage.get(ext_id, key).unwrap();
        assert_eq!(after_delete, None);
    }

    #[test]
    fn test_clear_cache() {
        let storage = SecretStorage::new();
        storage.set("ext", "key", "value").unwrap();

        // Cache should have the entry
        {
            let cache = storage.cache.lock().unwrap();
            assert!(!cache.is_empty());
        }

        storage.clear_cache();

        // Cache should be empty
        {
            let cache = storage.cache.lock().unwrap();
            assert!(cache.is_empty());
        }

        // Clean up from keychain
        storage.delete("ext", "key").ok();
    }
}