# VSCode Extension Host - Independent Crate Architecture

## Overview

This document outlines the architecture for packaging the DSCode extension host as an independent, reusable crate that provides VS Code extension compatibility for any Rust application.

## Crate Name: `vscode-extension-host`

### Design Goals

1. **Framework Agnostic** - Works with Tauri, native apps, or any Rust application
2. **Full VS Code API Compatibility** - Target 95%+ API coverage
3. **Type Safe** - Rust types for all VS Code API interactions
4. **Async Native** - Built on tokio for async operations
5. **Testable** - Comprehensive test suite with mock extensions

---

## Crate Structure

```
vscode-extension-host/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                    # Public API exports
│   ├── host/
│   │   ├── mod.rs
│   │   ├── manager.rs            # Extension lifecycle management
│   │   ├── activation.rs         # Activation event system
│   │   ├── context.rs            # ExtensionContext creation
│   │   └── sandbox.rs            # Extension sandboxing
│   │
│   ├── api/
│   │   ├── mod.rs                # VS Code API types
│   │   ├── commands.rs           # Commands namespace
│   │   ├── window.rs             # Window namespace
│   │   ├── workspace.rs          # Workspace namespace
│   │   ├── languages.rs          # Languages namespace
│   │   ├── debug.rs              # Debug namespace
│   │   ├── tasks.rs              # Tasks namespace
│   │   ├── scm.rs                # SCM namespace
│   │   ├── notebooks.rs          # Notebooks namespace
│   │   ├── testing.rs            # Testing namespace
│   │   └── ...
│   │
│   ├── ipc/
│   │   ├── mod.rs
│   │   ├── protocol.rs           # IPC message protocol
│   │   ├── transport.rs          # Transport abstraction (trait)
│   │   ├── nng.rs                # NNG transport implementation
│   │   └── stdio.rs              # Stdio transport (for LSP-style)
│   │
│   ├── registry/
│   │   ├── mod.rs
│   │   ├── commands.rs           # Command registry
│   │   ├── providers.rs          # Language provider registry
│   │   ├── contributions.rs      # Extension contributions
│   │   └── disposables.rs        # Disposable management
│   │
│   ├── types/
│   │   ├── mod.rs
│   │   ├── uri.rs                # URI implementation
│   │   ├── position.rs           # Position, Range, Selection
│   │   ├── document.rs           # TextDocument
│   │   ├── editor.rs             # TextEditor
│   │   └── ...
│   │
│   └── builtin/
│       ├── mod.rs
│       ├── commands.rs           # Built-in commands
│       └── providers.rs          # Built-in providers
│
├── node/                         # Node.js extension host
│   ├── package.json
│   ├── tsconfig.json
│   └── src/
│       ├── main.ts
│       ├── bridge.ts
│       └── api/
│           └── ...
│
└── tests/
    ├── integration/
    │   ├── activation_test.rs
    │   ├── commands_test.rs
    │   └── providers_test.rs
    └── extensions/               # Test extensions
        ├── test-command/
        ├── test-treeview/
        └── test-language/
```

---

## Public API

### Core Types

```rust
// lib.rs

/// Main extension host that manages extension lifecycle
pub struct ExtensionHost {
    config: ExtensionHostConfig,
    manager: ExtensionManager,
    registry: ExtensionRegistry,
    transport: Box<dyn IpcTransport>,
}

/// Configuration for the extension host
pub struct ExtensionHostConfig {
    /// Directory containing installed extensions
    pub extensions_dir: PathBuf,
    /// Directory for extension storage
    pub storage_dir: PathBuf,
    /// Directory for extension logs
    pub logs_dir: PathBuf,
    /// IPC configuration
    pub ipc: IpcConfig,
    /// Sandbox configuration
    pub sandbox: SandboxConfig,
}

/// Events emitted by the extension host
pub enum ExtensionHostEvent {
    /// Extension was activated
    ExtensionActivated { id: String },
    /// Extension was deactivated
    ExtensionDeactivated { id: String },
    /// Command was registered
    CommandRegistered { id: String, extension: String },
    /// Provider was registered
    ProviderRegistered { kind: ProviderKind, languages: Vec<String> },
    /// UI update required
    UiUpdate(UiUpdateEvent),
    /// Request to host application
    HostRequest(HostRequest),
}

/// Requests from extension host to host application
pub enum HostRequest {
    /// Show message dialog
    ShowMessage { severity: MessageSeverity, message: String, actions: Vec<String> },
    /// Show input box
    ShowInputBox { options: InputBoxOptions },
    /// Show quick pick
    ShowQuickPick { items: Vec<QuickPickItem>, options: QuickPickOptions },
    /// Show open dialog
    ShowOpenDialog { options: OpenDialogOptions },
    /// Show save dialog
    ShowSaveDialog { options: SaveDialogOptions },
    /// Read file
    ReadFile { uri: Uri },
    /// Write file
    WriteFile { uri: Uri, content: Vec<u8> },
    /// Execute in terminal
    TerminalExecute { terminal_id: String, command: String },
    /// Update status bar
    UpdateStatusBar { item: StatusBarItemUpdate },
    /// Update tree view
    UpdateTreeView { view_id: String, event: TreeViewEvent },
}

/// Response from host application
pub enum HostResponse {
    /// Message action selected
    MessageAction(Option<String>),
    /// Input box value
    InputValue(Option<String>),
    /// Quick pick selection
    QuickPickSelection(Option<Vec<usize>>),
    /// File URIs selected
    FileUris(Option<Vec<Uri>>),
    /// File contents
    FileContent(Result<Vec<u8>, FileSystemError>),
    /// Operation success
    Success,
    /// Operation failed
    Error(String),
}
```

