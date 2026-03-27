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
    /// Temp directory for this workspace
    temp_dir: Option<PathBuf>,
}

impl PathValidator {
    pub fn new() -> Self {
        Self { allowed_roots: Vec::new(), extensions_dir: None, temp_dir: None }
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

    pub fn set_temp_dir(&mut self, path: PathBuf) {
        if let Ok(canonical) = fs::canonicalize(&path) {
            self.temp_dir = Some(canonical);
        }
    }

    /// Validate and normalize a path from a URI
    pub fn validate_path(&self, uri: &str) -> Result<PathBuf, String> {
        let path_str = if uri.starts_with("file:///") {
            &uri[7..]
        } else if uri.starts_with("file://") {
            let after_slashes = &uri[7..];
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
        // Check workspace folders
        for root in &self.allowed_roots {
            if canonical_path.starts_with(root) {
                return true;
            }
        }

        // Check extensions directory
        if let Some(ext_dir) = &self.extensions_dir {
            if canonical_path.starts_with(ext_dir) {
                return true;
            }
        }

        // Check temp directory
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
            .map(|n| PathBuf::from(n))
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

        // This file should be accessible
        let this_file = current_dir.join("src/extension_host/path_validator.rs");
        let uri = format!("file://{}", this_file.display());

        if this_file.exists() {
            assert!(validator.validate_path(&uri).is_ok());
        }

        // Parent directory traversal should fail
        let parent_attack = "file://../../../etc/passwd";
        assert!(validator.validate_path(parent_attack).is_err());
    }

    #[test]
    fn test_error_sanitization() {
        let error = PathValidator::sanitize_error(&"/home/user/secret/file.txt: No such file");
        assert!(!error.contains("/home"));
        assert!(!error.contains("/secret"));
    }
}
