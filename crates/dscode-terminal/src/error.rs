use thiserror::Error;

/// Errors that can occur in terminal operations.
#[derive(Error, Debug)]
pub enum TerminalError {
    /// Failed to create a new terminal instance.
    #[error("Terminal creation failed: {0}")]
    CreationFailed(String),
    /// The requested terminal ID was not found.
    #[error("Terminal not found: {0}")]
    NotFound(String),
    /// An I/O error occurred in the PTY communication.
    #[error("PTY I/O error: {0}")]
    PtyError(String),
    /// The requested terminal profile was not found.
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),
    /// The requested shell binary is not available on this system.
    #[error("Shell not available: {0}")]
    ShellNotAvailable(String),
}
