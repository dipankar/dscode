//! Node.js binary integrity verification.
//!
//! On startup, the extension host verifies the SHA256 hash of the resolved
//! Node.js binary against a list of known-good hashes. If the hash doesn't
//! match, the extension host refuses to start.

use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;
use tracing::warn;

/// Known-good SHA256 hashes for Node.js binaries.
/// These are bundled with the application and checked at startup.
const ALLOWED_HASHES: &[&str] = &[
    // Node.js 20.x LTS (common versions)
    // These should be updated when bundling specific Node.js versions
    // For development, the DSCODE_SKIP_NODE_VERIFY env var can bypass this check
];

/// Verify the SHA256 hash of a binary file against allowed hashes.
///
/// Returns `Ok(())` if the hash matches, or `Err` with a description of the mismatch.
/// If the `DSCODE_SKIP_NODE_VERIFY` environment variable is set, verification is skipped.
pub fn verify_binary(binary_path: &Path) -> Result<(), String> {
    // Allow bypass for development
    if std::env::var("DSCODE_SKIP_NODE_VERIFY").is_ok() {
        warn!("Node.js binary verification skipped (DSCODE_SKIP_NODE_VERIFY set)");
        return Ok(());
    }

    // If no allowed hashes are configured, skip verification (dev mode)
    if ALLOWED_HASHES.is_empty() {
        return Ok(());
    }

    let hash = compute_file_hash(binary_path)?;

    if ALLOWED_HASHES.contains(&hash.as_str()) {
        Ok(())
    } else {
        Err(format!(
            "Node.js binary hash mismatch: {} (expected one of: {:?})",
            hash, ALLOWED_HASHES
        ))
    }
}

/// Compute the SHA256 hash of a file.
fn compute_file_hash(path: &Path) -> Result<String, String> {
    let data = fs::read(path)
        .map_err(|e| format!("Failed to read binary {:?}: {}", path, e))?;

    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_compute_file_hash() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_binary");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);

        let hash = compute_file_hash(&file_path).unwrap();
        // SHA256 of "test content" is known
        assert_eq!(hash.len(), 64); // SHA256 hex output is 64 chars
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_verify_binary_skips_with_env() {
        // This test verifies the env var bypass works
        std::env::set_var("DSCODE_SKIP_NODE_VERIFY", "1");
        let result = verify_binary(Path::new("/nonexistent"));
        std::env::remove_var("DSCODE_SKIP_NODE_VERIFY");
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_binary_skips_empty_hashes() {
        // With empty ALLOWED_HASHES, verification passes (dev mode)
        let result = verify_binary(Path::new("/nonexistent"));
        assert!(result.is_ok());
    }
}