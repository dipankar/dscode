# DSCode

> **The code editor you can take apart.**
> A familiar VS Code experience on top of a Rust core that ships as libraries — swap the editor, replace the terminal, or embed the whole session in your own app.

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

---

## Built for two audiences

| If you want to… | DSCode gives you |
|---|---|
| **Edit code, fast** | A VS Code-like desktop app with sub-second startup and ~60% less memory |
| **Keep your existing extensions** | A full Node.js extension host with the `vscode.*` API, sandboxed by default |
| **Build your own tool on top** | Six Rust crates — text buffer, LSP, DAP, terminal, extension host, session — that compose any way you like |
| **Run an editor headlessly** | The same session, workspace, and extension layer with no UI attached |

---

## For users: a familiar editor that doesn't cost you a gigabyte of RAM

- **Sub-second startup.** Native Rust binary, no Electron boot.
- **~60% less memory** than VS Code on the same workspace.
- **Your extensions still work.** ESLint, Prettier, GitLens, Python — they run unchanged on the bundled Node.js host.
- **Secure by default.** Every extension starts with zero permissions, isolated by an OS-native sandbox (macOS `sandbox-exec`, Linux `bubblewrap`, Windows Job Objects).
- **Familiar UI.** Monaco editor, xterm.js terminal, VS Code theme compatibility, ARIA-correct components.
- **Cross-platform.** macOS, Linux, and Windows.

