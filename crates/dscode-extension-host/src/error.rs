use thiserror::Error;

/// Errors that can occur in extension host operations.
#[derive(Error, Debug)]
pub enum ExtensionHostError {
    /// Failed to spawn the extension host process.
    #[error("Extension host spawn failed: {0}")]
    SpawnFailed(String),
    /// IPC communication error with the extension host.
    #[error("IPC connection error: {0}")]
    IpcError(String),
    /// The requested extension was not found.
    #[error("Extension not found: {0}")]
    ExtensionNotFound(String),
    /// The extension is already loaded and cannot be loaded again.
    #[error("Extension already loaded: {0}")]
    AlreadyLoaded(String),
    /// Sandbox configuration or application error.
    #[error("Sandbox error: {0}")]
    SandboxError(String),
    /// Path validation failed (traversal attack or outside workspace).
    #[error("Path validation failed: {0}")]
    PathValidation(String),
    /// The extension does not have the required permission.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    /// The extension has exceeded its rate limit.
    #[error("Rate limit exceeded")]
    RateLimited,
    /// Secret storage operation failed.
    #[error("Secret storage error: {0}")]
    SecretError(String),
    /// Invalid state transition for the extension host.
    #[error("Invalid state: {from} -> {to}")]
    InvalidState { from: String, to: String },
}
