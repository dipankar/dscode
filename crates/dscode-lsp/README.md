# dscode-lsp

[![docs.rs](https://img.shields.io/docsrs/dscode-lsp)](https://docs.rs/dscode-lsp)
[![crates.io](https://img.shields.io/crates/v/dscode-lsp.svg)](https://crates.io/crates/dscode-lsp)
[![CI](https://github.com/dipankar/dscode/actions/workflows/ci.yml/badge.svg)](https://github.com/dipankar/dscode/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Language Server Protocol client, manager, and connection pool for DSCode.

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

## Full API Docs

<https://docs.rs/dscode-lsp>

## License

MIT