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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dap_error_spawn_failed_display() {
        let err = DapError::SpawnFailed {
            session_id: "sess-1".to_string(),
            reason: "command not found".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("spawn failed"), "Display should contain 'spawn failed'");
        assert!(msg.contains("sess-1"), "Display should contain session id");
        assert!(msg.contains("command not found"), "Display should contain reason");
    }

    #[test]
    fn test_dap_error_crashed_display() {
        let err = DapError::Crashed { session_id: "sess-2".to_string() };
        let msg = err.to_string();
        assert!(msg.contains("crashed"), "Display should contain 'crashed'");
        assert!(msg.contains("sess-2"), "Display should contain session id");
    }

    #[test]
    fn test_dap_error_invalid_transition_display() {
        let err = DapError::InvalidTransition {
            from_state: "Stopped".to_string(),
            to_state: "Running".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid state transition"), "Display should contain transition label");
        assert!(msg.contains("Stopped"), "Display should contain from_state");
        assert!(msg.contains("Running"), "Display should contain to_state");
    }

    #[test]
    fn test_dap_error_request_timeout_display() {
        let err = DapError::RequestTimeout { timeout_secs: 30 };
        let msg = err.to_string();
        assert!(msg.contains("timed out"), "Display should contain 'timed out'");
        assert!(msg.contains("30"), "Display should contain timeout duration");
    }

    #[test]
    fn test_dap_error_not_registered_display() {
        let err = DapError::NotRegistered("sess-3".to_string());
        let msg = err.to_string();
        assert!(msg.contains("No debug adapter found"), "Display should contain 'No debug adapter found'");
        assert!(msg.contains("sess-3"), "Display should contain session id");
    }

    #[test]
    fn test_dap_error_io_display() {
        let err = DapError::Io("connection reset".to_string());
        let msg = err.to_string();
        assert!(msg.contains("I/O error"), "Display should contain 'I/O error'");
        assert!(msg.contains("connection reset"), "Display should contain the message");
    }

    #[test]
    fn test_dap_error_protocol_display() {
        let err = DapError::Protocol("malformed header".to_string());
        let msg = err.to_string();
        assert!(msg.contains("protocol error"), "Display should contain 'protocol error'");
        assert!(msg.contains("malformed header"), "Display should contain the message");
    }

    #[test]
    fn test_dap_error_into_string() {
        let err = DapError::SpawnFailed {
            session_id: "s".to_string(),
            reason: "r".to_string(),
        };
        let s: String = err.into();
        assert!(s.contains("spawn failed"), "Into<String> should produce the Display output");
    }

    #[test]
    fn test_dap_error_debug_format() {
        let err = DapError::Crashed { session_id: "s".to_string() };
        let debug = format!("{:?}", err);
        assert!(debug.contains("Crashed"), "Debug format should contain variant name");
    }
}