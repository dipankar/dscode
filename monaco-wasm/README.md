# @dscode/monaco-wasm

[![npm version](https://img.shields.io/npm/v/@dscode/monaco-wasm.svg)](https://www.npmjs.com/package/@dscode/monaco-wasm)
[![CI](https://github.com/dscode-dev/dscode/actions/workflows/ci.yml/badge.svg)](https://github.com/dscode-dev/dscode/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

WASM-based fast tokenizer for Monaco Editor, used by DSCode.

## Overview

This package provides a WebAssembly tokenizer that accelerates syntax highlighting in Monaco Editor. It supports JavaScript, TypeScript, Rust, and Python keyword sets, delivering 5-10x faster tokenization compared to pure-JavaScript implementations.

## Install

```bash
npm install @dscode/monaco-wasm
```

## Build

Requires [wasm-pack](https://rustwasm.github.io/wasm-pack/):

```bash
wasm-pack build --target web
```

## Usage

```typescript
import { FastTokenizer } from '@dscode/monaco-wasm';

const tokenizer = new FastTokenizer();
const tokens = tokenizer.tokenizeLine('const x = 1;', 'javascript');
// tokens: [0, 5, 73, 6, 1, 76, ...]  (start, length, type, ...)
```

## Supported Languages

- `javascript`
- `typescript`
- `rust`
- `python`

## License

MIT