### Builder Pattern

```rust
impl ExtensionHost {
    /// Create a new extension host builder
    pub fn builder() -> ExtensionHostBuilder {
        ExtensionHostBuilder::new()
    }
}

pub struct ExtensionHostBuilder {
    config: ExtensionHostConfig,
}

impl ExtensionHostBuilder {
    /// Set extensions directory
    pub fn extensions_dir(mut self, path: impl Into<PathBuf>) -> Self;

    /// Set storage directory
    pub fn storage_dir(mut self, path: impl Into<PathBuf>) -> Self;

    /// Use NNG IPC transport
    pub fn with_nng_transport(mut self, url: &str) -> Self;

    /// Use stdio transport
    pub fn with_stdio_transport(mut self) -> Self;

    /// Use custom transport
    pub fn with_transport(mut self, transport: impl IpcTransport + 'static) -> Self;

    /// Build the extension host
    pub async fn build(self) -> Result<ExtensionHost, ExtensionHostError>;
}
```

### Usage Example

```rust
use vscode_extension_host::{ExtensionHost, ExtensionHostEvent, HostRequest, HostResponse};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create extension host
    let host = ExtensionHost::builder()
        .extensions_dir("~/.myapp/extensions")
        .storage_dir("~/.myapp/storage")
        .with_nng_transport("ipc:///tmp/myapp-ext.ipc")
        .build()
        .await?;

    // Start the extension host
    host.start().await?;

    // Handle events from extensions
    while let Some(event) = host.next_event().await {
        match event {
            ExtensionHostEvent::HostRequest(request) => {
                let response = handle_request(request).await;
                host.respond(response).await?;
            }
            ExtensionHostEvent::UiUpdate(update) => {
                // Update your UI
            }
            _ => {}
        }
    }

    Ok(())
}

async fn handle_request(request: HostRequest) -> HostResponse {
    match request {
        HostRequest::ShowMessage { severity, message, actions } => {
            // Show dialog in your UI framework
            let selected = show_dialog(severity, &message, &actions).await;
            HostResponse::MessageAction(selected)
        }
        HostRequest::ReadFile { uri } => {
            match std::fs::read(uri.fs_path()) {
                Ok(content) => HostResponse::FileContent(Ok(content)),
                Err(e) => HostResponse::FileContent(Err(e.into())),
            }
        }
        // ... handle other requests
        _ => HostResponse::Success,
    }
}
```

---

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1-2)

1. **IPC Protocol Definition**
   - Define message types in Rust
   - Create transport trait
   - Implement NNG transport

2. **Extension Manager**
   - Load extension manifests
   - Spawn Node.js host process
   - Handle lifecycle events

3. **Basic Registries**
   - Command registry
   - Provider registry
   - Disposable tracking

### Phase 2: Activation System (Week 2-3)

1. **Activation Events**
   - `*` and `onStartupFinished`
   - `onLanguage:*`
   - `onCommand:*`
   - `onView:*`
   - `onUri`
   - `onFileSystem:*`

2. **Lazy Loading**
   - Only activate when needed
   - Track activation reasons
   - Activation timing telemetry

### Phase 3: Language Features (Week 3-4)

1. **Provider Invocation**
   - Completion provider
   - Hover provider
   - Definition provider
   - Reference provider
   - Code action provider
   - Document symbol provider

2. **Diagnostics**
   - Diagnostic collection
   - Problem matcher
   - Quick fixes

### Phase 4: UI Features (Week 4-5)

1. **Dialogs**
   - showOpenDialog
   - showSaveDialog
   - showInputBox
   - showQuickPick

2. **Views**
   - TreeView
   - Webview
   - Custom editors

3. **Status Bar**
   - Full status bar API
   - Progress indicators

### Phase 5: Advanced Features (Week 5-6)

1. **Workspace**
   - applyEdit
   - File system watcher
   - Configuration

2. **Debug**
   - Debug adapter protocol
   - Breakpoints
   - Debug console

3. **Tasks**
   - Task providers
   - Task execution
   - Problem matchers

### Phase 6: Polish & Package (Week 6-7)

1. **Testing**
   - Unit tests
   - Integration tests
   - Test extensions

2. **Documentation**
   - API docs
   - Examples
   - Migration guide

3. **Publishing**
   - Crate packaging
   - npm package for Node.js host
   - CI/CD setup

---

## Compatibility Tracking

We'll track VS Code API compatibility with a test matrix:

| API | Status | Test | Notes |
|-----|--------|------|-------|
| `commands.registerCommand` | ✅ | `test_command_registration` | |
| `commands.executeCommand` | ✅ | `test_command_execution` | |
| `window.showInformationMessage` | ✅ | `test_show_message` | |
| `languages.registerHoverProvider` | 🔄 | `test_hover_provider` | In progress |
| ... | | | |

---

## Migration Guide for DSCode

1. Add `vscode-extension-host` as dependency
2. Remove inline extension host code
3. Implement `HostRequest` handlers in Tauri commands
4. Wire up UI events to extension host

```rust
// In Tauri setup
let ext_host = ExtensionHost::builder()
    .extensions_dir(app_dirs.extensions())
    .storage_dir(app_dirs.storage())
    .with_nng_transport(&ipc_url)
    .build()
    .await?;

// Spawn event handler
tauri::async_runtime::spawn(async move {
    while let Some(event) = ext_host.next_event().await {
        match event {
            ExtensionHostEvent::HostRequest(req) => {
                let resp = handle_in_tauri(app_handle.clone(), req).await;
                ext_host.respond(resp).await.ok();
            }
            _ => {}
        }
    }
});
```
