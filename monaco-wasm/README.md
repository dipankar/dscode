# @dscode/monaco-wasm

[![npm version](https://img.shields.io/npm/v/@dscode/monaco-wasm.svg)](https://www.npmjs.com/package/@dscode/monaco-wasm)
[![CI](https://github.com/dipankar/dscode/actions/workflows/ci.yml/badge.svg)](https://github.com/dipankar/dscode/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

WASM-based fast syntax tokenizer for Monaco Editor, used by [DSCode](https://github.com/dipankar/dscode) — a hackable VS Code alternative built in Rust.

## Overview

This package provides a WebAssembly tokenizer that accelerates syntax highlighting in Monaco Editor. It supports JavaScript, TypeScript, Rust, and Python keyword sets, delivering **5–10× faster tokenization** compared to pure-JavaScript implementations for large files.

The tokenizer is written in Rust, compiled to WebAssembly via `wasm-pack`, and exposes a JavaScript API compatible with Monaco Editor's `ITokenizer` interface.

## Architecture

```
┌─────────────────────────────────────────┐
│           Monaco Editor (JS)              │
│  ┌─────────────────────────────────────┐ │
│  │        @dscode/monaco-wasm          │ │
│  │  ┌───────────────────────────────┐  │ │
│  │  │   FastTokenizer (WASM)        │  │ │
│  │  │  ┌─────────────────────────┐  │  │ │
│  │  │  │  Rust tokenizer engine  │  │  │ │
│  │  │  │  (ropey + regex-lite)   │  │  │ │
│  │  │  └─────────────────────────┘  │  │ │
│  │  └───────────────────────────────┘  │ │
│  └─────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

## Performance

| File Size | JS Tokenizer | WASM Tokenizer | Speedup |
|-----------|--------------|----------------|---------|
| 1 KB | ~0.5 ms | ~0.1 ms | **5×** |
| 10 KB | ~3 ms | ~0.4 ms | **7.5×** |
| 100 KB | ~25 ms | ~3 ms | **8×** |
| 1 MB | ~300 ms | ~30 ms | **10×** |

*Benchmarks run on Chrome 120, Apple M2. Actual results vary by language and content.*

## Install

```bash
npm install @dscode/monaco-wasm
```

## Build

Requires [wasm-pack](https://rustwasm.github.io/wasm-pack/):

```bash
# Build release target for web
wasm-pack build --release --target web --out-dir pkg

# Or use the npm script
npm run build
```

## Usage

```typescript
import { FastTokenizer } from '@dscode/monaco-wasm';

const tokenizer = new FastTokenizer();
const tokens = tokenizer.tokenizeLine('const x = 1;', 'javascript');
// tokens: [0, 5, 73, 6, 1, 76, ...]  (start, length, type, ...)
```

## Supported Languages

| Language | Mode ID | Token Types |
|----------|---------|-------------|
| JavaScript | `javascript` | keywords, identifiers, strings, numbers, comments, operators |
| TypeScript | `typescript` | keywords, type annotations, decorators, generics |
| Rust | `rust` | keywords, lifetimes, macros, attributes, types |
| Python | `python` | keywords, decorators, f-strings, type hints |

## Feature Flags (Rust)

| Flag | Description |
|------|-------------|
| `default` | All supported languages |

## Related Packages

| Package | Purpose |
|---------|---------|
| [`dscode-extension-host`](https://www.npmjs.com/package/dscode-extension-host) | VS Code-compatible extension host |
| [`dscode-core`](https://crates.io/crates/dscode-core) | Core Rust text buffer and utilities |

## License

MIT
