//! # dscode-extension-host
//!
//! Extension host process management, IPC, sandbox, and security for DSCode.
//!
//! This crate provides the infrastructure for running and communicating with

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(dead_code)]
//! extension host processes, including:
//!
//! - **Process lifecycle management** with a state machine ([`manager::ExtensionHostManager`])
//! - **Bidirectional IPC** over Unix domain sockets ([`ipc::IpcManager`])
//! - **Path validation** to prevent filesystem traversal attacks ([`path_validator::PathValidator`])
//! - **Capability-based permissions** for extensions ([`permissions::ExtensionPermissions`])
//! - **Per-extension rate limiting** using token bucket algorithm ([`rate_limiter::RateLimiter`])
//! - **Multi-platform sandboxing** (macOS sandbox-exec, Linux bwrap, Windows Job Objects) ([`sandbox`])
//! - **OS keyring secret storage** ([`secrets::SecretStorage`])
//!
//! ## Feature Flags
//!
//! - `tauri` — Enables Tauri integration for the extension host manager

mod binary_verifier;
mod error;
mod ipc;
mod manager;
mod path_validator;
mod permissions;
mod rate_limiter;
mod sandbox;
mod secrets;

pub use binary_verifier::verify_binary;
pub use error::ExtensionHostError;
pub use ipc::{ExtensionIpc, IncomingIpc, IncomingRequestHandler, IpcManager};
pub use manager::{ExtensionHostManager, ExtensionHostState};
pub use path_validator::PathValidator;
pub use permissions::{ExtensionPermissions, Permission};
pub use rate_limiter::RateLimiter;
pub use sandbox::{apply_sandbox, complete_sandbox_setup, SandboxConfig};
pub use secrets::SecretStorage;
