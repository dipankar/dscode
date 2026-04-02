# dscode

[![CI](https://github.com/dipankar/dscode/actions/workflows/ci.yml/badge.svg)](https://github.com/dipankar/dscode/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

DSCode — A fast, Rust-based Visual Studio Code alternative with full extension compatibility.

## Overview

This crate is the Tauri-based desktop application entry point for DSCode. It combines a Rust backend (Tauri 2.1) with a Svelte 4 frontend to deliver a high-performance code editor that runs existing VS Code extensions.

## Build from Source

### Prerequisites

- [Rust](https://rust-lang.org/) 1.78+
- [Node.js](https://nodejs.org/) 20+
- System dependencies for Tauri:
  - **Ubuntu/Debian**: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Microsoft Visual C++ Build Tools

### Steps

```bash
# Clone the repository
git clone https://github.com/dipankar/dscode.git
cd dscode

# Install Node dependencies
npm ci

# Download Node.js binaries for the extension host
npm run download-node:all

# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

## Development Setup

The project is organized as a Cargo workspace:

- `src-tauri/` — This crate: Tauri backend and desktop shell
- `src/` — Svelte 4 frontend
- `crates/*/` — Reusable Rust library crates
- `extension-host/` — Node.js extension host runtime
- `monaco-wasm/` — WASM tokenizer for Monaco Editor

See [CONTRIBUTING.md](../CONTRIBUTING.md) for development guidelines.

## Architecture

```
┌─────────────────────────────────────────────┐
│  Frontend (Svelte 4 + Monaco + xterm.js)   │
├─────────────────────────────────────────────┤
│  Tauri IPC Layer                            │
├─────────────────────────────────────────────┤
│  dscode-session  │  dscode-lsp  │  dscode-dap │
├─────────────────────────────────────────────┤
│  dscode-extension-host  │  dscode-terminal   │
├─────────────────────────────────────────────┤
│  dscode-core (TextBuffer, AppDirectories)   │
└─────────────────────────────────────────────┘
```

## License

MIT
