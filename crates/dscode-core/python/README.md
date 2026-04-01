# dscode-core Python Bindings

Python bindings for [dscode-core](https://github.com/dipankarsarkar/dscode), providing rope-based text buffer and platform-aware directory resolution.

## Installation

```bash
pip install dscode-core
```

## Usage

```python
from dscode_core_python import PyTextBuffer, PyAppDirectories

# Create a text buffer
buf = PyTextBuffer("Hello, world!")
print(buf.get_text())     # "Hello, world!"
print(buf.len_chars())   # 13

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
```

## License

MIT