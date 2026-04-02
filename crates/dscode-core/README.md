# dscode-core

[![docs.rs](https://img.shields.io/docsrs/dscode-core)](https://docs.rs/dscode-core)
[![crates.io](https://img.shields.io/crates/v/dscode-core.svg)](https://crates.io/crates/dscode-core)
[![CI](https://github.com/dipankar/dscode/actions/workflows/ci.yml/badge.svg)](https://github.com/dipankar/dscode/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Core types, text buffer, and configuration for DSCode.

## Install

```bash
cargo add dscode-core
```

## Usage

### TextBuffer

```rust
use dscode_core::TextBuffer;

let buffer = TextBuffer::new("Hello, world!");
assert_eq!(buffer.len_chars(), 13);
assert_eq!(buffer.get_text(), "Hello, world!");
```

### AppDirectories

```rust
use dscode_core::AppDirectories;

let dirs = AppDirectories::resolve()?;
println!("Extensions: {:?}", dirs.extensions_dir);
println!("Storage: {:?}", dirs.storage_dir);
println!("Logs: {:?}", dirs.logs_dir);
```

### Configuration

```rust
use dscode_core::AppDirectories;

let dirs = AppDirectories::resolve()?;
// User configuration file support at ~/.dscode/config.json
// Environment variable overrides (DSCODE_EXTENSIONS_DIR)
// Tilde (~) expansion in paths
```

See the `examples/` directory for more complete samples.

## Key Types

| Type | Description |
|------|-------------|
| `TextBuffer` | Rope-based text storage for efficient editing operations on large files |
| `AppDirectories` | Platform-aware directory resolution for extensions, storage, and logs |
| `CoreError` | Unified error type for core operations |

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `python` | off | Enables Python bindings via PyO3 |

## State Machine

`TextBuffer` uses the `ropey` rope data structure internally for O(log n) insertions, deletions, and slicing on large files.

## Full API Docs

<https://docs.rs/dscode-core>

## License

MIT
