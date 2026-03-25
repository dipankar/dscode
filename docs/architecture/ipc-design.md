# IPC Design with NNG

## Overview

DSCode uses **NNG (nanomsg-next-generation)** for inter-process communication between the Rust backend and external processes (extension host, LSP servers). Messages are serialized with **serde_json**. This provides:

- **Reliable delivery**: NNG handles retries, timeouts, and backpressure
- **Simple protocol**: JSON-based messages, easy to debug and extend
- **Multiple patterns**: REQ/REP for RPC, extensible to pub/sub
- **Cross-platform**: Linux, macOS, Windows
- **Async-safe**: All blocking NNG operations run via `tokio::task::spawn_blocking`

## IPC Architecture

```
┌─────────────────────────────────────────────────┐
│  Tauri Main Process (Rust)                       │
│  ┌─────────────────────────────────────────────┐ │
│  │  SessionManager                              │ │
│  │  - IPC handler (session/ipc.rs)              │ │
│  │  - Path validation on all fs operations      │ │
│  │  - spawn_blocking for I/O                    │ │
│  └──────────────────┬──────────────────────────┘ │
│                     │ NNG REQ/REP                 │
│  ┌──────────────────┼──────────────────────────┐ │
│  │  NNG Manager (nng_manager.rs)                │ │
│  │  - Socket lifecycle                          │ │
│  │  - Message routing                          │ │
│  │  - spawn_blocking for recv/send              │ │
│  └──────────────────┬──────────────────────────┘ │
└─────────────────────┼───────────────────────────┘
                      │
         ┌────────────┼──────────────┐
         │            │              │
    ┌────▼─────┐ ┌───▼──────┐ ┌───▼──────┐
    │Extension  │ │   LSP    │ │  Debug   │
    │Host      │ │  Client  │ │  DAP     │
    │(Node.js) │ │          │ │          │
    └──────────┘ └──────────┘ └──────────┘
```

## Message Protocol

### Serialization

All messages use JSON (serde_json). While not zero-copy, JSON provides:

- Easy debugging (human-readable)
- Universal compatibility (Node.js extension host can parse natively)
- Simpler type definitions (standard serde derives)
- Adequate performance for editor IPC (small, frequent messages)

### Message Format

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct IPCMessage {
    pub type: String,
    pub payload: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IPCResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}
```

### Request/Reply Pattern

The extension host communication uses NNG REQ/REP:

```rust
// Rust side (session/ipc.rs)
async fn handle_extension_request(&self, payload: serde_json::Value) -> Result<serde_json::Value> {
    let msg_type = payload.get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    match msg_type {
        "fileRead" => {
            let path = self.path_validator.validate_path(
                payload.get("path").and_then(|v| v.as_str()).unwrap_or("")
            )?;
            let content = tokio::task::spawn_blocking(move || {
                std::fs::read_to_string(&path)
            }).await.map_err(|e| e.to_string())??;
            Ok(json!({ "content": content }))
        }
        // ... other handlers
        _ => Err(format!("Unknown message type: {}", msg_type))
    }
}
```

```typescript
// Extension host side (nng-ipc.ts)
class NNGIPC {
  private worker: Worker;

  async send(message: IPCMessage): Promise<IPCResponse> {
    this.worker.postMessage(message);
    return new Promise((resolve) => {
      this.onResponse = resolve;
    });
  }
}

// Worker thread (handles blocking NNG recv)
const socket = nng.open(req0);
socket.dial(endpoint);

self.onmessage = async (e) => {
  socket.send(JSON.stringify(e.data));
  const reply = socket.recv(); // blocking, OK in worker
  self.postMessage(JSON.parse(reply.toString()));
};
```

## Extension Host IPC

### Worker Thread Pattern

The extension host's NNG receive is a blocking operation that would freeze the Node.js event loop. To avoid this, the blocking `recv()` runs in a dedicated worker thread:

```
Main Thread            Worker Thread
    │                      │
    │ postMessage(req) ──► │ socket.send(req)
    │                      │ reply = socket.recv()  ← blocking, OK here
    │ ◄── postMessage(res) │ postMessage(parse(reply))
    │                      │
    │ dispatch to handler  │
