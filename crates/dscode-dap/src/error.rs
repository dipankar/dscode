use thiserror::Error;

/// Errors that can occur in Debug Adapter Protocol operations.
#[derive(Error, Debug)]
pub enum DapError {
    /// Failed to spawn a debug adapter process.
    #[error("Debug adapter spawn failed for {session_id}: {reason}")]
    SpawnFailed { session_id: String, reason: String },
    /// The debug adapter process exited unexpectedly.
    #[error("Debug adapter crashed: {session_id}")]
    Crashed { session_id: String },
    /// An invalid state transition was attempted on the debug adapter.
    #[error("Invalid state transition: {from_state} -> {to_state}")]
    InvalidTransition { from_state: String, to_state: String },
    /// A DAP request exceeded its timeout.
    #[error("DAP request timed out after {timeout_secs}s")]
    RequestTimeout { timeout_secs: u64 },
    /// No debug adapter is registered for the given session.
    #[error("No debug adapter found for session: {0}")]
    NotRegistered(String),
    /// An I/O error occurred during DAP communication.
    #[error("DAP I/O error: {0}")]
    Io(String),
    /// A DAP protocol error (malformed messages, unexpected responses).
    #[error("DAP protocol error: {0}")]
    Protocol(String),
}

impl From<DapError> for String {
    fn from(err: DapError) -> Self {
        err.to_string()
    }
}