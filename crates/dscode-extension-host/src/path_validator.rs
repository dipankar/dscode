use std::fs;
/**
 * Path Validation and Sandboxing
 *
 * Prevents path traversal attacks and ensures extensions can only access
 * files within allowed directories (workspace, extensions folder, temp).
 */
use std::path::{Path, PathBuf};

/// Percent-decode a URI path component
fn percent_decode_str(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.bytes();

    while let Some(byte) = chars.next() {
        if byte == b'%' {
            let hi = chars.next();
            let lo = chars.next();
            if let (Some(h), Some(l)) = (hi, lo) {
                if let (Some(hv), Some(lv)) = (hex_digit(h), hex_digit(l)) {
                    result.push(char::from(hv * 16 + lv));
                    continue;
                }
            }
            result.push(byte as char);
        } else if byte == b'+' {
            result.push(' ');
        } else {
            result.push(byte as char);
        }
    }

    result
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct PathValidator {
    /// Allowed root directories (workspace folders)
    allowed_roots: Vec<PathBuf>,
    /// Extensions directory
    extensions_dir: Option<PathBuf>,
    /// Storage directory
    storage_dir: Option<PathBuf>,
    /// Logs directory
    logs_dir: Option<PathBuf>,
    /// Temp directory for this workspace
    temp_dir: Option<PathBuf>,
}

impl PathValidator {
    pub fn new() -> Self {
        Self {
            allowed_roots: Vec::new(),
            extensions_dir: None,
            storage_dir: None,
            logs_dir: None,
            temp_dir: None,
        }
    }

    pub fn add_workspace_folder(&mut self, path: PathBuf) {
        if let Ok(canonical) = fs::canonicalize(&path) {
            self.allowed_roots.push(canonical);
        }
    }

    pub fn set_extensions_dir(&mut self, path: PathBuf) {
        if let Ok(canonical) = fs::canonicalize(&path) {
            self.extensions_dir = Some(canonical);
        }
    }

    pub fn set_storage_dir(&mut self, path: PathBuf) {
        if let Ok(canonical) = fs::canonicalize(&path) {
            self.storage_dir = Some(canonical);
        }
    }

    pub fn set_logs_dir(&mut self, path: PathBuf) {
        if let Ok(canonical) = fs::canonicalize(&path) {
            self.logs_dir = Some(canonical);
        }
    }

    pub fn set_temp_dir(&mut self, path: PathBuf) {
        if let Ok(canonical) = fs::canonicalize(&path) {
            self.temp_dir = Some(canonical);
        }
    }

    /// Validate and normalize a path from a URI
    pub fn validate_path(&self, uri: &str) -> Result<PathBuf, String> {
        let path_str = if let Some(rest) = uri.strip_prefix("file:///") {
            rest
        } else if let Some(after_slashes) = uri.strip_prefix("file://") {
            if let Some(slash_pos) = after_slashes.find('/') {
                &after_slashes[slash_pos..]
            } else {
                after_slashes
            }
        } else if uri.starts_with("file:/") {
            &uri[5..]
        } else {
            uri
        };

        let decoded = percent_decode_str(path_str);
        self.validate_file_path(&decoded)
    }

    pub fn validate_file_path(&self, path_str: &str) -> Result<PathBuf, String> {
        let path = Path::new(path_str);

        let canonical = if path.exists() {
            fs::canonicalize(path)
                .map_err(|e| Self::sanitize_error(&format!("Invalid path: {}", e)))?
        } else {
            if let Some(parent) = path.parent() {
                if parent.as_os_str().is_empty() {
                    fs::canonicalize(".")
                        .map_err(|e| Self::sanitize_error(&format!("{}", e)))?
                        .join(path)
                } else if parent.exists() {
                    let canonical_parent = fs::canonicalize(parent)
                        .map_err(|e| Self::sanitize_error(&format!("{}", e)))?;
                    let joined = canonical_parent.join(path.file_name().unwrap_or_default());
                    if self.is_path_allowed(&joined) {
                        joined
                    } else {
                        path.to_path_buf()
                    }
                } else {
                    return Err("Parent directory does not exist".to_string());
                }
            } else {
                return Err("Invalid path: no parent directory".to_string());
            }
        };

        if self.is_path_allowed(&canonical) {
            Ok(canonical)
        } else {
            Err("Access denied: Path outside allowed directories".to_string())
        }
    }

    /// Check if a canonical path is within allowed directories
    fn is_path_allowed(&self, canonical_path: &Path) -> bool {
        for root in &self.allowed_roots {
            if canonical_path.starts_with(root) {
                return true;
            }
        }

        if let Some(ext_dir) = &self.extensions_dir {
            if canonical_path.starts_with(ext_dir) {
                return true;
            }
        }

        if let Some(storage) = &self.storage_dir {
            if canonical_path.starts_with(storage) {
                return true;
            }
        }

        if let Some(logs) = &self.logs_dir {
            if canonical_path.starts_with(logs) {
                return true;
            }
        }

        if let Some(temp) = &self.temp_dir {
            if canonical_path.starts_with(temp) {
                return true;
            }
        }

        false
    }

    /// Sanitize error messages to not leak full file system paths
    pub fn sanitize_error(error: &dyn std::fmt::Display) -> String {
        let error_str = error.to_string();
        if error_str.contains("/") || error_str.contains("\\") {
            "Operation failed: Invalid or inaccessible path".to_string()
        } else {
            error_str
        }
    }

    /// Get relative path for display (doesn't leak full FS structure)
    pub fn get_relative_path(&self, canonical_path: &Path) -> PathBuf {
        // Try to make path relative to workspace roots
        for root in &self.allowed_roots {
            if let Ok(rel) = canonical_path.strip_prefix(root) {
                return rel.to_path_buf();
            }
        }

        // Fallback to filename only
        canonical_path
            .file_name()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("unknown"))
    }
}