```

### Language Provider IPC

Extensions register language providers (hover, completion, etc.) which the Tauri backend invokes via NNG:

```typescript
// bridge.ts — request/response handlers
handlers.set('provideHover', async (payload) => {
  const provider = languageProviders.get(lang)?.get('hover');
  if (!provider) return null;
  return provider.provideHover(document, position, token);
});

handlers.set('provideCompletionItems', async (payload) => {
  const provider = languageProviders.get(lang)?.get('completion');
  if (!provider) return null;
  return provider.provideCompletionItems(document, position, token, context);
});
```

18+ provider types are supported: Hover, Completion, Diagnostics, SignatureHelp, Rename, CodeLens, CodeActions, Formatting, DocumentHighlights, FoldingRanges, SemanticTokens, DocumentSymbols, WorkspaceSymbols, Definitions, References, DocumentLinks, ColorPresentations, InlineCompletions.

## Security

### Path Validation

All filesystem operations through IPC are validated by `PathValidator`:

- Paths must be within the workspace allowlist
- URI percent-decoding applied before validation
- Path traversal attempts (e.g., `../../../etc/passwd`) rejected
- File size limits enforced (prevents OOM from large reads)
- Error messages sanitized to avoid leaking host paths

```rust
// session/ipc.rs — every handler goes through path validation
"fileRead" => {
    let raw_path = payload.get("path").and_then(|v| v.as_str()).unwrap_or("");
    let validated_path = self.path_validator.validate_path(raw_path)?;
    let content = tokio::task::spawn_blocking(move || {
        std::fs::read_to_string(&validated_path)
    }).await??;
    Ok(json!({ "content": content }))
}
```

### Sandbox Integration

The extension host process runs in a deny-by-default sandbox:

- **macOS**: `sandbox-exec` with dynamically generated Seatbelt profiles
- **Linux**: `bwrap` (bubblewrap) with minimal read-write binds
- **Network**: Denied by default unless extension explicitly requires it
- **Filesystem**: Only `~/.dscode` and workspace directories writable

## Error Handling

### Retry Logic

NNG handles transport-level retries automatically. Application-level errors are returned as structured error responses:

```rust
#[derive(Serialize, Deserialize)]
pub struct IPCError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}
```

### Crash Recovery

The extension host process manager implements exponential backoff restart:

```rust
// extension_host/manager.rs
const MAX_RESTART_ATTEMPTS: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 1000;

async fn handle_crash(&mut self) {
    if self.restart_attempts < MAX_RESTART_ATTEMPTS {
        let backoff = INITIAL_BACKOFF_MS * 2u64.pow(self.restart_attempts);
        tokio::time::sleep(Duration::from_millis(backoff)).await;
        self.restart_attempts += 1;
        self.spawn_extension_host().await;
    }
}
```

## Async Safety

### tokio::sync::Mutex

All async contexts use `tokio::sync::Mutex` instead of `std::sync::Mutex` to prevent deadlocks from holding a lock across `.await` points:

```rust
// lsp/client.rs
pub struct LspClient {
    sender: tokio::sync::Mutex<Option<LspSender>>,
    // NOT std::sync::Mutex (would deadlock if held across .await)
}

// session/mod.rs
pub struct SessionManager {
    lsp_manager: Arc<tokio::sync::Mutex<LspManager>>,
    path_validator: Arc<RwLock<PathValidator>>,
}
```

### spawn_blocking for I/O

All filesystem operations in Tauri command handlers use `tokio::task::spawn_blocking` to avoid blocking the async runtime:

```rust
async fn read_file(path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        std::fs::read_to_string(&path)
    }).await.map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}
```
