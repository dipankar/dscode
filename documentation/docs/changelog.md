---
title: Changelog
description: A chronological record of all notable changes to DSCode, following the Keep a Changelog format.
---

# Changelog

All notable changes to DSCode are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

Work in progress for the next release. See the [Roadmap](roadmap.md) for planned features.

### In Progress

- Replace in files (single and bulk operations)
- Search filters (include/exclude patterns, file type filters)
- Visual keybinding editor with conflict detection
- Settings UI (searchable settings editor beyond JSON)

---

## [0.1.0-alpha] - 2025-11-01

Initial alpha release of DSCode. This release establishes the core editing experience, extension system, terminal, git integration, and language intelligence features.

### Added

#### Core Editor

- **Monaco Editor integration** with full support for IntelliSense, multi-cursor editing, minimap, bracket matching, code folding, and 200+ built-in language grammars
- **Tauri v2 backend** with 48 Rust command modules covering file operations, git, terminal, search, debugging, themes, keybindings, and more
- **Svelte frontend** with reactive UI shell, split views, and panel management
- **File operations** -- open, save, save as, rename, delete, and move with async Rust I/O
- **Tab management** -- multi-tab editing with dirty indicators, tab reordering, and tab context menus
- **Command palette** with fuzzy search across all registered commands
- **Activity bar** with Explorer, Search, Source Control, Extensions, and Debug views
- **Sidebar** with collapsible tree views and resizable panels
- **Status bar** showing language mode, encoding, line/column position, git branch, and notifications

#### Language Intelligence

- **LSP client** (tower-lsp) with multi-server support and automatic server discovery
- **4 built-in language server configurations** for Python (Pyright), Rust (rust-analyzer), Go (gopls), and JSON (vscode-json-language-server). Additional language servers can be added through extensions
- **Auto-completion** with LSP-driven suggestions, documentation preview, and signature help
- **Go-to-definition** and **find references** with peek views and cross-file navigation
- **Hover documentation** showing type information, documentation, and function signatures
- **Diagnostics** with inline error/warning squiggles and a dedicated Problems panel
- **Tree-sitter syntax analysis** scaffolded (stub -- not yet functional; the `parse()` function returns empty results pending tree-sitter integration)
- **Semantic highlighting** via LSP semantic tokens

#### Extension System

- **Node.js extension host** running in a separate process, communicating via NNG (nanomsg) IPC with rkyv zero-copy serialization for sub-millisecond latency
- **VS Code API compatibility** at approximately 75% coverage, including core namespaces:
    - `vscode.languages` -- language feature providers, diagnostics, completions
    - `vscode.workspace` -- file system access, configuration, workspace folders
    - `vscode.window` -- editors, terminals, UI elements, notifications
    - `vscode.commands` -- command registration and execution
    - `vscode.extensions` -- extension metadata and inter-extension communication
    - `vscode.env` -- environment information, clipboard, URI handling
    - `vscode.debug` -- debug session management
