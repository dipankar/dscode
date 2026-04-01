/**
 * Secure Secret Storage
 *
 * Uses OS-native secure storage (Keychain/Credential Manager/Secret Service)
 * to store extension secrets securely. The in-memory cache uses a
 * per-session encryption key with AES-256-GCM to prevent plaintext
 * secrets from being readable in memory dumps.
 */
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use keyring::Entry;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use tracing::warn;
use zeroize::Zeroize;

const SERVICE_NAME: &str = "com.dscode.secrets";

/// How long cached entries remain valid (5 minutes).
const CACHE_TTL_SECS: u64 = 300;

/// An encrypted entry in the secret cache.
struct CachedSecret {
    /// AES-256-GCM ciphertext (nonce prepended).
    ciphertext: Vec<u8>,
    /// Time the entry was cached (seconds since epoch).
    cached_at: u64,
}

pub struct SecretStorage {
    /// In-memory cache for performance.
    /// Values are encrypted with a per-session AES-256-GCM key to prevent
    /// plaintext exposure in memory dumps.
    cache: Mutex<HashMap<String, CachedSecret>>,
    /// Per-session AES-256-GCM encryption key. Zeroized on drop.
    enc_key: Mutex<Vec<u8>>,
    /// Monotonic nonce counter for AES-GCM (never reused with same key).
    nonce_counter: AtomicU64,
}

impl SecretStorage {
    pub fn new() -> Self {
        // Generate a random per-session AES-256 key
        let key = Aes256Gcm::generate_key(OsRng);
        Self {
            cache: Mutex::new(HashMap::new()),
            enc_key: Mutex::new(key.to_vec()),
            nonce_counter: AtomicU64::new(0),
        }
    }

    /// Encrypt a plaintext string with the per-session key.
    /// Returns nonce || ciphertext (nonce is 12 bytes for AES-256-GCM).
    fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, String> {
        let cipher = self.cipher()?;
        // Use counter-based nonce: 4-byte prefix + 8-byte counter to get 12 bytes
        let counter = self.nonce_counter.fetch_add(1, Ordering::Relaxed);
        let nonce_bytes = Self::derive_nonce(counter);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        // Prepend nonce so we can decrypt later
        let mut out = Vec::with_capacity(12 + ciphertext.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    /// Decrypt a nonce||ciphertext blob with the per-session key.
    fn decrypt(&self, blob: &[u8]) -> Result<String, String> {
        if blob.len() < 13 {
            return Err("Ciphertext too short".into());
        }
        let cipher = self.cipher()?;
        let (nonce_bytes, ciphertext) = blob.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|_| String::from("Decryption failed (tampered or wrong key)"))?;
        String::from_utf8(plaintext)
            .map_err(|e| format!("Decrypted value is not valid UTF-8: {}", e))
    }

    /// Build the AES-256-GCM cipher from the current session key.
    fn cipher(&self) -> Result<Aes256Gcm, String> {
        let key_guard = self.enc_key.lock().unwrap_or_else(|e| {
            warn!("Encryption key lock poisoned, recovering");
            e.into_inner()
        });
        if key_guard.len() != 32 {
            return Err("Invalid encryption key length".into());
        }
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_guard);
        Ok(Aes256Gcm::new(key))
    }

    /// Derive a 12-byte nonce from a counter value.
    fn derive_nonce(counter: u64) -> [u8; 12] {
        let mut nonce = [0u8; 12];
        nonce[4..12].copy_from_slice(&counter.to_be_bytes());
        nonce
    }

    /// Get current time in seconds since epoch.
    fn now_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Evict expired entries from the cache.
    fn evict_expired(cache: &mut HashMap<String, CachedSecret>) {
        let now = Self::now_secs();
        cache.retain(|_, entry| now.saturating_sub(entry.cached_at) < CACHE_TTL_SECS);
    }

