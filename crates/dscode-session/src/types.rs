//! Core session types: lifecycle, state, events.
//!
//! These types are shared across all session modules and are the
//! primary public API surface of this crate.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::extensions::ExtensionInfo;

/// Session lifecycle states.
///
/// Tracks the overall lifecycle of the application session, from creation
/// through initialization to eventual shutdown.
///
/// State Diagram:
///
///   Uninitialized -> Initializing -> Ready -> ShuttingDown -> Shutdown
///                       |                          ^
///                       | (error)                  |
///                       v                          |
///                     Error ------------------------+
///                       |
///                       | (retry)
///                       v
///                     Initializing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionLifecycle {
    /// The session has not yet been created.
    Uninitialized,
    /// The session is loading extensions and initializing subsystems.
    Initializing,
    /// The session is fully initialized and ready for interaction.
    Ready,
    /// The session is shutting down gracefully.
    ShuttingDown,
    /// The session has completed shutdown.
    Shutdown,
    /// An unrecoverable error occurred during initialization.
    Error,
}

impl SessionLifecycle {
    /// Validates whether a transition from the current state to `to` is allowed.
    ///
    /// # Arguments
    ///
    /// * `to` - The target lifecycle state to transition to.
    pub fn can_transition_to(&self, to: SessionLifecycle) -> bool {
        matches!(
            (self, to),
            (
                SessionLifecycle::Uninitialized,
                SessionLifecycle::Initializing
            ) | (SessionLifecycle::Initializing, SessionLifecycle::Ready)
                | (SessionLifecycle::Initializing, SessionLifecycle::Error)
                | (SessionLifecycle::Ready, SessionLifecycle::ShuttingDown)
                | (SessionLifecycle::ShuttingDown, SessionLifecycle::Shutdown)
                | (SessionLifecycle::Error, SessionLifecycle::Initializing)
                | (SessionLifecycle::Error, SessionLifecycle::ShuttingDown)
        )
    }
}

/// Application session state snapshot.
///
/// Captures the current state of the IDE session at a point in time,
/// including open workspace folders, loaded extensions, and UI state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Paths of the currently open workspace folders.
    pub workspace_folders: Vec<PathBuf>,
    /// Extensions that are currently loaded and active.
    pub active_extensions: Vec<ExtensionInfo>,
    /// All extensions installed on disk.
    pub installed_extensions: Vec<ExtensionInfo>,
    /// Command IDs contributed by active extensions.
    pub available_commands: Vec<String>,
    /// Current status bar entries.
    pub status_bar_items: Vec<StatusBarItemState>,
}

/// Status bar item state for serialization.
///
/// Represents a single entry in the IDE status bar, including its
/// text, alignment, optional tooltip, color, and click command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarItemState {
    /// Unique identifier for this status bar item.
    pub id: String,
    /// Extension ID that owns this item.
    pub owner: String,
    /// Text to display (may include icon codicons like `$(git-branch)`).
    pub text: String,
    /// Optional tooltip shown on hover.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
    /// Optional CSS color for the item text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Optional command to execute when the item is clicked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<StatusBarCommand>,
    /// Alignment within the status bar (`"left"` or `"right"`).
    pub alignment: String,
    /// Optional priority controlling sort order within the alignment group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
}

/// A command attached to a status bar item.
///
/// When a status bar item is clicked, the referenced command is executed
/// with the given arguments (if any).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarCommand {
    /// The command identifier to execute.
    pub id: String,
    /// Optional JSON arguments to pass to the command handler.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<serde_json::Value>>,
}