- **Open VSX marketplace integration** -- search, browse, install, and update extensions from the [Open VSX Registry](https://open-vsx.org/)
- **VSIX package installation** -- install extensions from local `.vsix` files
- **Activation events** -- onLanguage, onCommand, onView, workspaceContains, onStartupFinished, and `*`
- **Extension contributions** -- commands, keybindings, menus, configuration, themes, snippets, grammars, and views
- **Webview API** -- extension-hosted webview panels with messaging
- **Tree view API** -- custom sidebar tree views with data providers

#### Git Integration

- **Native git operations** via the `git2` Rust crate (no external `git` binary required)
- **Repository detection** with automatic `.git` directory discovery
- **Status tracking** -- modified, staged, untracked, and conflicted file indicators in the Explorer and SCM view
- **Staging and unstaging** -- stage individual files, hunks, or all changes
- **Committing** -- commit message editor with multi-line support and amend option
- **Diff viewer** -- inline and side-by-side diff views with syntax highlighting
- **Branch management** -- create, switch, rename, and delete branches
- **Remote operations** -- push, pull, fetch with credential support (HTTPS and SSH)
- **SCM sidebar view** -- dedicated source control panel with grouped changes

#### Terminal

- **Integrated terminal** powered by xterm.js for rendering and portable-pty for PTY management
- **Multiple terminal instances** with a tabbed interface
- **Shell auto-detection** for bash, zsh, fish, PowerShell, and cmd
- **Cross-platform support** -- native PTY handling on macOS, Linux, and Windows
- **Customizable appearance** -- configurable font family, font size, colors, cursor style, and scrollback buffer size

#### Search

- **Find in files** with full-text search across the workspace, powered by ripgrep
- **Regex search** with regular expression pattern matching
- **Search results view** with grouped results, file context preview, and click-to-navigate

#### Debug

- **Debug Adapter Protocol (DAP) support** -- launch and attach debug sessions
- **Breakpoints** -- line breakpoints, conditional breakpoints, and logpoints
- **Debug toolbar** -- continue, pause, stop, and restart work via DebugManager state transitions; step over, step into, and step out are scaffolded (stub)
- **Variables and watch** -- inspect variables and add watch expressions
- **Debug console** -- evaluate expressions during debug sessions

#### Customization

- **Theme system** -- dark and light themes with VS Code-compatible color theme support
- **Icon themes** -- file icon themes loaded from extensions
- **Keybinding system** -- VS Code-compatible keybinding format with extension contributions
- **Settings system** -- user and workspace scopes, JSON-based configuration with defaults
- **Snippet support** -- VS Code snippet format for custom code templates

#### Platform

- **Cross-platform support** for macOS (Intel and Apple Silicon), Linux (x86_64 and aarch64), and Windows (x86_64)
- **Native installers** -- `.dmg` (macOS), `.deb` and `.AppImage` (Linux), `.msi` and `.exe` (Windows)
- **Single portable binary** option via Tauri build system
- **Install size under 30 MB** across all platforms

### Known Issues

- Some VS Code extensions may not activate due to incomplete API coverage (~75%). Extensions relying on unimplemented APIs will fail gracefully with an error in the extension host log.
- Remote development features (SSH, WSL, containers) are not yet available. These are planned for a future release.
- Notebook / Jupyter support: the extension host API (`vscode.notebooks`) is implemented, but frontend rendering is incomplete.
- Replace in files is partially implemented -- single-file replace works, but bulk replace across files is in progress.
- The Settings UI is JSON-only; a visual settings editor is in development.
- The keybinding editor does not yet support visual editing; keybindings must be configured via `keybindings.json`.
- Some debugger adapters may not auto-discover correctly; manual `launch.json` configuration may be required.
- Auto-update is not yet available; updates require manual download and installation.
- Debug stepping commands (`step_over`, `step_into`, `step_out`) are stubbed -- they validate the session but do not communicate with the debug adapter yet.
- Git pull only supports fast-forward merges; diverged histories require manual resolution via the terminal.

### Platform-Specific Notes

- **macOS** -- Universal binary supports both Intel and Apple Silicon. Requires macOS 12 (Monterey) or later for WebKit compatibility.
- **Linux** -- Requires WebKitGTK 4.1+. Tested on Ubuntu 22.04+, Fedora 38+, and Arch Linux. Wayland and X11 are both supported.
- **Windows** -- Requires WebView2 runtime (automatically installed by the `.msi` installer). Tested on Windows 10 21H2+ and Windows 11.

---

## What's Next

See the [Roadmap](roadmap.md) for the full development plan. Key priorities for the next release (v0.2.0-alpha):

- Replace in files (bulk operations across workspace)
- Search filters (include/exclude patterns, file type filters)
- Visual keybinding editor with conflict detection
- Settings UI (searchable, visual settings editor beyond JSON editing)
- Tree-sitter integration (completing the stub implementation)
- Full debug stepping via DAP adapter communication

---

## Contributors

DSCode is built by a growing community of contributors. Thank you to everyone who has contributed code, reported bugs, tested pre-release builds, and provided feedback.

- See the full list of contributors on [GitHub](https://github.com/dipankar/dscode/graphs/contributors)
- Want to contribute? Check out the [Contributing Guide](contributing/building-from-source.md)

---

<!-- Link definitions -->
[Unreleased]: https://github.com/dipankar/dscode/compare/v0.1.0-alpha...HEAD
[0.1.0-alpha]: https://github.com/dipankar/dscode/releases/tag/v0.1.0-alpha
