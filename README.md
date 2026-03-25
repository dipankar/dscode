# DSCode

> A fast, Rust-based Visual Studio Code alternative with full extension compatibility

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/tauri-%2324C8DB.svg?style=flat&logo=tauri&logoColor=white)](https://tauri.app/)
[![Svelte](https://img.shields.io/badge/svelte-%23f1413d.svg?style=flat&logo=svelte&logoColor=white)](https://svelte.dev/)

## Overview

DSCode is a code editor built with a **Rust backend** (Tauri 2.1) and **Svelte 4 frontend**, designed as a drop-in replacement for Visual Studio Code. It maintains near-100% compatibility with VS Code's extension ecosystem while delivering superior performance through native Rust architecture.

### Key Features

- **Fast**: Sub-second startup times, Rust-powered backend
- **Monaco Editor**: Same editor as VS Code, pixel-perfect experience
- **Full Extension Compatibility**: Run existing VS Code extensions with Node.js runtime
- **Svelte UI**: Reactive, lightweight component system with custom theming
- **xterm.js Terminal**: Built-in terminal with PTY support
- **Memory Efficient**: ~60% less memory than Electron-based VS Code
- **Secure by Design**: Process isolation, sandboxed extensions, deny-by-default permissions
- **LSP Support**: Language Server Protocol integration for intelligent code editing
- **Git Integration**: Full SCM via libgit2
- **Cross-Platform**: Linux, macOS, and Windows support

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│           Tauri Window (WebKit/Chromium)                │
│  ┌───────────────────────────────────────────────────┐  │
│  │   Frontend (Svelte 4 + TypeScript)                │  │
│  │   - Monaco Editor                                  │  │
│  │   - Svelte Components (34 components)              │  │
│  │   - CSS Custom Properties (VS Code theme compat)   │  │
│  └────────────────┬──────────────────────────────────┘  │
│                   │ Tauri IPC (invoke/emit)             │
│  ┌────────────────▼──────────────────────────────────┐  │
│  │   Rust Backend                                    │  │
│  │   - SessionManager (central coordinator)           │  │
│  │   - File System + Path Validation                  │  │
│  │   - Git Integration (libgit2)                     │  │
│  │   - Terminal (PTY)                                │  │
│  │   - Resource Monitoring                           │  │
│  └────────────────┬──────────────────────────────────┘  │
└───────────────────┼──────────────────────────────────────┘
                    │ NNG IPC (nanomsg, serde_json)
         ┌──────────┴────────────┬─────────────┐
         │                       │             │
    ┌────▼──────┐          ┌────▼────┐   ┌───▼────┐
    │ Extension │          │   LSP   │   │ Debug  │
    │   Host    │          │ Client  │   │  DAP   │
    │ (Node.js) │          │         │   │        │
    └───────────┘          └─────────┘   └────────┘
```

## Core Technologies

### Frontend

- **UI Framework**: [Svelte 4](https://svelte.dev/) - Reactive component system
- **Editor**: [Monaco Editor](https://microsoft.github.io/monaco-editor/) - VS Code's editor
- **Terminal**: [xterm.js](https://xtermjs.org/) - Terminal emulator
- **Icons**: [lucide-svelte](https://lucide.dev/) - Icon library
- **Styling**: CSS Custom Properties with VS Code theme compatibility

### Backend (Rust)

- **Framework**: [Tauri 2.1](https://tauri.app/) - Rust desktop app framework
- **IPC**: Tauri Commands + [NNG](https://nng.nanomsg.org/) for extension host/LSP communication
- **Serialization**: [serde_json](https://github.com/serde-rs/json) for all IPC
- **Text Processing**: [ropey](https://github.com/cessen/ropey) - Rope-based text storage
- **Syntax**: [tree-sitter](https://tree-sitter.github.io/) - Incremental parsing
- **LSP**: [tower-lsp](https://github.com/ebkalderon/tower-lsp) - Language Server Protocol
- **Git**: [git2](https://github.com/rust-lang/git2-rs) - libgit2 bindings
- **Terminal**: [portable-pty](https://github.com/wez/wezterm/tree/main/pty) - PTY support
- **Search**: ripgrep-based (ignore + grep crates)

### Extension Runtime

- **Runtime**: Full **Node.js** process (separate from Tauri)
- **Communication**: NNG REQ/REP IPC with worker threads
- **API**: vscode.\* API implementation
- **Security**: Deny-by-default sandbox (macOS: sandbox-exec, Linux: bwrap)
- **Secrets**: OS keyring integration via SecretStorage API

## Project Structure

```
dscode/
├── src-tauri/                     # Rust backend
│   ├── src/
│   │   ├── main.rs                # Tauri app entry point
│   │   ├── bootstrap.rs           # App initialization
│   │   ├── commands/              # Tauri command handlers
│   │   │   ├── file_ops.rs        # File operations
│   │   │   ├── git_ops.rs         # Git commands
│   │   │   ├── terminal_ops.rs    # Terminal commands
│   │   │   └── debug_ops.rs       # Debug commands
│   │   ├── session/              # Session management
│   │   │   ├── mod.rs             # SessionManager (central coordinator)
│   │   │   ├── ipc.rs             # Extension host IPC handler
│   │   │   ├── extensions.rs      # Extension lifecycle
│   │   │   └── workspace.rs       # Workspace management
│   │   ├── extension_host/       # Extension host management
│   │   │   ├── manager.rs        # Process lifecycle + crash recovery
│   │   │   ├── nng_ipc.rs        # NNG IPC with spawn_blocking
│   │   │   ├── nng_manager.rs    # NNG connection manager
│   │   │   ├── sandbox.rs        # Multi-platform sandboxing
│   │   │   ├── path_validator.rs # Path validation + traversal prevention
│   │   │   └── secrets.rs        # SecretStorage with keyring
│   │   ├── lsp/                  # Language Server Protocol
│   │   │   ├── client.rs         # LSP client (tokio async)
│   │   │   ├── manager.rs       # LSP manager
│   │   │   └── pool.rs          # LSP connection pool
│   │   ├── core/                 # Core editor logic
│   │   ├── debug/                # Debug Adapter Protocol
│   │   ├── terminal/             # Terminal manager
│   │   ├── git/                  # Git integration
│   │   ├── config/               # Configuration management
│   │   ├── monitoring/           # Resource monitoring
│   │   └── watcher.rs            # File watching
│   └── Cargo.toml
├── src/                           # Frontend (Svelte/TypeScript)
│   ├── main.ts                    # App entry, lazy component loading
│   ├── App.svelte                 # Root component + error boundary
│   ├── app.css                    # Global styles + CSS custom properties
│   ├── components/                # Svelte UI components
│   │   ├── ActivityBar.svelte
│   │   ├── EditorArea.svelte
│   │   ├── Sidebar.svelte
│   │   ├── CommandPalette.svelte
│   │   ├── QuickOpen.svelte
│   │   ├── DiffViewer.svelte
│   │   ├── GitView.svelte
│   │   ├── SearchView.svelte
│   │   ├── StatusBar.svelte
│   │   └── ...
│   ├── stores/                    # Svelte stores (state management)
│   │   ├── editor.ts
│   │   ├── git.ts
│   │   ├── windowPrompt.ts
│   │   ├── appError.ts
│   │   └── ...
│   └── lib/                       # Shared logic
│       ├── app-shell/            # App shell controller
│       ├── command-dispatcher.ts  # Command registry
│       ├── contracts/            # TypeScript interfaces
│       └── wasm-tokenizer.ts     # WASM syntax highlighting
├── extension-host/                # Node.js extension runtime
│   ├── src/
│   │   ├── main.ts               # Extension host entry point
│   │   ├── nng-ipc.ts            # NNG IPC with worker thread
│   │   ├── bridge.ts             # Request/response handlers
│   │   ├── api/
│   │   │   ├── vscode.ts         # vscode.* API implementation
│   │   │   └── languages.ts      # Language provider registry
│   │   └── extensions/
│   │       └── manager.ts        # Extension lifecycle management
│   └── package.json
└── docs/                          # Documentation
```

## Getting Started

### Prerequisites

- Rust 1.75+ (with cargo)
- Node.js 18+ and npm
- Platform-specific dependencies:
  - **Linux**: `libwebkit2gtk-4.0-dev`, `libssl-dev`, `libgtk-3-dev`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Microsoft Visual C++ Build Tools

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/dscode.git
cd dscode

# Install frontend dependencies
npm install

# Development mode (hot reload)
npm run tauri:dev

# Build for production
npm run tauri:build
```

### Running

```bash
# Development
npm run tauri:dev

# Production build
npm run tauri:build

# Then run the binary:
# macOS: src-tauri/target/release/bundle/macos/DSCode.app
# Linux: src-tauri/target/release/bundle/deb/dscode_*_amd64.deb
# Windows: src-tauri/target/release/bundle/msi/DSCode_*_x64.msi
```

## Extension Compatibility

### Supported Extension Types

| Type                       | Compatibility | Notes                           |
| -------------------------- | ------------- | ------------------------------- |
| Pure JavaScript/TypeScript | Full          | ESLint, Prettier, GitLens, etc. |
| Native Node Modules        | Full          | Python extension, C/C++ tools   |
| Language Providers         | Full          | 18+ provider types via IPC      |
| Webview Extensions         | Partial       | Tauri webview API               |
| Electron API Users         | Partial       | Shim layer for common APIs      |
| Proprietary (Microsoft)    | Reimplement   | Copilot, IntelliCode            |

### Language Provider Support

The extension host supports full two-way IPC for: Hover, Completion, Diagnostics, SignatureHelp, Rename, CodeLens, CodeActions, Formatting, DocumentHighlights, FoldingRanges, SemanticTokens, DocumentSymbols, WorkspaceSymbols, Definitions, References, DocumentLinks, ColorPresentations, and InlineCompletions.

## Security

- **Deny-by-default sandbox**: Extensions start with no permissions
- **Path validation**: All filesystem access validated against workspace allowlist
- **Zip Slip prevention**: VSIX extraction validates paths
- **OS keyring**: Extension secrets stored via OS-native keyring
- **Platform sandboxes**: macOS (sandbox-exec), Linux (bubblewrap), planned Windows (Job Objects)
- **Crash recovery**: Extension host restarts with exponential backoff (max 3 attempts)

## Documentation

- [Architecture Overview](docs/architecture/overview.md)
- [IPC Design](docs/architecture/ipc-design.md)
- [Extension System](docs/architecture/extension-system.md)
- [UI System](docs/architecture/ui-system.md)
- [LSP Integration](docs/architecture/lsp-integration.md)

## Contributing

```bash
# Install dependencies
npm install

# Run type checks
npm run check          # Svelte component type check
cd src-tauri && cargo check  # Rust type check

# Run linter
npm run lint

# Development with hot reload
npm run tauri:dev
```

## License

[MIT](LICENSE)
