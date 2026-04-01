# Building with DSCode Libraries

This guide shows how to use DSCode crates independently of the full desktop application.

## Minimal Text Editor

A single-file editor with LSP support in under 50 lines of Rust:

```rust
use dscode_core::TextBuffer;
use dscode_lsp::{LspManager, LspServerPool, LspServerStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a file into a rope-backed buffer
    let source = std::fs::read_to_string("src/main.rs")?;
    let mut buffer = TextBuffer::new(&source);

    // Start rust-analyzer
    let pool = LspServerPool::new(LspServerStrategy::OnePerLanguage);
    pool.register_server("rust".to_string(), "rust-analyzer".to_string(), vec!["--stdio".to_string()]).await;

    // Request hover info at line 10, column 5
    let client = pool.get_server("rust", None).await?;
    let hover = client.hover("file:///src/main.rs", 10, 5).await?;
    println!("{:?}", hover);

    // Edit the buffer
    buffer.insert(0, "// Header\n");
    println!("{}", buffer.get_text());

    Ok(())
}
```

## Embedding the Terminal

Use `dscode-terminal` in your own Tauri app or CLI tool:

```rust
use dscode_terminal::{TerminalEventSender, TerminalManager};

struct Logger;
impl TerminalEventSender for Logger {
    fn send_output(&self, id: &str, data: &str) {
        println!("[{}] {}", id, data);
    }
    fn send_close(&self, id: &str) {
        println!("[{}] closed", id);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = TerminalManager::new(Box::new(Logger));
    let id = manager.create_terminal(None, None, None)?;
    manager.start_reading(&id)?;
    manager.write_to_terminal(&id, "cargo test\n")?;
    Ok(())
}
```

For Tauri integration, enable the `tauri` feature and use `TauriEventSender`:

```toml
[dependencies]
dscode-terminal = { version = "0.1", features = ["tauri"] }
tauri = { version = "2" }
```

```rust
use dscode_terminal::{TerminalManager, TauriEventSender};
use tauri::AppHandle;

fn setup_terminal(app: AppHandle) {
    let sender = TauriEventSender::new(app);
    let manager = TerminalManager::new(Box::new(sender));
    // ...
}
```

## Headless Session for CI

Run extension scanning and workspace analysis without a UI:

```rust
use dscode_session::{ConfigurationStore, WorkspaceManager};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a workspace in-memory
    let mut workspace = WorkspaceManager::new();
    workspace.add_folder(PathBuf::from("/repo"))?;

    // Search files
    let files = workspace.find_files("**/*.rs", &Default::default())?;
    println!("Found {} Rust files", files.len());

    // Read configuration
    let mut config = ConfigurationStore::default_in_dir(Path::new("/repo/.dscode"))?;
    let settings = config.snapshot(None);
    println!("Settings: {:?}", settings);

    Ok(())
}
```

## Custom Extension Host

If you want to replace the Node.js extension host with your own runtime:

1. Implement the IPC protocol (length-prefixed JSON over Unix sockets).
2. Use `dscode_extension_host::IpcManager` to handle connections.
3. Set up `PathValidator` and `ExtensionPermissions` for security.
4. Implement `dscode_session::EventEmitter` to bridge events to your UI.

See `crates/dscode-extension-host/examples/sandbox.rs` for a minimal setup.

## Further Reading

- [dscode-core README](../crates/dscode-core/README.md)
- [dscode-lsp README](../crates/dscode-lsp/README.md)
- [dscode-session README](../crates/dscode-session/README.md)
- [Architecture Overview](architecture/overview.md)
