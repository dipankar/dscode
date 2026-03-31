//! Integration tests for dscode-extension-host crate.
//!
//! Tests multiple extension-host components working together:
//! ExtensionHostManager lifecycle, IpcManager creation, PathValidator,
//! RateLimiter, SecretStorage, and Permission parsing.

use dscode_extension_host::{
    ExtensionHostManager, ExtensionHostState, ExtensionPermissions, IpcManager, PathValidator,
    Permission, RateLimiter, SecretStorage,
};
use std::path::PathBuf;

// ── ExtensionHostManager lifecycle tests ─────────────────────────────────────

#[test]
fn test_extension_host_manager_new() {
    let manager = ExtensionHostManager::new("test.extension".to_string());
    assert_eq!(manager.state(), ExtensionHostState::Stopped);
}

#[test]
fn test_extension_host_manager_initial_state() {
    let manager = ExtensionHostManager::new("my-ext".to_string());
    // New manager should be in Stopped state
    assert_eq!(manager.state(), ExtensionHostState::Stopped);
}

#[test]
fn test_extension_host_manager_state_variants_distinct() {
    let states = [
        ExtensionHostState::Stopped,
        ExtensionHostState::Starting,
        ExtensionHostState::Running,
        ExtensionHostState::Unhealthy,
        ExtensionHostState::Restarting,
        ExtensionHostState::Stopping,
        ExtensionHostState::Crashed,
    ];

    for (i, a) in states.iter().enumerate() {
        for (j, b) in states.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn test_extension_host_state_copy_equality() {
    let state1 = ExtensionHostState::Stopped;
    let state2 = state1;
    assert_eq!(state1, state2);

    let state3 = ExtensionHostState::Running;
    assert_ne!(state1, state3);
}

// ── IpcManager creation and default state ────────────────────────────────────

#[test]
fn test_ipc_manager_new() {
    let _ipc = IpcManager::new();
    // IpcManager should be creatable without panicking
}

#[test]
fn test_ipc_manager_default() {
    let _ipc = IpcManager::default();
    // Default trait should work
}

#[tokio::test]
async fn test_ipc_manager_initial_state() {
    let ipc = IpcManager::new();

    // No connections should exist initially
    assert!(!ipc.is_connected("ext1").await);
    assert!(!ipc.is_connected("ext2").await);

    // Getting a nonexistent connection should return None
    assert!(ipc.get("ext1").await.is_none());
    assert!(ipc.get_outgoing("ext1").await.is_none());
}

#[tokio::test]
async fn test_ipc_manager_disconnect_nonexistent() {
    let ipc = IpcManager::new();

    // Disconnecting a nonexistent ID should not panic
    ipc.disconnect("nonexistent").await;

    // Still no connections
    assert!(!ipc.is_connected("nonexistent").await);
}

// ── PathValidator with multiple workspace folders ───────────────────────────

#[test]
fn test_path_validator_new() {
    let validator = PathValidator::new();
    // Default validator has no allowed roots
    let result = validator.validate_file_path("/etc/passwd");
    assert!(result.is_err());
}

#[test]
fn test_path_validator_default() {
    let validator = PathValidator::default();
    let result = validator.validate_file_path("/etc/passwd");
    assert!(result.is_err());
}

#[test]
fn test_path_validator_multiple_workspace_folders() {
    let mut validator = PathValidator::new();

    let dir1 = tempfile::tempdir().unwrap();
    let dir2 = tempfile::tempdir().unwrap();
    let dir3 = tempfile::tempdir().unwrap();

    validator.add_workspace_folder(dir1.path().to_path_buf());
    validator.add_workspace_folder(dir2.path().to_path_buf());
    validator.add_workspace_folder(dir3.path().to_path_buf());

    // Files within any of the workspace folders should be allowed
    let file1 = dir1.path().join("src").join("main.rs");
    std::fs::create_dir_all(file1.parent().unwrap()).unwrap();
    std::fs::write(&file1, "fn main() {}").unwrap();
    let result = validator.validate_file_path(&file1.to_string_lossy());
    assert!(result.is_ok());

    let file2 = dir2.path().join("config.json");
    std::fs::write(&file2, "{}").unwrap();
    let result = validator.validate_file_path(&file2.to_string_lossy());
    assert!(result.is_ok());

    let file3 = dir3.path().join("README.md");
    std::fs::write(&file3, "# Test").unwrap();
    let result = validator.validate_file_path(&file3.to_string_lossy());
    assert!(result.is_ok());
}

#[test]
fn test_path_validator_set_special_dirs() {
    let mut validator = PathValidator::new();

    let ws_dir = tempfile::tempdir().unwrap();
    let ext_dir = tempfile::tempdir().unwrap();
    let storage_dir = tempfile::tempdir().unwrap();
    let logs_dir = tempfile::tempdir().unwrap();
    let temp_dir = tempfile::tempdir().unwrap();

    validator.add_workspace_folder(ws_dir.path().to_path_buf());
    validator.set_extensions_dir(ext_dir.path().to_path_buf());
    validator.set_storage_dir(storage_dir.path().to_path_buf());
    validator.set_logs_dir(logs_dir.path().to_path_buf());
    validator.set_temp_dir(temp_dir.path().to_path_buf());

    // Files in the extensions dir should be allowed
    let ext_file = ext_dir.path().join("extension.js");
    std::fs::write(&ext_file, "// ext").unwrap();
    assert!(validator.validate_file_path(&ext_file.to_string_lossy()).is_ok());

    // Files in the storage dir should be allowed
    let storage_file = storage_dir.path().join("state.json");
    std::fs::write(&storage_file, "{}").unwrap();
    assert!(validator.validate_file_path(&storage_file.to_string_lossy()).is_ok());

    // Files in the logs dir should be allowed
    let log_file = logs_dir.path().join("app.log");
    std::fs::write(&log_file, "log entry").unwrap();
    assert!(validator.validate_file_path(&log_file.to_string_lossy()).is_ok());

    // Files in the temp dir should be allowed
    let tmp_file = temp_dir.path().join("scratch.txt");
    std::fs::write(&tmp_file, "temp data").unwrap();
    assert!(validator.validate_file_path(&tmp_file.to_string_lossy()).is_ok());
}

#[test]
fn test_path_validator_validate_path_uri_formats() {
    let mut validator = PathValidator::new();
    let dir = tempfile::tempdir().unwrap();
    validator.add_workspace_folder(dir.path().to_path_buf());

    let test_file = dir.path().join("test.txt");
    std::fs::write(&test_file, "hello").unwrap();

    // file:/// URI format (3 slashes for absolute paths on Unix)
    let uri = format!("file:///{}", test_file.display());
    let result = validator.validate_path(&uri);
    assert!(result.is_ok());

    // file://localhost/ URI format (with explicit localhost)
    let uri2 = format!("file://localhost{}", test_file.display());
    let result2 = validator.validate_path(&uri2);
    assert!(result2.is_ok());
}

#[test]
fn test_path_validator_blocks_outside_dirs() {
    let mut validator = PathValidator::new();
    let ws_dir = tempfile::tempdir().unwrap();
    validator.add_workspace_folder(ws_dir.path().to_path_buf());

    // System paths outside workspace should be blocked
    assert!(validator.validate_file_path("/etc/passwd").is_err());
    assert!(validator.validate_file_path("/usr/bin/ls").is_err());
}

#[test]
fn test_path_validator_error_sanitization() {
    let error_msg = "/home/user/secret/file.txt: No such file or directory";
    let sanitized = PathValidator::sanitize_error(&error_msg);
    // Sanitized error should not contain file paths
    assert!(!sanitized.contains("/home"));
    assert!(!sanitized.contains("/secret"));
}

#[test]
fn test_path_validator_get_relative_path() {
    let mut validator = PathValidator::new();
    let ws_dir = tempfile::tempdir().unwrap();
    validator.add_workspace_folder(ws_dir.path().to_path_buf());

    let _test_file = ws_dir.path().join("src").join("main.rs");
    let canonical_ws = std::fs::canonicalize(ws_dir.path()).unwrap();
    let canonical_file = canonical_ws.join("src").join("main.rs");

    let relative = validator.get_relative_path(&canonical_file);
    // Should be relative to workspace root
    assert_eq!(relative, PathBuf::from("src").join("main.rs"));
}

// ── RateLimiter with concurrent checks ──────────────────────────────────────

#[test]
fn test_rate_limiter_new() {
    let limiter = RateLimiter::new();
    // Should allow initial requests
    assert!(limiter.check_rate_limit("ext1").is_ok());
}

#[test]
fn test_rate_limiter_default() {
    let limiter = RateLimiter::default();
    assert!(limiter.check_rate_limit("ext1").is_ok());
}

#[test]
fn test_rate_limiter_with_quota() {
    let limiter = RateLimiter::with_quota(5);
    // 5 requests should succeed
    for _ in 0..5 {
        assert!(limiter.check_rate_limit("ext1").is_ok());
    }
    // 6th should fail
    assert!(limiter.check_rate_limit("ext1").is_err());
}

#[test]
fn test_rate_limiter_per_extension_isolation() {
    let limiter = RateLimiter::with_quota(3);

    // Exhaust quota for ext1
    for _ in 0..3 {
        limiter.check_rate_limit("ext1").unwrap();
    }
    assert!(limiter.check_rate_limit("ext1").is_err());

    // ext2 should still have full quota
    assert!(limiter.check_rate_limit("ext2").is_ok());
    assert!(limiter.check_rate_limit("ext2").is_ok());
    assert!(limiter.check_rate_limit("ext2").is_ok());
    assert!(limiter.check_rate_limit("ext2").is_err());
}

#[test]
fn test_rate_limiter_concurrent_checks() {
    use std::sync::Arc;
    use std::thread;

    let limiter = Arc::new(RateLimiter::with_quota(50));
    let mut handles = vec![];

    // Spawn multiple threads that check the rate limit
    for i in 0..4 {
        let limiter_clone = Arc::clone(&limiter);
        handles.push(thread::spawn(move || {
            let ext_id = format!("ext-thread-{}", i);
            let mut allowed = 0;
            for _ in 0..60 {
                if limiter_clone.check_rate_limit(&ext_id).is_ok() {
                    allowed += 1;
                }
            }
            allowed
        }));
    }

    // Each thread should get approximately 50 allowed requests
    for (i, handle) in handles.into_iter().enumerate() {
        let allowed = handle.join().unwrap();
        assert_eq!(
            allowed, 50,
            "Thread {} should have 50 allowed requests (got {})",
            i, allowed
        );
    }
}

#[test]
fn test_rate_limiter_remove_limiter() {
    let limiter = RateLimiter::with_quota(2);

    // Exhaust quota
    limiter.check_rate_limit("ext1").unwrap();
    limiter.check_rate_limit("ext1").unwrap();
    assert!(limiter.check_rate_limit("ext1").is_err());

    // Remove the limiter for ext1
    limiter.remove_limiter("ext1");

    // After removal, a new limiter should be created with fresh quota
    assert!(limiter.check_rate_limit("ext1").is_ok());
    assert!(limiter.check_rate_limit("ext1").is_ok());
    assert!(limiter.check_rate_limit("ext1").is_err());
}

#[test]
fn test_rate_limiter_error_message() {
    let limiter = RateLimiter::with_quota(1);
    limiter.check_rate_limit("my-ext").unwrap();

    let result = limiter.check_rate_limit("my-ext");
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Rate limit exceeded"));
    assert!(err_msg.contains("my-ext"));
}

// ── SecretStorage basic operations ──────────────────────────────────────────

#[test]
fn test_secret_storage_new() {
    let storage = SecretStorage::new();
    // Getting a nonexistent key should return None
    let result = storage.get("nonexistent.ext", "api_key").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_secret_storage_default() {
    let storage = SecretStorage::default();
    let result = storage.get("nonexistent.ext", "token").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_secret_storage_set_get_delete() {
    let storage = SecretStorage::new();
    // Use unique key with UUID to avoid keychain conflicts from previous runs
    let ext_id = format!("test.integration.{}", std::process::id());
    let key = format!("test_secret_key_{}", std::process::id());
    let value = "super_secret_value_123";

    // Clean up any previous entry first
    let _ = storage.delete(&ext_id, &key);

    // Set a secret
    storage.set(&ext_id, &key, value).unwrap();

    // Get it back
    let retrieved = storage.get(&ext_id, &key).unwrap();
    assert_eq!(retrieved, Some(value.to_string()));

    // Delete it
    storage.delete(&ext_id, &key).unwrap();

    // Should be gone
    let after_delete = storage.get(&ext_id, &key).unwrap();
    assert!(after_delete.is_none() || after_delete == Some(String::new()));
}

#[test]
fn test_secret_storage_different_extensions_isolated() {
    let storage = SecretStorage::new();
    let id = std::process::id();
    let ext1 = format!("ext1.isolated.{}", id);
    let ext2 = format!("ext2.isolated.{}", id);
    let key = format!("shared_key_{}", id);

    // Clean up any previous entries
    let _ = storage.delete(&ext1, &key);
    let _ = storage.delete(&ext2, &key);

    storage.set(&ext1, &key, "value_for_ext1").unwrap();
    storage.set(&ext2, &key, "value_for_ext2").unwrap();

    // Each extension should get its own value
    assert_eq!(
        storage.get(&ext1, &key).unwrap(),
        Some("value_for_ext1".to_string())
    );
    assert_eq!(
        storage.get(&ext2, &key).unwrap(),
        Some("value_for_ext2".to_string())
    );

    // Deleting ext1's secret should not affect ext2
    storage.delete(&ext1, &key).unwrap();
    assert_eq!(
        storage.get(&ext2, &key).unwrap(),
        Some("value_for_ext2".to_string())
    );

    // Clean up
    let _ = storage.delete(&ext2, &key);
}

#[test]
fn test_secret_storage_multiple_keys() {
    let storage = SecretStorage::new();
    let id = std::process::id();
    let ext = format!("my-ext.multi.{}", id);
    let key1 = format!("api_key_{}", id);
    let key2 = format!("refresh_token_{}", id);
    let key3 = format!("user_id_{}", id);

    // Clean up any previous entries
    let _ = storage.delete(&ext, &key1);
    let _ = storage.delete(&ext, &key2);
    let _ = storage.delete(&ext, &key3);

    storage.set(&ext, &key1, "abc123").unwrap();
    storage.set(&ext, &key2, "xyz789").unwrap();
    storage.set(&ext, &key3, "42").unwrap();

    assert_eq!(storage.get(&ext, &key1).unwrap(), Some("abc123".to_string()));
    assert_eq!(storage.get(&ext, &key2).unwrap(), Some("xyz789".to_string()));
    assert_eq!(storage.get(&ext, &key3).unwrap(), Some("42".to_string()));

    // Clean up
    let _ = storage.delete(&ext, &key1);
    let _ = storage.delete(&ext, &key2);
    let _ = storage.delete(&ext, &key3);
}

#[test]
fn test_secret_storage_delete_all_for_extension() {
    let storage = SecretStorage::new();
    let id = std::process::id();
    let ext_a = format!("ext-a.delall.{}", id);
    let ext_b = format!("ext-b.delall.{}", id);
    let key1 = format!("key1_{}", id);
    let key2 = format!("key2_{}", id);
    let key_b = format!("key1b_{}", id);

    // Clean up any previous entries
    let _ = storage.delete(&ext_a, &key1);
    let _ = storage.delete(&ext_a, &key2);
    let _ = storage.delete(&ext_b, &key_b);

    storage.set(&ext_a, &key1, "val1").unwrap();
    storage.set(&ext_a, &key2, "val2").unwrap();
    storage.set(&ext_b, &key_b, "val1b").unwrap();

    // Delete all for ext-a (clears cache)
    storage.delete_all_for_extension(&ext_a).unwrap();

    // ext-b should still have data (from keychain)
    let result = storage.get(&ext_b, &key_b).unwrap();
    assert_eq!(result, Some("val1b".to_string()));

    // Clean up ext-b
    let _ = storage.delete(&ext_b, &key_b);
}

// ── Permission parsing and checking ──────────────────────────────────────────

#[test]
fn test_permission_from_manifest_valid() {
    let perms = ExtensionPermissions::from_manifest(
        "my.ext".to_string(),
        Some(vec![
            "fileSystem.read".to_string(),
            "fileSystem.write".to_string(),
            "network.http".to_string(),
        ]),
    )
    .unwrap();

    assert!(perms.has_permission(&Permission::FileSystemRead));
    assert!(perms.has_permission(&Permission::FileSystemWrite));
    assert!(perms.has_permission(&Permission::NetworkHttp));
    assert!(!perms.has_permission(&Permission::FileSystemDelete));
}

#[test]
fn test_permission_from_manifest_empty() {
    let perms = ExtensionPermissions::from_manifest("my.ext".to_string(), None).unwrap();
    assert!(!perms.has_permission(&Permission::FileSystemRead));

    let perms_empty = ExtensionPermissions::from_manifest("my.ext".to_string(), Some(vec![]))
        .unwrap();
    assert!(!perms_empty.has_permission(&Permission::FileSystemRead));
}

#[test]
fn test_permission_from_manifest_invalid() {
    let result = ExtensionPermissions::from_manifest(
        "bad.ext".to_string(),
        Some(vec!["invalid.permission".to_string()]),
    );
    assert!(result.is_err());
}

#[test]
fn test_permission_check_granted() {
    let perms = ExtensionPermissions::new(
        "my.ext".to_string(),
        vec![Permission::FileSystemRead, Permission::NetworkHttp],
    );

    assert!(perms.check_permission(&Permission::FileSystemRead).is_ok());
    assert!(perms.check_permission(&Permission::NetworkHttp).is_ok());
}

#[test]
fn test_permission_check_denied() {
    let perms = ExtensionPermissions::new("my.ext".to_string(), vec![Permission::FileSystemRead]);

    let result = perms.check_permission(&Permission::NetworkHttp);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("my.ext"));
    assert!(err.contains("NetworkHttp"));
}

#[test]
fn test_permission_all_categories() {
    // Test that all permission categories can be parsed from manifest strings
    let all_permissions = vec![
        "fileSystem.read",
        "fileSystem.write",
        "fileSystem.delete",
        "workspace.read",
        "workspace.write",
        "workspace.execute",
        "network.http",
        "network.https",
        "network.webSocket",
        "clipboard.read",
        "clipboard.write",
        "secrets.read",
        "secrets.write",
        "ui.notifications",
        "ui.dialogs",
        "ui.quickPick",
        "textEditor.read",
        "textEditor.write",
        "textEditor.selection",
        "debug.start",
        "debug.stop",
        "debug.breakpoints",
        "terminal.create",
        "terminal.send",
        "tasks.execute",
        "commands.execute",
        "authentication.getSession",
        "authentication.createSession",
    ];

    let result = ExtensionPermissions::from_manifest(
        "full.ext".to_string(),
        Some(all_permissions.iter().map(|s| s.to_string()).collect()),
    );

    assert!(result.is_ok());
    let perms = result.unwrap();

    // Verify a sample from each category
    assert!(perms.has_permission(&Permission::FileSystemRead));
    assert!(perms.has_permission(&Permission::WorkspaceExecute));
    assert!(perms.has_permission(&Permission::NetworkWebSocket));
    assert!(perms.has_permission(&Permission::ClipboardWrite));
    assert!(perms.has_permission(&Permission::SecretsWrite));
    assert!(perms.has_permission(&Permission::ShowQuickPick));
    assert!(perms.has_permission(&Permission::TextEditorSelection));
    assert!(perms.has_permission(&Permission::DebugBreakpoints));
    assert!(perms.has_permission(&Permission::TerminalSend));
    assert!(perms.has_permission(&Permission::TasksExecute));
    assert!(perms.has_permission(&Permission::CommandsExecute));
    assert!(perms.has_permission(&Permission::AuthenticationCreateSession));
}

#[test]
fn test_permission_equality_and_hash() {
    // Permissions should be usable as HashSet keys
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(Permission::FileSystemRead);
    set.insert(Permission::FileSystemRead); // Duplicate should not add
    set.insert(Permission::NetworkHttp);

    assert_eq!(set.len(), 2);
    assert!(set.contains(&Permission::FileSystemRead));
    assert!(set.contains(&Permission::NetworkHttp));
}

// ── Cross-component integration ────────────────────────────────────────────

#[test]
fn test_extension_host_state_and_permissions_integration() {
    // Verify that ExtensionHostState and ExtensionPermissions can be used together
    // to model a realistic scenario: a manager starts, permissions are checked
    let manager = ExtensionHostManager::new("perm-test".to_string());
    assert_eq!(manager.state(), ExtensionHostState::Stopped);

    let perms = ExtensionPermissions::from_manifest(
        "perm-test".to_string(),
        Some(vec!["fileSystem.read".to_string(), "network.http".to_string()]),
    )
    .unwrap();

    // An extension with file system read can be tracked
    assert!(perms.has_permission(&Permission::FileSystemRead));
    assert_eq!(manager.state(), ExtensionHostState::Stopped);
}

#[test]
fn test_path_validator_and_permissions_integration() {
    // Verify PathValidator and ExtensionPermissions work together:
    // an extension with FileSystemRead permission should be able to
    // read paths validated by PathValidator
    let mut validator = PathValidator::new();
    let ws_dir = tempfile::tempdir().unwrap();
    validator.add_workspace_folder(ws_dir.path().to_path_buf());

    let perms = ExtensionPermissions::from_manifest(
        "fs-ext".to_string(),
        Some(vec!["fileSystem.read".to_string()]),
    )
    .unwrap();

    // The extension has read permission, and the path is within allowed dirs
    assert!(perms.has_permission(&Permission::FileSystemRead));

    let test_file = ws_dir.path().join("data.txt");
    std::fs::write(&test_file, "content").unwrap();
    assert!(validator.validate_file_path(&test_file.to_string_lossy()).is_ok());

    // An extension without write permission should not be able to write
    assert!(!perms.has_permission(&Permission::FileSystemWrite));
}

#[test]
fn test_rate_limiter_with_permissions_integration() {
    // RateLimiter should limit regardless of permissions
    let limiter = RateLimiter::with_quota(2);
    let perms = ExtensionPermissions::from_manifest(
        "net-ext".to_string(),
        Some(vec!["network.http".to_string()]),
    )
    .unwrap();

    // Extension has network permission
    assert!(perms.has_permission(&Permission::NetworkHttp));

    // But rate limiter still limits requests
    assert!(limiter.check_rate_limit("net-ext").is_ok());
    assert!(limiter.check_rate_limit("net-ext").is_ok());
    assert!(limiter.check_rate_limit("net-ext").is_err()); // Rate limited
}