/// Events emitted from the session to the UI layer.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum SessionEvent {
    /// Extension was installed
    ExtensionInstalled { extension_id: String },

    /// Extension was loaded/activated
    ExtensionLoaded { extension_id: String },

    /// Extension was unloaded/deactivated
    ExtensionUnloaded { extension_id: String },

    /// Extension was deleted
    ExtensionDeleted { extension_id: String },

    /// Extension list changed
    ExtensionsChanged { extensions: Vec<ExtensionInfo> },

    /// Workspace folder added
    WorkspaceFolderAdded { path: PathBuf },

    /// Workspace folder removed
    WorkspaceFolderRemoved { path: PathBuf },

    /// Session state changed
    StateChanged { state: SessionState },

    /// Available commands changed
    CommandsChanged { commands: Vec<String> },

    /// Status bar items changed
    StatusBarItems { items: Vec<StatusBarItemState> },

    /// Window message request
    WindowMessage {
        level: String,
        message: String,
        actions: Option<Vec<String>>,
    },

    /// Window message with actionable items (awaiting user response)
    WindowActionRequest {
        id: String,
        level: String,
        message: String,
        actions: Vec<String>,
    },

    /// Status bar transient message shown
    StatusBarMessageShown { id: String, text: String },

    /// Status bar transient message cleared
    StatusBarMessageCleared { id: String },

    /// Output channel registered
    OutputChannelRegistered { channel: String },

    /// Output channel received new content
    OutputChannelAppended { channel: String, value: String },

    /// Output channel cleared
    OutputChannelCleared { channel: String },

    /// Output channel disposed
    OutputChannelDisposed { channel: String },

    /// Output channel visibility toggled
    OutputChannelVisibility { channel: String, visible: bool },

    /// Tree view reveal request from extension host
    TreeViewReveal {
        view_id: String,
        element: serde_json::Value,
        options: serde_json::Value,
    },

    /// Configuration changed
    ConfigurationChanged {
        section: Option<String>,
        key: Option<String>,
    },

    /// Document content changed
    DocumentChanged { path: String, content: String },

    /// Editor decorations updated
    EditorDecorations {
        uri: String,
        key: String,
        decorations: serde_json::Value,
    },

    /// Quick pick selection required
    QuickPickRequest {
        id: String,
        items: Vec<serde_json::Value>,
        can_pick_many: bool,
        place_holder: Option<String>,
        title: Option<String>,
        match_on_description: bool,
        match_on_detail: bool,
    },

    /// Input box value requested
    InputBoxRequest {
        id: String,
        prompt: Option<String>,
        place_holder: Option<String>,
        value: Option<String>,
        password: bool,
        value_selection: Option<(usize, usize)>,
    },

    /// Execute command request from extension host
    ExecuteCommandRequest {
        id: String,
        command: String,
        args: Vec<serde_json::Value>,
    },

    /// Language feature provider registered
    ProviderRegistered {
        provider_type: String,
        provider_id: String,
        owner: String,
        selector: serde_json::Value,
        trigger_characters: Option<Vec<String>>,
        metadata: Option<serde_json::Value>,
    },

    /// Language configuration changed
    LanguageConfigurationChanged {
        language: String,
        configuration: serde_json::Value,
    },

    /// Diagnostics updated
    DiagnosticsUpdated {
        uri: String,
        diagnostics: serde_json::Value,
    },

    /// Diagnostics cleared
    DiagnosticsCleared { uri: String },

    /// Context key changed (setContext)
    ContextChanged {
        key: String,
        value: serde_json::Value,
    },

    /// Request to show an open-folder dialog (from extension host)
    OpenFolderDialog,

    /// Request to show an open-file dialog (from extension host)
    OpenFileDialog,

    /// Request to open a specific folder path (from extension host)
    OpenFolder { uri: String },
}

/// Trait for emitting session events to the UI layer.
///
/// In the Tauri build, this is backed by `tauri::AppHandle::emit`.
/// In headless/testing builds, a no-op or mock implementation can be used.
pub trait EventEmitter: Send + Sync {
    /// Emit a session event.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to emit to the UI layer.
    fn emit_event(&self, event: &SessionEvent);
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── StatusBarItemState serde round-trip ──────────────────────────────────────

