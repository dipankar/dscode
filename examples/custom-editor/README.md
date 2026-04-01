# Custom Editor Example

A minimal Tauri application that uses `dscode-core` and `dscode-lsp` to build a single-file code editor.

## Structure

```
examples/custom-editor/
├── Cargo.toml
├── src/
│   └── main.rs
└── README.md
```

## Running

```bash
cd examples/custom-editor
cargo run
```

## What it demonstrates

- Using `TextBuffer` for rope-backed text storage
- Spawning `rust-analyzer` via `LspServerPool`
- Sending an LSP `initialize` request
- Bridging LSP responses to a simple stdout UI

## Extending

Replace the stdout "UI" with a minimal webview (e.g., `wry` or a full Tauri window) to turn this into a standalone desktop editor.
