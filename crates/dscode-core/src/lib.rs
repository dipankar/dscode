//! # dscode-core
//!
//! Core types, text buffer, and configuration for the DSCode IDE.
//!
//! This crate provides the fundamental building blocks used across all DSCode
//! components:
//!
//! - [`TextBuffer`] — Rope-based text storage for efficient editing operations
//! - [`AppDirectories`] — Platform-aware directory resolution for extensions, storage, and logs
//! - [`CoreError`] — Unified error type for core operations
//!
//! # Example
//!
//! ```rust
//! use dscode_core::TextBuffer;
//!
//! let buffer = TextBuffer::new("Hello, world!");
//! assert_eq!(buffer.len_chars(), 13);
//! assert_eq!(buffer.get_text(), "Hello, world!");
//! ```

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(dead_code)]

mod config;
mod error;
mod text_buffer;
// mod syntax; // Placeholder for tree-sitter integration

/// Re-export of [`AppDirectories`] — platform-aware directory resolution.
///
/// See [`AppDirectories::resolve`] for the full resolution logic including
/// environment variable overrides and user configuration files.
pub use config::AppDirectories;

/// Re-export of [`CoreError`] — the unified error type for dscode-core.
///
/// All fallible operations in this crate return `Result<T, CoreError>`.
pub use error::CoreError;

/// Re-export of [`TextBuffer`] — a rope-based text buffer for efficient editing.
///
/// Use [`TextBuffer::new`] to create a buffer from a string slice.
pub use text_buffer::TextBuffer;