    #[test]
    fn test_status_bar_item_state_serde_roundtrip() {
        let item = StatusBarItemState {
            id: "status.git".to_string(),
            owner: "git".to_string(),
            text: "$(git-branch) main".to_string(),
            tooltip: Some("Current branch".to_string()),
            color: None,
            command: Some(StatusBarCommand {
                id: "git.checkout".to_string(),
                arguments: None,
            }),
            alignment: "left".to_string(),
            priority: Some(1.0),
        };
        let json = serde_json::to_string(&item).unwrap();
        let deserialized: StatusBarItemState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, item.id);
        assert_eq!(deserialized.owner, item.owner);
        assert_eq!(deserialized.text, item.text);
        assert_eq!(deserialized.tooltip, item.tooltip);
        assert_eq!(deserialized.color, item.color);
        assert_eq!(deserialized.command.unwrap().id, "git.checkout");
        assert_eq!(deserialized.alignment, item.alignment);
        assert_eq!(deserialized.priority, item.priority);
    }

    #[test]
    fn test_status_bar_item_state_minimal() {
        let item = StatusBarItemState {
            id: "test".to_string(),
            owner: "test-owner".to_string(),
            text: "hello".to_string(),
            tooltip: None,
            color: None,
            command: None,
            alignment: "right".to_string(),
            priority: None,
        };
        let json = serde_json::to_string(&item).unwrap();
        // None fields with skip_serializing_if should not appear in JSON
        assert!(!json.contains("tooltip"));
        assert!(!json.contains("color"));
        assert!(!json.contains("command"));
        assert!(!json.contains("priority"));
        let deserialized: StatusBarItemState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test");
        assert!(deserialized.tooltip.is_none());
        assert!(deserialized.command.is_none());
    }

    #[test]
    fn test_status_bar_command_with_arguments() {
        let cmd = StatusBarCommand {
            id: "openFile".to_string(),
            arguments: Some(vec![serde_json::json!("/path/to/file.rs")]),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: StatusBarCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "openFile");
        assert_eq!(deserialized.arguments.unwrap().len(), 1);
    }

    // ── SessionLifecycle state transitions ───────────────────────────────────────

    #[test]
    fn test_session_lifecycle_valid_transitions() {
        assert!(SessionLifecycle::Uninitialized.can_transition_to(SessionLifecycle::Initializing));
        assert!(SessionLifecycle::Initializing.can_transition_to(SessionLifecycle::Ready));
        assert!(SessionLifecycle::Initializing.can_transition_to(SessionLifecycle::Error));
        assert!(SessionLifecycle::Ready.can_transition_to(SessionLifecycle::ShuttingDown));
        assert!(SessionLifecycle::ShuttingDown.can_transition_to(SessionLifecycle::Shutdown));
        assert!(SessionLifecycle::Error.can_transition_to(SessionLifecycle::Initializing));
        assert!(SessionLifecycle::Error.can_transition_to(SessionLifecycle::ShuttingDown));
    }

    #[test]
    fn test_session_lifecycle_invalid_transitions() {
        assert!(!SessionLifecycle::Uninitialized.can_transition_to(SessionLifecycle::Ready));
        assert!(!SessionLifecycle::Uninitialized.can_transition_to(SessionLifecycle::Shutdown));
        assert!(!SessionLifecycle::Ready.can_transition_to(SessionLifecycle::Initializing));
        assert!(!SessionLifecycle::Ready.can_transition_to(SessionLifecycle::Error));
        assert!(!SessionLifecycle::Shutdown.can_transition_to(SessionLifecycle::Ready));
        assert!(!SessionLifecycle::Shutdown.can_transition_to(SessionLifecycle::Initializing));
    }

    #[test]
    fn test_session_lifecycle_equality() {
        assert_eq!(SessionLifecycle::Ready, SessionLifecycle::Ready);
        assert_ne!(SessionLifecycle::Ready, SessionLifecycle::Initializing);
    }

    // ── SessionEvent serialization ───────────────────────────────────────────────

    #[test]
    fn test_session_event_extension_installed() {
        let event = SessionEvent::ExtensionInstalled {
            extension_id: "test.ext".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        // With #[serde(tag = "type", content = "data")], the variant name is the tag
        assert!(json.contains("ExtensionInstalled"), "JSON was: {}", json);
        assert!(json.contains("test.ext"));
    }

    #[test]
    fn test_session_event_commands_changed() {
        let event = SessionEvent::CommandsChanged {
            commands: vec!["cmd1".to_string(), "cmd2".to_string()],
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("CommandsChanged"), "JSON was: {}", json);
        assert!(json.contains("cmd1"));
        assert!(json.contains("cmd2"));
    }

    #[test]
    fn test_session_event_serialization_structure() {
        // Verify the tag/content structure
        let event = SessionEvent::ExtensionInstalled {
            extension_id: "my.ext".to_string(),
        };
        let value = serde_json::to_value(&event).unwrap();
        assert_eq!(
            value.get("type").and_then(|v| v.as_str()),
            Some("ExtensionInstalled")
        );
        assert!(value.get("data").is_some());
    }
}
