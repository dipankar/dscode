# Extension System Design

## Table of Contents
- [Overview](#overview)
- [Multi-Runtime Architecture](#multi-runtime-architecture)
- [Extension Lifecycle](#extension-lifecycle)
- [VS Code API Implementation](#vs-code-api-implementation)
- [Webview System](#webview-system)
- [Security & Sandboxing](#security--sandboxing)
- [Extension Discovery](#extension-discovery)

## Overview

DSCode's extension system supports **three types of extensions**:

1. **VS Code Extensions** (JavaScript/TypeScript) - Full compatibility
2. **Native Rust Extensions** - High performance
3. **Hybrid Extensions** - Combine both

The system automatically selects the appropriate runtime based on extension requirements.

## Multi-Runtime Architecture

```
┌─────────────────────────────────────────────────────┐
│         Extension Host Process                      │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌──────────────────────────────────────────────┐  │
│  │      Runtime Selector & Manager              │  │
│  └───┬──────────────┬───────────────┬───────────┘  │
│      │              │               │               │
│  ┌───▼────┐    ┌───▼────┐     ┌───▼────┐          │
│  │  Deno  │    │ Node.js│     │ Native │          │
│  │  Core  │    │ Compat │     │  Rust  │          │
│  │        │    │        │     │  (dll) │          │
│  │ ┌────┐ │    │ ┌────┐ │     │        │          │
│  │ │Iso1│ │    │ │Proc│ │     │ ┌────┐ │          │
│  │ │Iso2│ │    │ │1..N│ │     │ │Lib1│ │          │
│  │ │Iso3│ │    │ └────┘ │     │ │Lib2│ │          │
│  │ └────┘ │    │        │     │ └────┘ │          │
│  └────────┘    └────────┘     └────────┘          │
│      │              │               │               │
│      └──────────────┴───────────────┘               │
│                     │                               │
│            ┌────────▼────────┐                      │
│            │ VS Code API     │                      │
│            │ Implementation  │                      │
│            └────────┬────────┘                      │
│                     │                               │
└─────────────────────┼───────────────────────────────┘
                      │ nng IPC
              ┌───────▼────────┐
              │  Main Process  │
              └────────────────┘
```

### Runtime Selection Logic

```rust
pub enum ExtensionRuntime {
    Deno(DenoEngine),
    Node(NodeEngine),
    Native(NativeEngine),
}

impl ExtensionRuntime {
    pub fn select(manifest: &ExtensionManifest) -> Self {
        // Check for native Rust extension
        if manifest.main_path().ends_with(".so")
            || manifest.main_path().ends_with(".dll")
            || manifest.main_path().ends_with(".dylib")
        {
            return ExtensionRuntime::Native(NativeEngine::new());
        }

        // Check for native Node modules
        if manifest.has_native_dependencies() {
            return ExtensionRuntime::Node(NodeEngine::new());
        }

        // Check for webviews (needs full Node for best compat)
        if manifest.contributes_webviews() {
            return ExtensionRuntime::Node(NodeEngine::new());
        }

        // Default: Pure JS/TS → Deno (fastest)
        ExtensionRuntime::Deno(DenoEngine::new())
    }
}

// Extension manifest analysis
impl ExtensionManifest {
    fn has_native_dependencies(&self) -> bool {
        if let Some(deps) = &self.package_json.dependencies {
            // Check for known native modules
            let native_modules = [
                "node-pty",
                "nsfw",
                "spdlog",
                "keytar",
                "@vscode/sqlite3",
                // ... more
            ];
            deps.keys().any(|k| native_modules.contains(&k.as_str()))
        } else {
            false
        }
    }

    fn contributes_webviews(&self) -> bool {
        self.package_json
            .contributes
            .as_ref()
            .map(|c| c.views.is_some() || c.custom_editors.is_some())
            .unwrap_or(false)
    }
}
```

## Extension Lifecycle

### 1. Discovery

```rust
pub struct ExtensionScanner {
    search_paths: Vec<PathBuf>,
}

impl ExtensionScanner {
    pub fn scan(&self) -> Vec<ExtensionManifest> {
        let mut extensions = Vec::new();

        for path in &self.search_paths {
            // ~/.dscode/extensions
            // /usr/share/dscode/extensions
            // ./extensions (built-in)

            for entry in std::fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                let manifest_path = entry.path().join("package.json");

                if manifest_path.exists() {
                    if let Ok(manifest) = ExtensionManifest::load(&manifest_path) {
                        extensions.push(manifest);
                    }
                }
            }
        }

        extensions
    }
}
```

### 2. Activation Events

```json
{
    "activationEvents": [
        "onLanguage:rust",           // When Rust file opened
        "onCommand:myext.cmd",        // When command executed
        "onView:myext.view",          // When view becomes visible
        "onFileSystem:sftp",          // When SFTP file accessed
        "workspaceContains:**/Cargo.toml", // Workspace contains pattern
        "*"                           // Always activate (discouraged)
    ]
}
```

```rust
pub struct ActivationManager {
    extensions: HashMap<ExtensionId, ExtensionInfo>,
    activation_listeners: HashMap<ActivationEvent, Vec<ExtensionId>>,
}

impl ActivationManager {
    pub async fn activate(&mut self, ext_id: &ExtensionId) -> Result<()> {
        let ext = self.extensions.get(ext_id).unwrap();

        // Select runtime
        let runtime = ExtensionRuntime::select(&ext.manifest);

        // Load extension code
        runtime.load(ext).await?;

        // Call activate function
        runtime.call("activate", &[ctx]).await?;

        ext.state = ExtensionState::Activated;
        Ok(())
    }

    pub async fn trigger_activation(&mut self, event: &ActivationEvent) {
        if let Some(ext_ids) = self.activation_listeners.get(event) {
            // Activate in parallel
            let futures: Vec<_> = ext_ids
                .iter()
                .map(|id| self.activate(id))
                .collect();

            futures::future::join_all(futures).await;
        }
    }
}
```

### 3. Deactivation

```rust
impl ActivationManager {
    pub async fn deactivate(&mut self, ext_id: &ExtensionId) -> Result<()> {
        let ext = self.extensions.get_mut(ext_id).unwrap();

        // Call deactivate function (with timeout)
        tokio::time::timeout(
            Duration::from_secs(5),
            ext.runtime.call("deactivate", &[]),
        )
        .await??;

        // Clean up resources
        ext.runtime.dispose().await?;

        ext.state = ExtensionState::Deactivated;
        Ok(())
    }
}
```

## VS Code API Implementation

### API Surface

```typescript
// vscode.d.ts (exposed to extensions)
declare module 'vscode' {
    export namespace window {
        export function showInformationMessage(message: string): Thenable<void>;
        export function showErrorMessage(message: string): Thenable<void>;
        export function showWarningMessage(message: string): Thenable<void>;
        export function showQuickPick(items: string[]): Thenable<string | undefined>;
        export function createOutputChannel(name: string): OutputChannel;
        export function createWebviewPanel(...): WebviewPanel;
        export function createTreeView(...): TreeView;
        export function createTerminal(...): Terminal;
        // ... 100+ more APIs
    }

    export namespace workspace {
        export function openTextDocument(uri: Uri): Thenable<TextDocument>;
        export function saveAll(): Thenable<boolean>;
        export const workspaceFolders: readonly WorkspaceFolder[] | undefined;
        export function findFiles(...): Thenable<Uri[]>;
        // ... 50+ more APIs
    }

    export namespace commands {
        export function registerCommand(command: string, callback: Function): Disposable;
        export function executeCommand(command: string, ...args: any[]): Thenable<any>;
        // ... more
    }

    export namespace languages {
        export function registerCompletionItemProvider(...): Disposable;
        export function registerHoverProvider(...): Disposable;
        export function registerDefinitionProvider(...): Disposable;
        // ... 20+ more provider registrations
    }

    // ... 10+ more namespaces
}
```

### Implementation Bridge

```rust
// Rust side: API implementation
pub struct VSCodeAPI {
    ipc: IPCClient,
}

impl VSCodeAPI {
    pub async fn window_show_information_message(&self, message: String) -> Result<()> {
        let msg = IPCMessage::ShowMessage {
            severity: MessageSeverity::Info,
            message,
        };
        self.ipc.send(msg).await?;
        Ok(())
    }

    pub async fn workspace_open_text_document(&self, uri: String) -> Result<TextDocument> {
        let msg = IPCMessage::OpenTextDocument { uri: uri.clone() };
        let response = self.ipc.send(msg).await?;

        match response {
            IPCMessage::TextDocument(doc) => Ok(doc),
            _ => Err(ExtensionError::UnexpectedResponse),
        }
    }
}

// JavaScript side: API binding (injected into Deno/Node)
// This is generated code that bridges JS → Rust

globalThis.vscode = {
    window: {
        showInformationMessage: async (message) => {
            return await __dscode_ipc_call('window.showInformationMessage', [message]);
        },
        showErrorMessage: async (message) => {
            return await __dscode_ipc_call('window.showErrorMessage', [message]);
        },
        // ... all other APIs
    },
    workspace: {
        openTextDocument: async (uri) => {
            return await __dscode_ipc_call('workspace.openTextDocument', [uri.toString()]);
        },
        // ... all other APIs
    },
    // ... all other namespaces
};

// IPC call function (injected by runtime)
async function __dscode_ipc_call(method, args) {
    const request = {
        method,
        args,
        requestId: generateRequestId(),
    };

    // Send via nng (implemented in Rust, exposed to JS)
    const response = await __dscode_send_ipc(JSON.stringify(request));
    return JSON.parse(response);
}
```

### Deno Runtime Implementation

```rust
use deno_core::*;

pub struct DenoEngine {
    isolates: HashMap<ExtensionId, JsRuntime>,
}

impl DenoEngine {
    pub fn new() -> Self {
        Self {
            isolates: HashMap::new(),
        }
    }

    pub async fn load(&mut self, ext: &ExtensionInfo) -> Result<()> {
        // Create isolated runtime
        let mut runtime = JsRuntime::new(RuntimeOptions {
            module_loader: Some(Rc::new(ExtensionModuleLoader::new())),
            extensions: vec![
                // Inject our IPC bridge as a Deno extension
                dscode_extension::init_ops(),
            ],
            ..Default::default()
        });

        // Inject vscode API shim
        runtime
            .execute_script("<vscode-api>", include_str!("vscode_api_shim.js"))
            .unwrap();

        // Load extension's main file
        let main_module = deno_core::resolve_path(
            &ext.manifest.main_path(),
            &ext.path,
        )?;

        let mod_id = runtime.load_main_module(&main_module, None).await?;
        let result = runtime.mod_evaluate(mod_id);
        runtime.run_event_loop(false).await?;
        result.await?;

        self.isolates.insert(ext.id.clone(), runtime);
        Ok(())
    }

    pub async fn call(&mut self, ext_id: &ExtensionId, method: &str, args: &[Value]) -> Result<Value> {
        let runtime = self.isolates.get_mut(ext_id).unwrap();

        let code = format!(
            r#"
            (async () => {{
                const ext = globalThis.__extension;
                const result = await ext.{}({});
                return result;
            }})()
            "#,
            method,
            serde_json::to_string(&args)?
        );

        let result = runtime.execute_script("<call>", &code)?;
        let result = runtime.resolve_value(result).await?;

        Ok(result)
    }
}

// Deno extension for IPC
#[op]
async fn op_dscode_ipc_call(method: String, args: Value) -> Result<Value, AnyError> {
    // Get IPC client from op state
    let ipc = /* get from state */;

    // Send to main process
    let msg = IPCMessage::ExtensionAPICall { method, args };
    let response = ipc.send(msg).await?;

    match response {
        IPCMessage::ExtensionAPIResponse(result) => Ok(result),
        _ => Err(anyhow::anyhow!("Unexpected response")),
    }
}

deno_core::extension!(
    dscode_extension,
    ops = [op_dscode_ipc_call],
);
```

### Node.js Runtime Implementation

```rust
use napi_rs::*;

pub struct NodeEngine {
    processes: HashMap<ExtensionId, Child>,
    channels: HashMap<ExtensionId, Channel>,
}

impl NodeEngine {
    pub async fn load(&mut self, ext: &ExtensionInfo) -> Result<()> {
        // Spawn Node.js process
        let mut cmd = Command::new("node");
        cmd.arg("--expose-gc");
        cmd.arg(ext.manifest.main_path());

        // Set up IPC channel
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd.spawn()?;
        let channel = Channel::new(child.stdin.take().unwrap(), child.stdout.take().unwrap());

        // Send initialization message
        channel.send(Message::Init {
            extension_id: ext.id.clone(),
            api_socket: format!("ipc:///tmp/dscode-api.ipc"),
        }).await?;

        self.processes.insert(ext.id.clone(), child);
        self.channels.insert(ext.id.clone(), channel);

        Ok(())
    }

    pub async fn call(&mut self, ext_id: &ExtensionId, method: &str, args: &[Value]) -> Result<Value> {
        let channel = self.channels.get_mut(ext_id).unwrap();

        channel.send(Message::Call {
            method: method.to_string(),
            args: args.to_vec(),
        }).await?;

        let response = channel.recv().await?;

        match response {
            Message::Result(value) => Ok(value),
            Message::Error(err) => Err(err.into()),
            _ => Err(ExtensionError::UnexpectedResponse.into()),
        }
    }
}
```

## Webview System

### Architecture

```
┌─────────────────────────────────────────┐
│   Extension (in Extension Host)         │
│                                         │
│   const panel = vscode.window           │
│     .createWebviewPanel(...)            │
│                                         │
│   panel.webview.html = "<div>...</div>" │
│                                         │
│   panel.webview.onDidReceiveMessage(    │
│     msg => { ... }                      │
│   )                                     │
└────────────┬────────────────────────────┘
             │ nng IPC
             │
┌────────────▼────────────────────────────┐
│   Main Process                          │
│                                         │
│   ┌─────────────────────────────────┐   │
│   │  Webview Manager                │   │
│   │  ┌───────────────────────────┐  │   │
│   │  │ Webview Container (wry)   │  │   │
│   │  │                           │  │   │
│   │  │ ┌───────────────────────┐ │  │   │
│   │  │ │   Chromium Engine     │ │  │   │
│   │  │ │   ┌───────────────┐   │ │  │   │
│   │  │ │   │  HTML/CSS/JS  │   │ │  │   │
│   │  │ │   └───────────────┘   │ │  │   │
│   │  │ └───────────────────────┘ │  │   │
│   │  └───────────────────────────┘  │   │
│   └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

### Implementation

```rust
use wry::WebView;

pub struct WebviewPanel {
    id: String,
    webview: WebView,
    extension_id: ExtensionId,
    options: WebviewOptions,
}

pub struct WebviewManager {
    panels: HashMap<String, WebviewPanel>,
    ipc: IPCClient,
}

impl WebviewManager {
    pub fn create_panel(&mut self, id: String, options: WebviewOptions) -> Result<()> {
        // Create webview with wry
        let webview = WebViewBuilder::new()?
            .with_html(&options.initial_html)?
            .with_ipc_handler(|msg| {
                // Message from webview → extension
                self.on_webview_message(id.clone(), msg);
            })
            .build()?;

        let panel = WebviewPanel {
            id: id.clone(),
            webview,
            extension_id: options.extension_id,
            options,
        };

        self.panels.insert(id, panel);
        Ok(())
    }

    pub fn post_message(&mut self, id: &str, message: Value) -> Result<()> {
        let panel = self.panels.get_mut(id).unwrap();

        // Send message from extension → webview
        let script = format!(
            "window.postMessage({}, '*')",
            serde_json::to_string(&message)?
        );

        panel.webview.evaluate_script(&script)?;
        Ok(())
    }

    fn on_webview_message(&self, id: String, message: String) {
        // Forward to extension
        let msg = IPCMessage::WebviewMessage {
            panel_id: id,
            message: serde_json::from_str(&message).unwrap(),
        };

        self.ipc.send(msg);
    }
}
```

### Webview API

```typescript
// Extension code
const panel = vscode.window.createWebviewPanel(
    'myView',
    'My View Title',
    vscode.ViewColumn.One,
    {
        enableScripts: true,
        retainContextWhenHidden: true,
    }
);

// Set HTML content
panel.webview.html = `
    <!DOCTYPE html>
    <html>
    <head>
        <style>
            body { font-family: var(--vscode-font-family); }
        </style>
    </head>
    <body>
        <h1>Hello from Webview</h1>
        <button onclick="sendMessage()">Click me</button>
        <script>
            const vscode = acquireVsCodeApi();

            function sendMessage() {
                vscode.postMessage({ command: 'clicked' });
            }

            window.addEventListener('message', event => {
                const message = event.data;
                console.log('Received:', message);
            });
        </script>
    </body>
    </html>
`;

// Receive messages from webview
panel.webview.onDidReceiveMessage(message => {
    if (message.command === 'clicked') {
        vscode.window.showInformationMessage('Button clicked!');
    }
});

// Send message to webview
panel.webview.postMessage({ command: 'refresh' });
```

## Security & Sandboxing

### Process Isolation

```
Main Process (trusted)
    ↓ nng IPC (controlled API)
Extension Host (semi-trusted)
    ↓ V8 isolate boundary
Extension Code (untrusted)
```

### Capability-Based Security

```json
// package.json
{
    "capabilities": {
        "filesystem": {
            "scope": "workspace",
            "access": "read-write"
        },
        "network": {
            "allowed_hosts": ["api.github.com", "*.myservice.com"]
        },
        "subprocess": {
            "allowed": false
        }
    }
}
```

```rust
pub struct ExtensionSandbox {
    capabilities: Capabilities,
}

impl ExtensionSandbox {
    pub fn check_filesystem_access(&self, path: &Path) -> Result<()> {
        match &self.capabilities.filesystem.scope {
            Scope::Workspace => {
                if !path.starts_with(&workspace_root) {
                    return Err(SecurityError::PermissionDenied);
                }
            }
            Scope::None => return Err(SecurityError::PermissionDenied),
            _ => {}
        }
        Ok(())
    }

    pub fn check_network_access(&self, host: &str) -> Result<()> {
        if !self.capabilities.network.is_allowed(host) {
            return Err(SecurityError::PermissionDenied);
        }
        Ok(())
    }
}
```

### Resource Limits

```rust
pub struct ResourceLimits {
    max_memory: usize,      // 512 MB
    max_cpu_time: Duration,  // 10 seconds per call
    max_file_handles: usize, // 1024
    max_websockets: usize,   // 100
}

impl DenoEngine {
    fn enforce_limits(&self, ext_id: &ExtensionId) {
        let usage = self.get_usage(ext_id);

        if usage.memory > self.limits.max_memory {
            self.kill_extension(ext_id, "Memory limit exceeded");
        }

        if usage.cpu_time > self.limits.max_cpu_time {
            self.kill_extension(ext_id, "CPU time limit exceeded");
        }
    }
}
```

## Extension Discovery

### Marketplace Integration

See [vscode-compat.md](vscode-compat.md) for marketplace details.

### Local Extensions

```
~/.dscode/extensions/
    publisher.extension-1.0.0/
        package.json
        extension.js
        README.md
        ...
```

### Extension Recommendations

```rust
pub struct ExtensionRecommender {
    workspace_patterns: Vec<Pattern>,
}

impl ExtensionRecommender {
    pub fn recommend(&self, workspace: &Workspace) -> Vec<ExtensionId> {
        let mut recommendations = Vec::new();

        // Based on files in workspace
        if workspace.contains("Cargo.toml") {
            recommendations.push("rust-lang.rust-analyzer");
        }

        if workspace.contains("package.json") {
            recommendations.push("dbaeumer.vscode-eslint");
        }

        // Based on popularity (from telemetry)
        recommendations.extend(self.get_popular_extensions());

        recommendations
    }
}
```