> Download a prebuilt release from the [**Releases**](https://github.com/dipankar/dscode/releases) page, or build from source — see [Quickstart](#quickstart).

---

## For developers: an IDE that ships as a kit, not a black box

Every subsystem is a separate crate with a clean API. `cargo add` what you need, ignore the rest. The desktop app itself is just one composition of these crates — yours can be different.

```toml
[dependencies]
dscode-core           = "0.3"   # rope text buffer, app directories
dscode-lsp            = "0.3"   # LSP client, manager, pool
dscode-dap            = "0.3"   # Debug Adapter Protocol
dscode-terminal       = "0.3"   # PTY lifecycle, event sender trait
dscode-extension-host = "0.3"   # sandbox, IPC, permissions, rate limiter
dscode-session        = "0.3"   # workspace, configuration, lifecycle
```

Beyond Rust: `npm i @dscode/monaco-wasm`, `pip install dscode-core`.

```rust
use dscode_core::{TextBuffer, AppDirectories};

let buffer = TextBuffer::new("Hello, world!");
println!("{} chars", buffer.len_chars());

let dirs = AppDirectories::resolve(None);
println!("Config dir: {:?}", dirs.config_dir);
```

```rust
use dscode_lsp::{LspManager, LspServerPool, LspServerStrategy};

let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);
let manager = LspManager::new(pool);
manager.register_server("rust", "rust-analyzer", vec!["--stdio".into()]);
```

Each crate has optional Tauri integration via a feature flag, so the same crate works in a desktop app or a headless CI runner:

```toml
dscode-terminal = { version = "0.3", features = ["tauri"] }
```

See the [**Library Guide**](docs/library-guide.md) for the full tour.

### The crates

| Crate | What it gives you | Install |
|---|---|---|
| [`dscode-core`](https://crates.io/crates/dscode-core) | Rope-based `TextBuffer`, `AppDirectories`, `CoreError` | `cargo add dscode-core` |
| [`dscode-lsp`](https://crates.io/crates/dscode-lsp) | LSP client, manager, connection pool | `cargo add dscode-lsp` |
| [`dscode-dap`](https://crates.io/crates/dscode-dap) | Debug Adapter Protocol client, manager, pool | `cargo add dscode-dap` |
| [`dscode-extension-host`](https://crates.io/crates/dscode-extension-host) | Host manager, IPC, sandbox, permissions, rate limiter, secrets | `cargo add dscode-extension-host` |
| [`dscode-terminal`](https://crates.io/crates/dscode-terminal) | Terminal manager, PTY lifecycle, `TerminalEventSender` trait | `cargo add dscode-terminal` |
| [`dscode-session`](https://crates.io/crates/dscode-session) | Session manager, extension lifecycle, workspace, configuration | `cargo add dscode-session` |

---

## For hackers: replace anything

| Subsystem | Default | Swap for |
|---|---|---|
| **Editor** | Monaco (via `monaco-wasm`) | A WebGL renderer, a Vim core, or a plain textarea |
| **Extension host** | Node.js + NNG IPC | A Wasm runtime, a Lua engine, or nothing at all |
| **Terminal** | xterm.js + `portable-pty` | Alacritty, your own ANSI parser, or a remote SSH session |
| **UI framework** | Svelte 4 | React, Solid, Vue, or raw DOM |
| **Session state** | `dscode-session` with Tauri | The same crate without Tauri, for headless CI |
| **Text storage** | `ropey` (`dscode-core::TextBuffer`) | A gap buffer, a piece table, or a CRDT |
| **LSP client** | `dscode-lsp::LspManager` | Direct `LspClient` use, or a custom protocol handler |

### Recipes

- **Headless CI runner** — use `dscode-session` without the `tauri` feature to scan workspaces, run extensions, and emit diagnostics as JSON.
- **Custom terminal UI** — implement `TerminalEventSender` from `dscode-terminal` and forward PTY output to a websocket, a log file, or your own renderer.
- **Embed in your own Tauri app** — pull in `dscode-session` with the `tauri` feature and you have the full IDE session inside your application.
- **Write a different LSP client** — `dscode-lsp` exposes `LspClient` directly if you'd rather build a bespoke manager.
- **Build a live theme editor** — combine `dscode-core::AppDirectories` with the CSS-custom-property theme layer to preview themes in real time.

Code examples for each are in [docs/library-guide.md](docs/library-guide.md).

---

## Design principles

1. **Everything is a library.** The application is a thin composition layer; the real value is in the crates.
2. **APIs over implementations.** We standardise on interfaces (`TerminalEventSender`, `LspClient`, `TextBuffer`) so you can swap implementations without touching the rest of the system.
3. **VS Code compatibility is a feature, not a constraint.** We support VS Code extensions because they're useful — but their limitations don't dictate our architecture.
4. **Security by default.** Extensions can't read your SSH keys by accident. Deny-by-default sandboxing is non-negotiable.
5. **Performance is table stakes.** Sub-second startup and low memory aren't traded for convenience.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│             Tauri Window (WebKit/Chromium)                   │
│  ┌────────────────────────────────────────────────────────┐  │
│  │   Frontend (Svelte 4 + TypeScript)                     │  │
│  │   - Monaco Editor                                      │  │
│  │   - Svelte Components (38 components)                  │  │
│  │   - CSS Custom Properties (VS Code theme compat)       │  │
│  │   - ARIA accessibility, focus traps                    │  │
│  └──────────────────────┬─────────────────────────────────┘  │
│                         │ Tauri IPC (invoke/emit)             │
│  ┌──────────────────────▼─────────────────────────────────┐  │
│  │   Rust Binary (src-tauri)                              │  │
│  │   - SessionManager (central coordinator)               │  │
│  │   - Tauri Command Handlers (45 modules)                │  │
│  │   - Bootstrap, Logging, Monitoring, File Watcher       │  │
│  └──────────┬─────────────────────────────────────────────┘  │
└─────────────┼────────────────────────────────────────────────┘
              │ depends on
┌─────────────▼────────────────────────────────────────────────┐
│  Cargo Workspace Library Crates                              │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐  │
│  │ dscode-core  │ │  dscode-lsp  │ │    dscode-dap        │  │
│  │ TextBuffer   │ │  LspClient   │ │    DebugAdapter      │  │
│  │ AppDirs      │ │  LspManager  │ │    DebugManager      │  │
│  │ CoreError    │ │  LspPool     │ │    DapPool           │  │
│  └──────────────┘ └──────────────┘ └──────────────────────┘  │
│  ┌──────────────────────┐ ┌──────────────┐                   │
│  │ dscode-extension-host│ │ dscode-term  │                   │
│  │ HostManager          │ │ TermManager  │                   │
│  │ IPC + Sandbox        │ │ PTY          │                   │
│  │ Permissions + Rate   │ │ EventSender  │                   │
│  │ Secrets + Validator  │ │ TermError    │                   │
│  └──────────────────────┘ └──────────────┘                   │
│  ┌──────────────────────────────────────────────────────────┐│
│  │ dscode-session                                           ││
│  │ ConfigStore, ExtensionLifecycle, Workspace, Documents,   ││
│  │ Contributions, EventEmitter, SessionState                ││
│  └──────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────┘
              │ NNG IPC (nanomsg, serde_json)
   ┌──────────┴────────────┬─────────────┐
   │                       │             │
 ┌─▼─────────┐     ┌──────▼────┐  ┌────▼────┐
 │ Extension │     │    LSP    │  │  Debug  │
 │   Host    │     │  Server   │  │   DAP   │
 │ (Node.js) │     │           │  │  Server │
 └───────────┘     └───────────┘  └─────────┘
```

### Stack at a glance

**Frontend** — Svelte 4, Monaco Editor, xterm.js, lucide-svelte, CSS custom properties for VS Code theme compatibility.

**Backend (Rust)** — Tauri 2.1, Tauri Commands + [NNG](https://nng.nanomsg.org/) for out-of-process IPC, [ropey](https://github.com/cessen/ropey) text buffers, [tree-sitter](https://tree-sitter.github.io/) parsing, [tower-lsp](https://github.com/ebkalderon/tower-lsp), [git2](https://github.com/rust-lang/git2-rs), [portable-pty](https://github.com/wez/wezterm/tree/main/pty), [tracing](https://github.com/tokio-rs/tracing), [thiserror](https://github.com/dtolnay/thiserror), ripgrep-based search.

**Extension runtime** — a real Node.js process, NNG REQ/REP IPC with worker threads, full `vscode.*` API surface, OS-keyring-backed `SecretStorage`, deny-by-default sandbox.

---

## Quickstart

### Prerequisites

- Rust 1.75+ with `cargo`
- Node.js 20+ and `npm`
- Platform-specific dependencies:
  - **Linux:** `libwebkit2gtk-4.1-dev`, `libssl-dev`, `libgtk-3-dev`, `librsvg2-dev`, `patchelf`
  - **macOS:** Xcode Command Line Tools
  - **Windows:** Microsoft Visual C++ Build Tools

### Build and run

```bash
git clone https://github.com/dipankar/dscode.git
cd dscode
npm install

npm run tauri:dev        # development with hot reload
npm run tauri:build      # production bundle
```

Production artefacts land in:

- **macOS:** `src-tauri/target/release/bundle/macos/DSCode.app`
- **Linux:** `src-tauri/target/release/bundle/deb/dscode_*_amd64.deb`
- **Windows:** `src-tauri/target/release/bundle/msi/DSCode_*_x64.msi`

### Test and lint

```bash
cargo test --workspace                       # all crate tests
cargo test -p dscode-core                    # one crate
cargo clippy --workspace -- -D warnings      # lint
```

---

## Project layout

```
dscode/
├── Cargo.toml                # workspace root
├── crates/
│   ├── dscode-core/          # TextBuffer, AppDirectories
│   ├── dscode-lsp/           # LSP client, manager, pool
│   ├── dscode-dap/           # Debug Adapter Protocol
│   ├── dscode-extension-host/# Extension host management
│   ├── dscode-terminal/      # Terminal manager
│   └── dscode-session/       # Session management
├── src-tauri/                # Tauri binary (the default app)
│   └── src/
│       ├── main.rs
│       ├── bootstrap.rs
│       ├── session/          # SessionManager + IPC
│       └── commands/         # 45 Tauri command modules
├── monaco-wasm/              # Monaco WASM bindings
├── extension-host/           # Node.js extension runtime
├── src/                      # Svelte frontend (38 components)
├── packages/                 # npm and pypi bindings
└── docs/                     # Documentation
```

---

## Extension compatibility

DSCode's extension host isn't a compatibility shim — it's a **full Node.js runtime** speaking the same IPC protocol as VS Code. Extensions are first-class citizens, but they're also sandboxed and observable.

| Extension type | Compatibility | Notes |
|---|---|---|
| Pure JavaScript / TypeScript | Full | ESLint, Prettier, GitLens, etc. |
| Native Node modules | Full | Python extension, C/C++ tools |
| Language providers | Full | 18+ provider types via IPC |
| Webview extensions | Partial | Tauri webview API |
| Electron API users | Partial | Shim layer for common APIs |
| Proprietary (Microsoft) | Reimplement | Copilot, IntelliCode |

**Language providers supported with full two-way IPC:** Hover, Completion, Diagnostics, SignatureHelp, Rename, CodeLens, CodeActions, Formatting, DocumentHighlights, FoldingRanges, SemanticTokens, DocumentSymbols, WorkspaceSymbols, Definitions, References, DocumentLinks, ColorPresentations, InlineCompletions.

**Extras DSCode adds on top of the VS Code API:**

- **Sandbox declarations** — extensions declare filesystem, network, and shell permissions in `package.json`. Denied by default.
- **Rate limiting** — built-in per-extension quota.
- **OS keyring access** — `vscode.SecretStorage` is backed by the OS-native keyring.
- **Hot reload** — extensions reload without restarting the editor.

---

## Security

- **Deny-by-default sandbox** — extensions start with no permissions.
- **Platform sandboxes** — macOS `sandbox-exec`, Linux `bubblewrap`, Windows Job Objects.
- **Path validation** — all filesystem access is checked against the workspace allowlist.
- **VSIX hardening** — Zip-Slip prevention, SHA256 hash check, `package.json` cross-validation.
- **OS keyring** — extension secrets are stored via the OS-native keyring.
- **Crash recovery** — extension host restarts with exponential backoff (max 3 attempts).
- **Stale IPC cleanup** — orphaned socket files and pending requests are reaped automatically.
- **Node.js verification** — bundled Node binary is hash-verified on startup.

See [SECURITY.md](SECURITY.md) for the disclosure process.

---

## Documentation

- [Architecture Overview](docs/architecture/overview.md)
- [IPC Design](docs/architecture/ipc-design.md)
- [Extension System](docs/architecture/extension-system.md)
- [UI System](docs/architecture/ui-system.md)
- [LSP Integration](docs/architecture/lsp-integration.md)
- [Library Guide](docs/library-guide.md)
- [Roadmap](docs/roadmap.md)

---

## Contributing

DSCode is a community project. Contributions are welcome — whether you're fixing a bug, adding a feature, or replacing an entire subsystem.

**Adding a new crate**

1. Create `crates/my-crate/` with a `Cargo.toml` that inherits `[workspace.package]`.
2. Add `"crates/my-crate"` to `[workspace].members` in the root `Cargo.toml`.
3. Write a `README.md` with badges, install instructions, and a usage example.
4. Add an `examples/` directory with at least one runnable example.
5. Open a PR. CI checks formatting, clippy, tests, docs, and `cargo publish --dry-run`.

**Replacing a subsystem**

1. Open an issue describing your use case.
2. We'll help you identify the minimal API surface to implement.
3. Submit a PR. Hackability beats backwards compatibility in pre-1.0 releases.

Full guidelines in [CONTRIBUTING.md](CONTRIBUTING.md).

---

## License

[MIT](LICENSE)