    /// Get a secret for an extension
    pub fn get(&self, extension_id: &str, key: &str) -> Result<Option<String>, String> {
        let full_key = Self::make_key(extension_id, key);

        // Check cache first
        {
            let mut cache = self.cache.lock().unwrap_or_else(|e| {
                warn!("Secrets cache lock poisoned, recovering");
                e.into_inner()
            });
            Self::evict_expired(&mut cache);
            if let Some(entry) = cache.get(&full_key) {
                let plaintext = self.decrypt(&entry.ciphertext)?;
                return Ok(Some(plaintext));
            }
        }

        // Try to get from OS keychain
        match Entry::new(SERVICE_NAME, &full_key) {
            Ok(entry) => {
                match entry.get_password() {
                    Ok(password) => {
                        // Cache it (encrypted)
                        let ciphertext = self.encrypt(&password)?;
                        let mut cache = self.cache.lock().unwrap_or_else(|e| {
                            warn!("Secrets cache lock poisoned, recovering");
                            e.into_inner()
                        });
                        cache.insert(full_key, CachedSecret {
                            ciphertext,
                            cached_at: Self::now_secs(),
                        });
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

                // Update cache (encrypted)
                let ciphertext = self.encrypt(value)?;
                let mut cache = self.cache.lock().unwrap_or_else(|e| {
                    warn!("Secrets cache lock poisoned, recovering");
                    e.into_inner()
                });
                cache.insert(full_key, CachedSecret {
                    ciphertext,
                    cached_at: Self::now_secs(),
                });

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
        // Zeroize the encryption key first
        if let Ok(mut key) = self.enc_key.lock() {
            key.zeroize();
        }
        // Best-effort clear of the cache on drop to reduce
        // the window where secrets remain in memory.
        if let Ok(mut cache) = self.cache.lock() {
            // Zeroize each ciphertext before clearing
            for entry in cache.values_mut() {
                entry.ciphertext.zeroize();
            }
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

    #[test]
    fn test_encryption_roundtrip() {
        let storage = SecretStorage::new();
        let plaintext = "my_secret_password";
        let encrypted = storage.encrypt(plaintext).unwrap();
        // Ciphertext should differ from plaintext
        assert_ne!(&encrypted[12..], plaintext.as_bytes());
        // Decryption should recover the original
        let decrypted = storage.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_cache_stores_encrypted() {
        let storage = SecretStorage::new();
        storage.set("ext2", "k2", "plaintext_value").unwrap();

        // Verify cache contains encrypted data, not plaintext
        let cache = storage.cache.lock().unwrap();
        let entry = cache.get("ext2:k2").unwrap();
        // The cached ciphertext should not contain the plaintext
        assert!(!String::from_utf8_lossy(&entry.ciphertext).contains("plaintext_value"));
    }

    #[test]
    fn test_nonce_uniqueness() {
        let storage = SecretStorage::new();
        let ct1 = storage.encrypt("aaa").unwrap();
        let ct2 = storage.encrypt("aaa").unwrap();
        // Nonces (first 12 bytes) must differ
        assert_ne!(&ct1[..12], &ct2[..12]);
    }

    #[test]
    fn test_ttl_eviction() {
        let storage = SecretStorage::new();
        storage.set("ext3", "k3", "val").unwrap();

        // Manually expire the entry by setting cached_at to epoch
        {
            let mut cache = storage.cache.lock().unwrap();
            if let Some(entry) = cache.get_mut("ext3:k3") {
                entry.cached_at = 0; // Set to epoch so TTL check evicts it
            }
        }

        // Evict expired entries
        {
            let mut cache = storage.cache.lock().unwrap();
            SecretStorage::evict_expired(&mut cache);
        }

        // After eviction, cache entry should be gone
        {
            let cache = storage.cache.lock().unwrap();
            assert!(cache.get("ext3:k3").is_none());
        }

        // Clean up
        storage.delete("ext3", "k3").ok();
    }
}