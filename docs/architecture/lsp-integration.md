# LSP Integration

## Overview

DSCode implements a full Language Server Protocol (LSP) client using `tower-lsp`, supporting all LSP features with intelligent server management and performance optimizations.

## Architecture

```
┌──────────────────────────────────────┐
│   Main Process                       │
│   ┌────────────────────────────────┐ │
│   │  LSP Client Manager            │ │
│   │  - Server lifecycle            │ │
│   │  - Request routing             │ │
│   │  - Response caching            │ │
│   └───────────┬────────────────────┘ │
└───────────────┼──────────────────────┘
                │ nng IPC
┌───────────────▼──────────────────────┐
│   LSP Proxy Process                  │
│   ┌────────────────────────────────┐ │
│   │ JSON-RPC ↔ nng Bridge          │ │
│   └───┬────────────────────────────┘ │
│       │                               │
│   ┌───▼────────┐  ┌───────────────┐  │
│   │ rust-       │  │ typescript-   │  │
│   │ analyzer    │  │ language-     │  │
│   └────────────┘  │ server        │  │
│                   └───────────────┘  │
└──────────────────────────────────────┘
```

## Server Management

```rust
pub struct LSPManager {
    servers: HashMap<ServerId, LSPServer>,
    server_configs: HashMap<LanguageId, ServerConfig>,
}

impl LSPManager {
    pub async fn start_server(&mut self, lang: &str) -> Result<ServerId> {
        let config = self.server_configs.get(lang)?;

        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());

        let process = cmd.spawn()?;
        let server = LSPServer::new(process, config.clone());

        // Initialize server
        server.initialize(workspace_root).await?;

        let id = ServerId::new();
        self.servers.insert(id, server);
        Ok(id)
    }
}
```

## Request/Response Flow

```rust
#[derive(Archive, Serialize, Deserialize)]
pub enum LSPRequest {
    Completion { uri: String, position: Position },
    Hover { uri: String, position: Position },
    Definition { uri: String, position: Position },
    References { uri: String, position: Position },
    // ... all LSP requests
}

impl LSPClient {
    pub async fn completion(&self, uri: &str, pos: Position) -> Result<Vec<CompletionItem>> {
        let req = LSPRequest::Completion {
            uri: uri.to_string(),
            position: pos,
        };

        let response = self.send_request(req).await?;
        Ok(response.into_completion_items())
    }
}
```

## Features Supported

- ✅ Completion (with snippets, documentation)
- ✅ Hover (with markdown rendering)
- ✅ Signature Help
- ✅ Go to Definition/Declaration/Implementation
- ✅ Find References
- ✅ Document Symbols
- ✅ Workspace Symbols
- ✅ Code Actions (quick fixes, refactorings)
- ✅ Formatting
- ✅ Rename
- ✅ Diagnostics (errors, warnings)
- ✅ Semantic Tokens
- ✅ Inlay Hints
- ✅ Call Hierarchy
- ✅ Type Hierarchy

## Performance Optimizations

### Request Caching
```rust
pub struct LSPCache {
    completions: LruCache<(String, Position), Vec<CompletionItem>>,
    hover: LruCache<(String, Position), Hover>,
    ttl: Duration,
}
```

### Request Debouncing
```rust
pub struct DebouncedLSP {
    pending: HashMap<RequestId, tokio::task::JoinHandle<Response>>,
    debounce_ms: u64,
}
```

### Parallel Requests
```rust
// Send multiple requests in parallel
let (completions, hover, diagnostics) = tokio::join!(
    lsp.completion(uri, pos),
    lsp.hover(uri, pos),
    lsp.diagnostics(uri),
);
```
