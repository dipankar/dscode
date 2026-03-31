//! # dscode-core
//!
//! Core types, text buffer, and configuration for DSCode.
//!
//! This crate provides fundamental building blocks used across all DSCode components:
//!
//! - [`TextBuffer`] — Rope-based text storage for efficient editing operations
//! - [`AppDirectories`] — Platform-aware directory resolution for extensions, storage, and logs
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

mod config;
mod error;
mod text_buffer;
// mod syntax; // Placeholder for tree-sitter integration

pub use config::AppDirectories;
pub use error::CoreError;
pub use text_buffer::TextBuffer;