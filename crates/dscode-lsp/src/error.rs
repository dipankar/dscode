use thiserror::Error;

/// Errors that can occur in LSP operations.
#[derive(Error, Debug)]
pub enum LspError {
    /// Failed to spawn a language server process.
    #[error("Failed to spawn language server for {language}: {reason}")]
    SpawnFailed { language: String, reason: String },

    /// LSP initialize handshake failed.
    #[error("LSP handshake failed for {language}: {detail}")]
    HandshakeFailed { language: String, detail: String },

    /// An LSP request exceeded its timeout.
    #[error("LSP request timed out for {language} after {timeout_secs}s")]
    RequestTimeout { language: String, timeout_secs: u64 },

    /// The language server process exited unexpectedly.
    #[error("LSP server crashed: {language}")]
    ServerCrashed { language: String },

    /// An invalid state transition was attempted on the LSP client.
    #[error("Invalid state transition for {language}: {from_state} -> {to_state}")]
    InvalidTransition {
        language: String,
        from_state: String,
        to_state: String,
    },

    /// No language server is registered for the given language.
    #[error("No language server registered for: {0}")]
    NotRegistered(String),

    /// An I/O error occurred during LSP communication.
    #[error("LSP I/O error: {0}")]
    Io(String),

    /// An LSP protocol error (malformed messages, unexpected responses).
    #[error("LSP protocol error: {0}")]
    Protocol(String),
}

impl From<LspError> for String {
    fn from(err: LspError) -> Self {
        err.to_string()
    }
}
