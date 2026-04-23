//! # dscode-session
//!
//! Session manager, extension lifecycle, workspace, and configuration for DSCode.
//!
//! This crate provides the central coordination layer for the DSCode IDE, managing:
//!
//! - Extension lifecycle (install, load, unload, delete)
//! - LSP/DAP server coordination
//! - Application state synchronization
//! - Event emission to UI
//! - Workspace and configuration management
//!
//! ## Feature Flags
//!
//! - `tauri` — Enables Tauri integration (event emission via `AppHandle`)

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(dead_code)]

mod configuration;
mod contributions;
mod documents;
mod extensions;
mod marketplace;
mod types;
mod workspace;

// The `ipc` and `ipc_providers` modules have deep Tauri AppHandle coupling.
// They are included only when the `tauri` feature is enabled.
#[cfg(feature = "tauri")]
mod ipc;
#[cfg(feature = "tauri")]
mod ipc_providers;

/// Persistent configuration store backed by a JSON settings file.
pub use configuration::ConfigurationStore;
/// Declared contributions (commands, languages, grammars, etc.) from an extension's `package.json`.
pub use contributions::ExtensionContributes;
/// Detects the language ID for a file path based on its extension.
pub use documents::detect_language_id;
/// Manages document versions, decorations, and text-edit operations.
pub use documents::DocumentManager;
/// Zero-based line/character position in a text document.
pub use documents::PositionPayload;
/// Start/end positions describing a range in a text document.
pub use documents::RangePayload;
/// A single text replacement scoped to a range.
pub use documents::TextEditPayload;
/// Summary of contributions registered by a single extension.
pub use extensions::ExtensionContribution;
/// Runtime metadata for an extension visible to the session layer.
pub use extensions::ExtensionInfo;
/// Persisted metadata for an extension installed on disk.
pub use extensions::InstalledExtension;
/// Trait for emitting session events to the UI layer.
pub use types::EventEmitter;
/// Tagged event enum for all notifications sent from session to UI.
pub use types::SessionEvent;
/// Finite-state-machine for the application session lifecycle.
pub use types::SessionLifecycle;
/// Snapshot of the current application session state.
pub use types::SessionState;
/// A command attached to a status bar item.
pub use types::StatusBarCommand;
/// Serializable state of a single status bar entry.
pub use types::StatusBarItemState;
/// Converts a `file://` URI to a filesystem [`std::path::PathBuf`].
pub use workspace::workspace_path_from_uri;
/// Converts a filesystem path to a `file://` URI string.
pub use workspace::workspace_uri_from_path;
/// File decoration data (badge, tooltip, color, propagation).
pub use workspace::FileDecoration;
/// Registration record for a file decoration provider.
pub use workspace::FileDecorationProvider;
/// Options controlling a workspace-wide file search.
pub use workspace::FindFilesOptions;
/// A single match within a text search result.
pub use workspace::TextSearchMatch;
/// Options controlling a workspace-wide text search.
pub use workspace::TextSearchOptions;
/// A file URI together with its matched ranges.
pub use workspace::TextSearchResult;
/// A named section of workspace-scoped key/value settings.
pub use workspace::WorkspaceConfiguration;
/// A workspace folder with its URI, display name, and index.
pub use workspace::WorkspaceFolder;
/// Manages workspace folders, configurations, and file decorations.
pub use workspace::WorkspaceManager;

#[cfg(feature = "tauri")]
pub use ipc::{SessionManager, TauriEventEmitter};

// Re-export types from dependency crates for convenience
/// Well-known application directory paths.
pub use dscode_core::AppDirectories;
/// Pool of debug adapter protocol (DAP) server processes.
pub use dscode_dap::DebugAdapterPool;
/// Usage statistics for the DAP server pool.
pub use dscode_dap::DebugPoolStats;
/// Orchestrator for the extension host subprocess.
pub use dscode_extension_host::ExtensionHostManager;
/// Lifecycle state of the extension host process.
pub use dscode_extension_host::ExtensionHostState;
/// Manages IPC communication with the extension host.
pub use dscode_extension_host::IpcManager;
/// Validates filesystem paths for sandbox safety.
pub use dscode_extension_host::PathValidator;
/// Extension secret storage backed by the OS keychain.
pub use dscode_extension_host::SecretStorage;
/// Pool of language server protocol (LSP) server processes.
pub use dscode_lsp::LspServerPool;
/// Strategy for spawning LSP server instances.
pub use dscode_lsp::LspServerStrategy;
