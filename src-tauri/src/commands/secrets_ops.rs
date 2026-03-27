/**
 * Secrets Storage Operations
 *
 * Provides secure storage for extension secrets using OS keychain.
 * Uses the keyring crate which provides:
 * - macOS: Keychain
 * - Linux: Secret Service (libsecret) or kernel keyring
 * - Windows: Credential Manager
 */
use keyring::Entry;
use serde::{Deserialize, Serialize};

const SERVICE_NAME: &str = "com.dscode.secrets";

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretResult {
    pub value: Option<String>,
}

/// Get a secret value from secure storage
#[tauri::command]
pub async fn secret_get(extension_id: String, key: String) -> Result<SecretResult, String> {
    let username = format!("{}:{}", extension_id, key);

    let entry = Entry::new(SERVICE_NAME, &username)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.get_password() {
        Ok(password) => Ok(SecretResult { value: Some(password) }),
        Err(keyring::Error::NoEntry) => Ok(SecretResult { value: None }),
        Err(e) => Err(format!("Failed to get secret: {}", e)),
    }
}

/// Store a secret value in secure storage
#[tauri::command]
pub async fn secret_store(extension_id: String, key: String, value: String) -> Result<(), String> {
    let username = format!("{}:{}", extension_id, key);

    let entry = Entry::new(SERVICE_NAME, &username)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    entry.set_password(&value).map_err(|e| format!("Failed to store secret: {}", e))
}

/// Delete a secret from secure storage
#[tauri::command]
pub async fn secret_delete(extension_id: String, key: String) -> Result<(), String> {
    let username = format!("{}:{}", extension_id, key);

    let entry = Entry::new(SERVICE_NAME, &username)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.delete_password() {
        Ok(_) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted, that's fine
        Err(e) => Err(format!("Failed to delete secret: {}", e)),
    }
}

/// Check if a secret exists in secure storage
#[tauri::command]
pub async fn secret_has(extension_id: String, key: String) -> Result<bool, String> {
    let username = format!("{}:{}", extension_id, key);

    let entry = Entry::new(SERVICE_NAME, &username)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    match entry.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(format!("Failed to check secret: {}", e)),
    }
}
