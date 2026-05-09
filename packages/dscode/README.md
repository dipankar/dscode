# dscode

[![npm version](https://img.shields.io/npm/v/dscode.svg)](https://www.npmjs.com/package/dscode)
[![CI](https://github.com/dipankar/dscode/actions/workflows/ci.yml/badge.svg)](https://github.com/dipankar/dscode/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Install [DSCode](https://github.com/dipankar/dscode) — a fast, hackable Visual Studio Code alternative built with Rust and Tauri — directly from npm.

## Install

```bash
npm install -g dscode
```

## Usage

After installation, launch DSCode from your terminal:

```bash
dscode
```

The package downloads the correct platform-native binary automatically:

- **macOS** (Apple Silicon / Intel): `.app` bundle from `.dmg`
- **Linux** (x86_64 / aarch64): `.AppImage`
- **Windows** (x86_64): `.zip` / `.msi` / `.exe`

If you installed with `--ignore-scripts`, the binary is downloaded on first `dscode` invocation.

## Requirements

- Node.js 18+
- macOS 11+, Linux (glibc 2.31+), or Windows 10+

## How it works

1. `npm install` triggers a `postinstall` script.
2. The script queries GitHub Releases for the latest DSCode version.
3. It downloads the platform-specific asset (`.dmg`, `.AppImage`, `.zip`, etc.).
4. The asset is extracted into `node_modules/dscode/dist/` (or the global npm prefix).
5. Running `dscode` launches the extracted binary.

## Uninstall

```bash
npm uninstall -g dscode
```

## License

MIT
