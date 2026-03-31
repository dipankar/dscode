use thiserror::Error;

/// Errors that can occur in dscode-core operations.
#[derive(Error, Debug)]
pub enum CoreError {
    /// Configuration error (invalid settings, missing required fields).
    #[error("Configuration error: {0}")]
    Config(String),

    /// Failed to resolve a platform-specific directory path.
    #[error("Path resolution failed: {0}")]
    PathResolution(String),

    /// An I/O error occurred during file or directory operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<CoreError> for String {
    fn from(err: CoreError) -> Self {
        err.to_string()
    }
}