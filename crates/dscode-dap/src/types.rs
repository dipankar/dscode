use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub name: Option<String>,
    pub path: Option<String>,
    #[serde(rename = "sourceReference")]
    pub source_reference: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: Option<i64>,
    #[serde(default)]
    pub verified: bool,
    pub message: Option<String>,
    pub source: Option<Source>,
    pub line: Option<i64>,
    pub column: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceBreakpoint {
    pub line: i64,
    pub column: Option<i64>,
    pub condition: Option<String>,
    #[serde(rename = "hitCondition")]
    pub hit_condition: Option<String>,
    #[serde(rename = "logMessage")]
    pub log_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub id: i64,
    pub name: String,
    pub source: Option<Source>,
    pub line: i64,
    pub column: i64,
    #[serde(rename = "endLine")]
    pub end_line: Option<i64>,
    #[serde(rename = "endColumn")]
    pub end_column: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value: String,
    #[serde(rename = "type")]
    pub var_type: Option<String>,
    #[serde(rename = "variablesReference")]
    pub variables_reference: i64,
    #[serde(rename = "namedVariables")]
    pub named_variables: Option<i64>,
    #[serde(rename = "indexedVariables")]
    pub indexed_variables: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub name: String,
    #[serde(rename = "variablesReference")]
    pub variables_reference: i64,
    pub expensive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequestArguments {
    pub program: String,
    pub args: Option<Vec<String>>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub stop_on_entry: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum DebugState {
    Stopped,
    Initialized,
    Running,
    Paused,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSession {
    pub id: String,
    pub name: String,
    pub state: DebugState,
    pub adapter_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_session_serialization() {
        let session = DebugSession {
            id: "session-1".to_string(),
            name: "Debug Main".to_string(),
            state: DebugState::Running,
            adapter_type: "cppdbg".to_string(),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&session).expect("Failed to serialize DebugSession");

        // Verify key fields are in the JSON output
        assert!(json.contains("session-1"));
        assert!(json.contains("Debug Main"));
        assert!(json.contains("Running"));
        assert!(json.contains("cppdbg"));

        // Deserialize back and verify round-trip
        let deserialized: DebugSession =
            serde_json::from_str(&json).expect("Failed to deserialize DebugSession");
        assert_eq!(deserialized.id, session.id);
        assert_eq!(deserialized.name, session.name);
        assert_eq!(deserialized.state, session.state);
        assert_eq!(deserialized.adapter_type, session.adapter_type);

        // Test each DebugState variant
        for state in [
            DebugState::Stopped,
            DebugState::Initialized,
            DebugState::Running,
            DebugState::Paused,
            DebugState::Terminated,
        ] {
            let s = DebugSession {
                id: "test".to_string(),
                name: "test".to_string(),
                state: state.clone(),
                adapter_type: "test-adapter".to_string(),
            };
            let json = serde_json::to_string(&s).unwrap();
            let roundtrip: DebugSession = serde_json::from_str(&json).unwrap();
            assert_eq!(roundtrip.state, state);
        }
    }

    #[test]
    fn test_breakpoint_serialization() {
        // Test Breakpoint round-trip
        let bp = Breakpoint {
            id: Some(42),
            verified: true,
            message: Some("Breakpoint set".to_string()),
            source: Some(Source {
                name: Some("main.rs".to_string()),
                path: Some("/src/main.rs".to_string()),
                source_reference: None,
            }),
            line: Some(10),
            column: Some(5),
        };

        let json = serde_json::to_string(&bp).expect("Failed to serialize Breakpoint");
        let roundtrip: Breakpoint =
            serde_json::from_str(&json).expect("Failed to deserialize Breakpoint");
        assert_eq!(roundtrip.id, bp.id);
        assert_eq!(roundtrip.verified, bp.verified);
        assert_eq!(roundtrip.message, bp.message);
        assert_eq!(roundtrip.line, bp.line);
        assert_eq!(roundtrip.column, bp.column);

        // Test SourceBreakpoint round-trip
        let sbp = SourceBreakpoint {
            line: 25,
            column: Some(8),
            condition: Some("x > 0".to_string()),
            hit_condition: Some("5".to_string()),
            log_message: Some("Hit breakpoint at line 25".to_string()),
        };

        let json = serde_json::to_string(&sbp).expect("Failed to serialize SourceBreakpoint");
        let roundtrip: SourceBreakpoint =
            serde_json::from_str(&json).expect("Failed to deserialize SourceBreakpoint");
        assert_eq!(roundtrip.line, sbp.line);
        assert_eq!(roundtrip.column, sbp.column);
        assert_eq!(roundtrip.condition, sbp.condition);
        assert_eq!(roundtrip.hit_condition, sbp.hit_condition);
        assert_eq!(roundtrip.log_message, sbp.log_message);

        // Test minimal SourceBreakpoint (only required field)
        let minimal = SourceBreakpoint {
            line: 1,
            column: None,
            condition: None,
            hit_condition: None,
            log_message: None,
        };
        let json = serde_json::to_string(&minimal).unwrap();
        let roundtrip: SourceBreakpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.line, 1);
        assert!(roundtrip.column.is_none());
    }
}
