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
