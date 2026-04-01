# Changelog

All notable changes to DSCode will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-04-16

### Added

- VS Code extension compatibility via Node.js extension host with NNG IPC
- Monaco Editor integration with Svelte 4 frontend
- Language provider support for 22 types: Hover, Completion, Diagnostics, SignatureHelp, Rename, CodeLens, CodeActions, Formatting, DocumentHighlights, FoldingRanges, SemanticTokens, DocumentSymbols, WorkspaceSymbols, Definitions, References, DocumentLinks, ColorPresentations, InlineCompletions, CallHierarchy, TypeDefinition, SelectionRanges, and LinkedEditing
- LSP integration via tower-lsp with connection pooling and multi-language support
- Debug Adapter Protocol (DAP) integration for debugging support
- Integrated terminal via xterm.js with PTY support (portable-pty)
- Git integration powered by libgit2 (git2 crate) — clone, diff, blame, branch management
- Extension marketplace for browsing and installing VS Code extensions
- VS Code-compatible theming via CSS custom properties
- Virtualized file tree for fast directory navigation in large workspaces
- Ripgrep-based search (integrated via ignore + grep crates)
- File watcher with notify crate for real-time change detection
- Rope-based text storage (ropey) for efficient editing
- Tree-sitter incremental parsing for syntax awareness
- Command palette and quick open for fast navigation
- Diff viewer for comparing file changes
- Extension sandbox deny-by-default security model with platform isolation
- PathValidator for filesystem access control and traversal prevention
- SecretStorage API backed by OS-native keyring
- Extension host crash recovery with exponential backoff (max 3 restart attempts)
- Resource monitoring panel (sysinfo-based)
- Configuration management system
- Svelte component architecture with 34 components and 11 stores
- Lazy-loaded overlays for performance

### Changed

- Initial release — no prior versions to compare

### Security

- Deny-by-default extension sandbox (macOS: sandbox-exec, Linux: bubblewrap)
- PathValidator enforces workspace allowlist for all filesystem IPC
- Zip Slip prevention during VSIX extraction
- OS keyring integration for extension secret storage

## Deprecation Policy

DSCode follows Semantic Versioning. As a `0.x` project:

- **Minor releases** (`0.x.0`) may introduce breaking API changes.
- **Patch releases** (`0.x.y`) are backwards-compatible bug fixes.
- Public APIs marked with `#[deprecated]` will be kept for at least one minor release cycle before removal.
- Migration notes for breaking changes will be added to this changelog under the relevant version.

## Migration Notes

This is the initial release — no prior versions exist to migrate from.
