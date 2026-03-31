# dscode-core

Core types, text buffer, and configuration for DSCode.

## Overview

`dscode-core` provides fundamental building blocks used across all DSCode components:

- **`TextBuffer`** — Rope-based text storage for efficient editing operations on large files
- **`AppDirectories`** — Platform-aware directory resolution for extensions, storage, and logs

## Features

- Fast text operations via `ropey` rope data structure
- Platform-appropriate directory resolution (macOS, Linux, Windows)
- Environment variable overrides (`DSCODE_EXTENSIONS_DIR`)
- User configuration file support (`~/.dscode/config.json`)
- Tilde (`~`) expansion in paths

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

## License

MIT