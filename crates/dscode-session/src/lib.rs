//! # dscode-session
//!
//! Session manager, extension lifecycle, workspace, and configuration for DSCode.
//!
//! This crate provides the central coordination layer for the DSCode IDE, managing:
//! - Extension lifecycle (install, load, unload, delete)
//! - LSP/DAP server coordination
//! - Application state synchronization
//! - Event emission to UI
//! - Workspace and configuration management
//!
//! ## Feature Flags
//!
//! - `tauri` — Enables Tauri integration (event emission via `AppHandle`)

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

pub use configuration::ConfigurationStore;
pub use contributions::ExtensionContributes;
pub use documents::{detect_language_id, DocumentManager, PositionPayload, RangePayload, TextEditPayload};
pub use extensions::{ExtensionContribution, ExtensionInfo, InstalledExtension};
pub use types::{
    EventEmitter, SessionEvent, SessionLifecycle, SessionState, StatusBarCommand, StatusBarItemState,
};
pub use workspace::{
    workspace_path_from_uri, workspace_uri_from_path, FileDecoration, FileDecorationProvider,
    FindFilesOptions, TextSearchMatch, TextSearchOptions, TextSearchResult, WorkspaceConfiguration,
    WorkspaceFolder, WorkspaceManager,
};

// Re-export types from dependency crates for convenience
pub use dscode_core::AppDirectories;
pub use dscode_dap::{DebugAdapterPool, DebugPoolStats};
pub use dscode_extension_host::{
    ExtensionHostManager, ExtensionHostState, IpcManager, PathValidator, SecretStorage,
};
pub use dscode_lsp::{LspServerPool, LspServerStrategy};
