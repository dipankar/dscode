# dscode

[![PyPI](https://img.shields.io/pypi/v/dscode.svg)](https://pypi.org/project/dscode/)
[![CI](https://github.com/dipankar/dscode/actions/workflows/ci.yml/badge.svg)](https://github.com/dipankar/dscode/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Install [DSCode](https://github.com/dipankar/dscode) — a fast, hackable Visual Studio Code alternative built with Rust and Tauri — directly from PyPI.

## Install

```bash
pip install dscode
```

Requires Python 3.9 or later.

## Usage

After installation, launch DSCode from your terminal:

```bash
dscode
```

The package lazily downloads the correct platform-native binary on first run:

- **macOS** (Apple Silicon / Intel): `.app` bundle
- **Linux** (x86_64 / aarch64): `.AppImage`
- **Windows** (x86_64): portable binary

## How it works

1. `pip install dscode` installs a lightweight Python wrapper.
2. On first `dscode` invocation, the wrapper queries GitHub Releases for the latest version.
3. It downloads the platform-specific asset and extracts it into a local cache directory.
4. Subsequent launches use the cached binary.

## Requirements

- Python 3.9+
- macOS 11+, Linux (glibc 2.31+), or Windows 10+

## Uninstall

```bash
pip uninstall dscode
```

## License

MIT
