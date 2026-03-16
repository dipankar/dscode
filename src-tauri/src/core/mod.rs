// Core editor functionality will go here
// - Text buffer (ropey)
// - Syntax parsing (tree-sitter)
// - LSP integration
// - Configuration management

pub mod text_buffer;
pub mod syntax;

pub use text_buffer::*;
pub use syntax::*;
