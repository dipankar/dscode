use thiserror::Error;

/// The unified error type for all fallible operations in `dscode-core`.
///
/// Every public function in this crate that can fail returns a
/// `Result<T, CoreError>`. The variants cover configuration errors,
/// path resolution failures, and I/O errors from the underlying filesystem.
///
/// # Display
///
/// Each variant implements [`std::fmt::Display`] via `thiserror`, so calling
/// `.to_string()` on a `CoreError` produces a human-readable message suitable
/// for logging or displaying to the user.
///
/// # Conversion
///
/// `CoreError` implements `From<std::io::Error>` so I/O errors can be
/// propagated with the `?` operator. It also implements `Into<String>` for
/// ergonomic string conversion.
#[derive(Error, Debug)]
pub enum CoreError {
    /// A configuration error, such as an invalid setting value or a missing
    /// required field in a configuration file.
    ///
    /// The contained `String` describes the specific configuration problem.
    #[error("Configuration error: {0}")]
    Config(String),

    /// A platform-specific directory path could not be resolved.
    ///
    /// This typically occurs when the home directory is unavailable or a
    /// user-configured path (e.g. `~` expansion) cannot be expanded.
    ///
    /// The contained `String` describes which path failed and why.
    #[error("Path resolution failed: {0}")]
    PathResolution(String),

    /// An I/O error occurred during a file or directory operation.
    ///
    /// This variant wraps [`std::io::Error`] and is automatically created via
    /// the `?` operator on any I/O call that fails (e.g. creating directories,
    /// reading configuration files).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Allows converting a [`CoreError`] into a [`String`] for ergonomic error
/// reporting.
///
/// This is useful when a string representation of the error is needed without
/// preserving the typed error value.
impl From<CoreError> for String {
    fn from(err: CoreError) -> Self {
        err.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_error_config_display() {
        let err = CoreError::Config("invalid setting".to_string());
        let msg = err.to_string();
        assert!(
            msg.contains("Configuration error"),
            "Display should contain variant label"
        );
        assert!(
            msg.contains("invalid setting"),
            "Display should contain the inner message"
        );
    }

    #[test]
    fn test_core_error_path_resolution_display() {
        let err = CoreError::PathResolution("home dir unavailable".to_string());
        let msg = err.to_string();
        assert!(
            msg.contains("Path resolution failed"),
            "Display should contain variant label"
        );
        assert!(
            msg.contains("home dir unavailable"),
            "Display should contain the inner message"
        );
    }

    #[test]
    fn test_core_error_io_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = CoreError::Io(io_err);
        let msg = err.to_string();
        assert!(
            msg.contains("I/O error"),
            "Display should contain variant label"
        );
        assert!(
            msg.contains("access denied"),
            "Display should contain the inner io message"
        );
    }

    #[test]
    fn test_core_error_into_string() {
        let err = CoreError::Config("bad config".to_string());
        let s: String = err.into();
        assert!(s.contains("Configuration error"));
        assert!(s.contains("bad config"));
    }

    #[test]
    fn test_core_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let core_err: CoreError = io_err.into();
        match core_err {
            CoreError::Io(_) => {} // expected
            other => panic!("Expected Io variant, got {:?}", other),
        }
    }

    #[test]
    fn test_core_error_debug_format() {
        // Ensure Debug formatting works without panic
        let err = CoreError::Config("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Config"));
    }
}
