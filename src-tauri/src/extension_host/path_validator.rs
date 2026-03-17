/**
 * Path Validation and Sandboxing
 *
 * Prevents path traversal attacks and ensures extensions can only access
 * files within allowed directories (workspace, extensions folder, temp).
 */

use std::path::{Path, PathBuf};
use std::fs;

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
        Self {
            allowed_roots: Vec::new(),
            extensions_dir: None,
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

    pub fn set_temp_dir(&mut self, path: PathBuf) {
        if let Ok(canonical) = fs::canonicalize(&path) {
            self.temp_dir = Some(canonical);
        }
    }

    /// Validate and normalize a path from a URI
    pub fn validate_path(&self, uri: &str) -> Result<PathBuf, String> {
        // Parse file:// URI
        let path_str = uri.strip_prefix("file://").unwrap_or(uri);
        let path = Path::new(path_str);

        // Resolve to canonical path (follows symlinks, resolves ..)
        let canonical = fs::canonicalize(path)
            .map_err(|e| Self::sanitize_error(&format!("Invalid path: {}", e)))?;

        // Check if path is within allowed directories
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
    pub fn sanitize_error(error: &str) -> String {
        // Remove full paths from error messages
        // Replace absolute paths with relative or generic messages
        if error.contains("/") || error.contains("\\") {
            "Operation failed: Invalid or inaccessible path".to_string()
        } else {
            error.to_string()
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
        canonical_path.file_name()
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
        let error = PathValidator::sanitize_error("/home/user/secret/file.txt: No such file");
        assert!(!error.contains("/home"));
        assert!(!error.contains("/secret"));
    }
}
