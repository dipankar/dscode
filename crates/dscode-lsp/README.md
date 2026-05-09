# dscode-lsp

[![docs.rs](https://img.shields.io/docsrs/dscode-lsp)](https://docs.rs/dscode-lsp)
[![crates.io](https://img.shields.io/crates/v/dscode-lsp.svg)](https://crates.io/crates/dscode-lsp)
[![CI](https://github.com/dipankar/dscode/actions/workflows/ci.yml/badge.svg)](https://github.com/dipankar/dscode/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Language Server Protocol (LSP) client, manager, and connection pool for [DSCode](https://github.com/dipankar/dscode) — a hackable VS Code alternative built in Rust.

## Install

```bash
cargo add dscode-lsp
```

## Overview

`dscode-lsp` provides a complete LSP client implementation with a state-machine-based lifecycle, a server registry for managing language servers, and a connection pool with configurable strategies.

## Features

- **`LspClient`** — State-machine-based LSP client that manages a language server process (spawn, initialize, communicate, shutdown)
- **`LspManager`** — Registry for language servers with lazy startup and default server registration
- **`LspServerPool`** — Connection pool with configurable strategies (`OnePerLanguage` or `MultiplePerLanguage`)
- **Strict lifecycle management** — Validated state transitions prevent invalid operations
- **Request timeout** — 30-second timeout on all LSP requests with proper cleanup
- **Process lifecycle** — Automatic cleanup on drop, crash detection, and state transitions

## Feature Flags

| Flag | Description |
|------|-------------|
| `default` | Standard LSP client with tokio runtime |

## API Surface

### `LspClient`

The primary LSP client handle. Manages a single language server subprocess.

| Method | Description |
|--------|-------------|
| `new(command, args)` | Spawn a new language server process |
| `initialize(root_url, capabilities)` | Send `Initialize` and wait for `Initialized` |
| `hover(url, line, character)` | Request hover information at a position |
| `goto_definition(url, line, character)` | Request definition locations |
| `goto_type_definition(url, line, character)` | Request type definition locations |
| `goto_implementation(url, line, character)` | Request implementation locations |
| `find_references(url, line, character)` | Request symbol references |
| `document_symbols(url)` | Request symbols in a document |
| `workspace_symbols(query)` | Request symbols across the workspace |
| `code_action(url, range, context)` | Request code actions |
| `completion(url, line, character)` | Request completions at a position |
| `formatting(url, options)` | Request document formatting |
| `did_open(url, language_id, version, text)` | Notify server of document open |
| `did_change(url, version, changes)` | Notify server of document changes |
| `did_save(url)` | Notify server of document save |
| `did_close(url)` | Notify server of document close |
| `shutdown()` | Send shutdown request |
| `state()` | Get current client state |

### `LspManager`

Registry for managing multiple language servers by language ID.

| Method | Description |
|--------|-------------|
| `new()` | Create a new manager |
| `register_server(language, command, args)` | Register a language server configuration |
| `start_server(language)` | Start the registered server for a language |
| `get_client(language)` | Get the active client for a language |
| `stop_server(language)` | Stop a running server |
| `stop_all()` | Stop all running servers |

### `LspServerPool`

Connection pool supporting multiple concurrent language servers.

| Method | Description |
|--------|-------------|
| `new(strategy)` | Create a pool with the given strategy |
| `register_server(language, command, args)` | Register a server configuration |
| `get_server(language, workspace)` | Get or create a server connection |
| `release_server(language, workspace)` | Return a server to the pool |
| `shutdown_all()` | Shutdown all pooled servers |

## Usage

```rust
use dscode_lsp::{LspManager, LspServerPool, LspServerStrategy};

// Simple manager usage
let manager = LspManager::new();
manager.register_server("rust", "rust-analyzer", vec![]).await;
manager.start_server("rust").await?;

// Connection pool with strategy
let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);
pool.register_server("rust".to_string(), "rust-analyzer".to_string(), vec![]).await;

let client = pool.get_server("rust", Some("file:///path/to/project")).await?;
let hover = client.hover(url, 0, 0).await?;
```

## State Machine

The `LspClient` follows a strict lifecycle:

```
Stopped → Starting → Initializing → Ready → ShuttingDown → Stopped
              ↘ Crashed ↗                         ↗
```

All state transitions are validated. Invalid transitions return an error.

## Related Crates

- `dscode-core` — Core types and text buffer.
- `dscode-session` — Session manager that uses `dscode-lsp`.
- `dscode-dap` — Debug Adapter Protocol client.
- `dscode-extension-host` — Extension host process management.

## Full API Docs

<https://docs.rs/dscode-lsp>

## License

MIT