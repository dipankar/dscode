/**
 * LSP (Language Server Protocol) Manager
 *
 * Manages language server processes and handles communication
 * between the editor and language servers
 */
pub mod client;
pub mod manager;
pub mod pool;

pub use client::{LspClient, LspClientState};
pub use manager::LspManager;
pub use pool::{LspPoolStats, LspServerPool, LspServerStrategy};
