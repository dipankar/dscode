# DSCode

> A fully hackable Visual Studio Code alternative. Built in Rust. Every subsystem is a library you can replace.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/dipankar/dscode/actions/workflows/ci.yml/badge.svg)](https://github.com/dipankar/dscode/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/tauri-%2324C8DB.svg?style=flat&logo=tauri&logoColor=white)](https://tauri.app/)
[![Svelte](https://img.shields.io/badge/svelte-%23f1413d.svg?style=flat&logo=svelte&logoColor=white)](https://svelte.dev/)

**Ecosystem**

[![crates.io: dscode-core](https://img.shields.io/crates/v/dscode-core?label=dscode-core)](https://crates.io/crates/dscode-core)
[![crates.io: dscode-lsp](https://img.shields.io/crates/v/dscode-lsp?label=dscode-lsp)](https://crates.io/crates/dscode-lsp)
[![crates.io: dscode-dap](https://img.shields.io/crates/v/dscode-dap?label=dscode-dap)](https://crates.io/crates/dscode-dap)
[![crates.io: dscode-extension-host](https://img.shields.io/crates/v/dscode-extension-host?label=dscode-extension-host)](https://crates.io/crates/dscode-extension-host)
[![crates.io: dscode-terminal](https://img.shields.io/crates/v/dscode-terminal?label=dscode-terminal)](https://crates.io/crates/dscode-terminal)
[![crates.io: dscode-session](https://img.shields.io/crates/v/dscode-session?label=dscode-session)](https://crates.io/crates/dscode-session)
[![npm: @dscode/monaco-wasm](https://img.shields.io/npm/v/@dscode/monaco-wasm?label=monaco-wasm)](https://www.npmjs.com/package/@dscode/monaco-wasm)
[![npm: dscode-extension-host](https://img.shields.io/npm/v/dscode-extension-host?label=extension-host)](https://www.npmjs.com/package/dscode-extension-host)
[![PyPI: dscode-core](https://img.shields.io/pypi/v/dscode-core?label=dscode-core%20python)](https://pypi.org/project/dscode-core/)

## Overview

DSCode is a **hackable code editor** built with a Rust backend (Tauri) and Svelte frontend. Unlike VS Code, which is a monolithic Electron app, DSCode is a **loosely coupled system of libraries** — each designed to be used independently, replaced, or embedded in your own projects.

The default application gives you a familiar VS Code-like experience out of the box. But the real goal is that **nothing is hardcoded**: swap the editor, replace the terminal, write your own extension host, or embed the entire session layer in a headless CI runner.

### Why DSCode?

- **Hackable by Design**: Every subsystem is a separate crate with a clean API. Don't like Monaco? Plug in your own renderer. Don't need the UI? Use `dscode-session` headlessly.
- **VS Code Extension Compatible**: Run existing VS Code extensions via a Node.js-based extension host with full `vscode.*` API support.
- **Rust-Native Performance**: Sub-second startup, ~60% less memory than Electron, rope-based text buffers, and native file watching.
- **Secure by Default**: Extensions run in a deny-by-default sandbox with OS-native isolation (macOS sandbox-exec, Linux bubblewrap, Windows Job Objects).
- **Built as Libraries First**: The app is a thin shell around crates you can `cargo add` into your own projects.

### Key Features

- **Fast**: Sub-second startup times, Rust-powered backend
- **Monaco Editor**: Same editor engine as VS Code, pixel-perfect experience
- **Full Extension Compatibility**: Run existing VS Code extensions with Node.js runtime
- **Svelte UI**: Reactive, lightweight component system with custom theming
- **xterm.js Terminal**: Built-in terminal with PTY support
- **Memory Efficient**: ~60% less memory than Electron-based VS Code
- **Secure by Design**: Process isolation, sandboxed extensions, deny-by-default permissions
- **LSP Support**: Language Server Protocol integration for intelligent code editing
- **Git Integration**: Full SCM via libgit2
- **Cross-Platform**: Linux, macOS, and Windows support
- **Reusable Libraries**: Core components available as Cargo crates for use in other Rust projects

## Library Crates

DSCode is not a single binary — it is a **workspace of libraries** that happen to ship with a default UI. Every crate is designed for independent use:

| Crate | Description | Install |
|-------|-------------|---------|
| `dscode-core` | TextBuffer (rope-based), AppDirectories, CoreError | `cargo add dscode-core` |
| `dscode-lsp` | LSP client, manager, connection pool | `cargo add dscode-lsp` |
| `dscode-dap` | Debug Adapter Protocol client, manager, pool | `cargo add dscode-dap` |
| `dscode-extension-host` | Extension host manager, IPC, sandbox, permissions, rate limiter, secrets | `cargo add dscode-extension-host` |
| `dscode-terminal` | Terminal manager, PTY lifecycle, TerminalEventSender trait | `cargo add dscode-terminal` |
| `dscode-session` | Session manager, extension lifecycle, workspace, configuration | `cargo add dscode-session` |

### Using as a Library

Import just what you need. No UI, no Tauri, no frontend required:

```rust
use dscode_core::{TextBuffer, AppDirectories};

fn main() {
    let buffer = TextBuffer::new("Hello, world!");
    println!("Buffer length: {} chars", buffer.len_chars());

    let app_dirs = AppDirectories::resolve(None);
    println!("Config dir: {:?}", app_dirs.config_dir);
}
```

```rust
use dscode_lsp::{LspManager, LspServerPool, LspServerStrategy};

fn main() {
    let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);
    let manager = LspManager::new(pool);
    manager.register_server("rust", "rust-analyzer", vec!["--stdio".to_string()]);
}
```

Each crate has optional Tauri integration via feature flags:
```toml
[dependencies]
dscode-terminal = { version = "0.2", features = ["tauri"] }
```

## Design Principles

1. **Everything is a library**: The application is a thin composition layer. The real value is in the crates.
2. **APIs over implementations**: We standardize on interfaces (`TerminalEventSender`, `LspClient`, `TextBuffer`) so you can swap implementations without touching the rest of the system.
3. **VS Code compatibility is a feature, not a constraint**: We support VS Code extensions because they are useful. We do not let their limitations dictate our architecture.
4. **Security by default**: Extensions should not be able to read your SSH keys by accident. Deny-by-default sandboxing is non-negotiable.
5. **Performance is table stakes**: Rust gives us sub-second startup and low memory usage. We do not trade this away for convenience.

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│             Tauri Window (WebKit/Chromium)                   │
│  ┌────────────────────────────────────────────────────────┐ │
│  │   Frontend (Svelte 4 + TypeScript)                     │ │
│  │   - Monaco Editor                                      │ │
│  │   - Svelte Components (38 components)                   │ │
│  │   - CSS Custom Properties (VS Code theme compat)        │ │
│  │   - ARIA accessibility, focus traps                     │ │
│  └──────────────────────┬─────────────────────────────────┘ │
│                          │ Tauri IPC (invoke/emit)            │
│  ┌───────────────────────▼─────────────────────────────────┐ │
│  │   Rust Binary (src-tauri)                               │ │
│  │   - SessionManager (central coordinator)                │ │
│  │   - Tauri Command Handlers (45 modules)                 │ │
│  │   - Bootstrap, Logging, Monitoring, File Watcher        │ │
│  └──────────┬──────────────────────────────────────────────┘ │
└─────────────┼────────────────────────────────────────────────┘
              │ depends on
┌─────────────▼────────────────────────────────────────────────┐
│  Cargo Workspace Library Crates                              │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐ │
│  │ dscode-core  │ │  dscode-lsp  │ │    dscode-dap        │ │
│  │ TextBuffer   │ │  LspClient   │ │    DebugAdapter      │ │
│  │ AppDirs      │ │  LspManager  │ │    DebugManager      │ │
│  │ CoreError    │ │  LspPool     │ │    DapPool           │ │
│  └──────────────┘ └──────────────┘ └──────────────────────┘ │
│  ┌──────────────────────┐ ┌──────────────┐                 │
│  │ dscode-extension-host │ │  dscode-term │                 │
│  │ HostManager          │ │  TermManager │                 │
│  │ IPC + Sandbox        │ │  PTY         │                 │
│  │ Permissions + Rate   │ │  EventSender │                 │
│  │ Secrets + Validator  │ │  TermError   │                 │
│  └──────────────────────┘ └──────────────┘                 │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │ dscode-session                                          │ │
│  │ ConfigStore, ExtensionLifecycle, Workspace, Documents,  │ │
│  │ Contributions, EventEmitter, SessionState                │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
              │ NNG IPC (nanomsg, serde_json)
   ┌──────────┴────────────┬─────────────┐
   │                       │             │
  ┌▼──────────┐     ┌─────▼────┐   ┌────▼───┐
  │ Extension │     │   LSP    │   │ Debug  │
  │   Host    │     │ Server  │   │  DAP    │
  │ (Node.js) │     │         │   │ Server │
  └───────────┘     └─────────┘   └────────┘
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
- **Logging**: [tracing](https://github.com/tokio-rs/tracing) - Structured logging
- **Errors**: [thiserror](https://github.com/dtolnay/thiserror) - Typed error enums
- **Search**: ripgrep-based (ignore + grep crates)

### Extension Runtime

- **Runtime**: Full **Node.js** process (separate from Tauri)
- **Communication**: NNG REQ/REP IPC with worker threads
- **API**: vscode.\* API implementation
- **Security**: Deny-by-default sandbox (macOS: sandbox-exec, Linux: bwrap, Windows: Job Objects)
- **Secrets**: OS keyring integration via SecretStorage API

## Project Structure

```
dscode/
├── Cargo.toml                     # Workspace root
├── crates/
│   ├── dscode-core/               # TextBuffer, AppDirectories
│   ├── dscode-lsp/                 # LSP client, manager, pool
│   ├── dscode-dap/                 # Debug Adapter Protocol
│   ├── dscode-extension-host/      # Extension host management
│   ├── dscode-terminal/            # Terminal manager
│   └── dscode-session/            # Session management
├── src-tauri/                     # Tauri binary crate
│   ├── src/
│   │   ├── main.rs                # App entry point
│   │   ├── bootstrap.rs           # App initialization
│   │   ├── session/               # SessionManager + IPC
│   │   ├── commands/              # 45 Tauri command modules
│   │   └── ...
│   └── Cargo.toml
├── monaco-wasm/                   # Monaco WASM bindings
├── extension-host/                # Node.js extension runtime
├── src/                           # Svelte frontend
│   ├── components/                # 38 Svelte components
│   ├── stores/                    # State management
│   └── lib/                       # Shared logic
└── docs/                          # Documentation
```

## Getting Started

### Prerequisites

- Rust 1.75+ (with cargo)
- Node.js 20+ and npm
- Platform-specific dependencies:
  - **Linux**: `libwebkit2gtk-4.1-dev`, `libssl-dev`, `libgtk-3-dev`, `librsvg2-dev`, `patchelf`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Microsoft Visual C++ Build Tools

### Building from Source

```bash
# Clone the repository
git clone https://github.com/dipankar/dscode.git
cd dscode

# Install frontend dependencies
npm install

# Development mode (hot reload)
npm run tauri:dev

# Build for production
npm run tauri:build
```

### Running Tests

```bash
# All workspace tests
cargo test --workspace

# Specific crate tests
cargo test -p dscode-core
cargo test -p dscode-lsp

# Clippy lint
cargo clippy --workspace -- -D warnings
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

DSCode's extension host is not a compatibility shim — it is a **full Node.js runtime** that speaks the same IPC protocol as VS Code. Extensions are first-class citizens, but they are also fully sandboxed and observable.

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

### Writing Extensions for DSCode

Extensions use the standard VS Code `package.json` + `vscode` API. DSCode adds a few extra capabilities:

- **Sandbox declarations**: Extensions declare permissions in `package.json` (filesystem, network, shell). Denied by default.
- **Rate limiting**: Built-in per-extension quota system.
- **OS keyring access**: Secure credential storage via `vscode.SecretStorage`.
- **Hot reload**: Extensions can be reloaded without restarting the editor.

## Security

- **Deny-by-default sandbox**: Extensions start with no permissions
- **Path validation**: All filesystem access validated against workspace allowlist
- **Zip Slip prevention**: VSIX extraction validates paths
- **VSIX manifest verification**: SHA256 hash check + package.json cross-validation
- **OS keyring**: Extension secrets stored via OS-native keyring
- **Platform sandboxes**: macOS (sandbox-exec), Linux (bubblewrap), Windows (Job Objects)
- **Crash recovery**: Extension host restarts with exponential backoff (max 3 attempts)
- **Stale IPC cleanup**: Orphaned socket files and pending requests cleaned up automatically
- **Node.js verification**: Binary hash verification on startup

## Documentation

- [Architecture Overview](docs/architecture/overview.md)
- [IPC Design](docs/architecture/ipc-design.md)
- [Extension System](docs/architecture/extension-system.md)
- [UI System](docs/architecture/ui-system.md)
- [LSP Integration](docs/architecture/lsp-integration.md)
- [Roadmap](docs/roadmap.md)

## Hackability

DSCode is designed to be disassembled and reassembled. Every major subsystem is a separate crate with a minimal public API. You are not just a user — you are a co-author.

### Replace Anything

| Subsystem | Default Implementation | Swap For |
|-----------|----------------------|----------|
| **Editor** | Monaco Editor (via `monaco-wasm`) | Your own WebGL renderer, Vim mode, or a lightweight textarea |
| **Extension Host** | Node.js + NNG IPC | A Wasm runtime, a Lua engine, or nothing at all |
| **Terminal** | xterm.js + portable-pty | Alacritty, your own ANSI parser, or a remote SSH session |
| **UI Framework** | Svelte 4 | React, Solid, Vue, or raw DOM |
| **Session State** | `dscode-session` (Tauri) | `dscode-session` without Tauri for headless CI |
| **Text Storage** | `ropey` (via `dscode-core::TextBuffer`) | Your own gap buffer, piece table, or CRDT |
| **LSP Client** | `dscode-lsp::LspManager` | Direct `LspClient` usage, or a custom protocol handler |

### Common Recipes

- **Headless CI runner**: Use `dscode-session` without the `tauri` feature to scan workspaces, run extensions, and export diagnostics as JSON.
- **Custom terminal UI**: Implement `TerminalEventSender` from `dscode-terminal` to forward PTY output to a web socket, log file, or custom UI.
- **Embed in your own Tauri app**: Import `dscode-session` with the `tauri` feature to get the full IDE session in your application.
- **Write a new LSP client**: `dscode-lsp` exposes `LspClient` directly if you want to build your own manager instead of using `LspManager`.
- **Build a VS Code theme editor**: Use `dscode-core::AppDirectories` and the CSS custom property system to preview themes in real time.

See [docs/library-guide.md](docs/library-guide.md) for concrete code examples.

## Contributing

DSCode is a community project. Contributions are welcome — whether you are fixing a bug, adding a feature, or replacing an entire subsystem.

### Adding a New Crate

1. Create `crates/my-crate/` with a `Cargo.toml` that inherits `[workspace.package]`.
2. Add `"crates/my-crate"` to the `[workspace].members` array in root `Cargo.toml`.
3. Write a `README.md` with badges, install instructions, and a usage example.
4. Add an `examples/` directory with at least one runnable example.
5. Open a PR. CI will check formatting, clippy, tests, docs, and `cargo publish --dry-run`.

### Replacing a Subsystem

If you want to replace Monaco, the extension host, or any other component:

1. Open an issue describing your use case.
2. We will help you identify the minimal API surface you need to implement.
3. Submit a PR. We prioritize hackability over backwards compatibility in pre-1.0 releases.

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines including workspace build instructions, code style, and PR process.

## License

[MIT](LICENSE)