impl Default for PathValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_path_validation() {
        let mut validator = PathValidator::new();
        let current_dir = env::current_dir().unwrap();
        validator.add_workspace_folder(current_dir.clone());

        // This file should be accessible (Cargo.toml always exists in workspace root)
        let this_file = current_dir.join("Cargo.toml");
        // Use file:/// URI format (3 slashes for absolute paths)
        let uri = if this_file.is_absolute() {
            format!("file:///{}", this_file.display())
        } else {
            format!("file://{}", this_file.display())
        };

        if this_file.exists() {
            assert!(validator.validate_path(&uri).is_ok());
        }

        // Parent directory traversal should fail
        let parent_attack = "file:///../../../etc/passwd";
        assert!(validator.validate_path(parent_attack).is_err());
    }

    #[test]
    fn test_error_sanitization() {
        let error = PathValidator::sanitize_error(&"/home/user/secret/file.txt: No such file");
        assert!(!error.contains("/home"));
        assert!(!error.contains("/secret"));
    }

    #[test]
    fn test_path_validator_allows_workspace_paths() {
        let mut validator = PathValidator::new();
        let current_dir = env::current_dir().unwrap();
        validator.add_workspace_folder(current_dir.clone());

        // A path within the workspace should be allowed
        let test_path = current_dir.join("src").join("lib.rs");
        let test_path_str = test_path.to_string_lossy().to_string();
        if test_path.exists() {
            let result = validator.validate_file_path(&test_path_str);
            assert!(result.is_ok());
        }

        // The workspace root itself should be allowed
        let root_str = current_dir.to_string_lossy().to_string();
        if current_dir.exists() {
            let result = validator.validate_file_path(&root_str);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_path_validator_blocks_traversal() {
        let mut validator = PathValidator::new();
        let current_dir = env::current_dir().unwrap();
        validator.add_workspace_folder(current_dir);

        // Paths with ../ should be blocked (or resolve outside workspace)
        let traversal_uris = [
            "file://../../../etc/passwd",
            "file://../../tmp/malicious",
        ];

        for uri in &traversal_uris {
            let result = validator.validate_path(uri);
            assert!(result.is_err(), "Expected path traversal to be blocked: {}", uri);
        }
    }

    #[test]
    fn test_path_validator_blocks_absolute_paths() {
        let validator = PathValidator::new();

        // A validator with no allowed roots should block everything
        let result = validator.validate_file_path("/etc/passwd");
        assert!(result.is_err(), "Absolute path should be blocked with no workspace roots");

        let result = validator.validate_file_path("/usr/bin/python");
        assert!(result.is_err(), "Absolute path should be blocked with no workspace roots");
    }
}