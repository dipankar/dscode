# dscode-core Python Bindings

[![PyPI](https://img.shields.io/pypi/v/dscode-core.svg)](https://pypi.org/project/dscode-core/)
[![CI](https://github.com/dipankar/dscode/actions/workflows/ci.yml/badge.svg)](https://github.com/dipankar/dscode/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Python bindings for [dscode-core](https://github.com/dipankar/dscode), the core Rust library behind [DSCode](https://github.com/dipankar/dscode) — a hackable VS Code alternative. Provides a high-performance rope-based text buffer and cross-platform directory resolution for building Python-native editors and developer tools.

## Installation

```bash
pip install dscode-core
```

Requires Python 3.9 or later.

## Features

- **`PyTextBuffer`** — High-performance rope-based text buffer with O(log n) insertions and deletions
- **`PyAppDirectories`** — Cross-platform directory resolution (config, data, cache, logs)
- **Zero-copy views** — Efficient text slicing without duplicating underlying data
- **Unicode aware** — Character, line, and byte indexing supported

## Usage

```python
from dscode_core_python import PyTextBuffer, PyAppDirectories

# Create a text buffer
buf = PyTextBuffer("Hello, world!")
print(buf.get_text())     # "Hello, world!"
print(buf.len_chars())    # 13

# Insert and delete
buf.insert(7, "beautiful ")
print(buf.get_text())     # "Hello, beautiful world!"

# Resolve platform directories
dirs = PyAppDirectories()
print(dirs.extensions_dir())
print(dirs.storage_dir())
print(dirs.logs_dir())
```

## Development

```bash
# Install maturin
pip install maturin

# Build and install in development mode
maturin develop

# Build wheel
maturin build --release

# Run tests
pytest
```

## Related Crates

| Crate | Purpose |
|-------|---------|
| [`dscode-core`](https://crates.io/crates/dscode-core) | Core Rust text buffer and configuration |
| [`dscode-lsp`](https://crates.io/crates/dscode-lsp) | Language Server Protocol client |
| [`dscode-session`](https://crates.io/crates/dscode-session) | Session manager aggregating all subsystems |

## License

MIT
