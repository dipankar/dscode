//! # dscode-lsp
//!
//! Language Server Protocol client, manager, and connection pool for DSCode.
//!
//! This crate provides a complete LSP client implementation with:
//! - [`LspClient`] — State-machine-based LSP client that manages a language server process
//! - [`LspManager`] — Registry for language servers with lazy startup
//! - [`LspServerPool`] — Connection pool with configurable strategies (one-per-language, multiple)
//!
//! # State Machine
//!
//! The `LspClient` follows a strict lifecycle:
//!
//! ```text
//! Stopped → Starting → Initializing → Ready → ShuttingDown → Stopped
//!                ↘ Crashed ↗                         ↗
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use dscode_lsp::{LspManager, LspServerPool, LspServerStrategy};
//!
//! # async fn example() {
//! let manager = LspManager::new();
//! manager.register_server("rust", "rust-analyzer", vec![]).await;
//!
//! let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);
//! pool.register_server("rust".to_string(), "rust-analyzer".to_string(), vec![]).await;
//! # }
//! ```

mod client;
mod error;
mod manager;
mod pool;

pub use client::{LspClient, LspClientState};
pub use error::LspError;
pub use manager::LspManager;
pub use pool::{LspPoolStats, LspServerPool, LspServerStrategy};