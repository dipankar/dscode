# dscode-session

Session manager, extension lifecycle, workspace, and configuration for DSCode.

This crate is the central coordination layer for the DSCode IDE. It manages
extension lifecycle (install, load, unload, delete), workspace folders,
configuration storage, LSP/DAP coordination, document management, and event
emission to the UI.

## Features

- **Session lifecycle state machine** -- `SessionLifecycle` tracks the session
  from `Uninitialized` through `Initializing` to `Ready`, with `Error` and
  `ShuttingDown`/`Shutdown` states for failure recovery and cleanup.
- **Extension management** -- scan, install (VSIX or marketplace), load,
  unload, and delete extensions. Dependency resolution with transitive install.
  VSIX manifest validation and integrity checking.
- **Workspace management** -- add/remove workspace folders, file search with
  glob include/exclude patterns, text search with regex support, file watchers
  for extension-provided patterns.
- **Configuration store** -- hierarchical JSON configuration with dot-notation
  sections, persisted to disk, with change notifications.
- **Extension contributions** -- rich type system for commands, languages,
  grammars, themes, keybindings, menus, snippets, debuggers, views, and
  terminal profiles declared in extension manifests.
- **IPC request handling** -- full bidirectional IPC dispatch for 70+ request
  types from the extension host (status bar, commands, file system, documents,
  decorations, secrets, configuration, language features, and more).
- **Language feature providers** -- register 20+ provider types (completion,
  hover, definition, references, code actions, formatting, rename, etc.) from
  extension contributions.
- **UI interactions** -- status bar items, output channels, quick picks, input
  boxes, message dialogs, progress notifications, and file/folder dialogs.
- **Document management** -- open, persist, apply text edits, track versions,
  detect language IDs, manage decorations.
- **VSIX installation** -- extract, validate, and install `.vsix` extension
  packages with Zip Slip attack prevention and SHA-256 hash verification.

## State Machine

The session follows a strict lifecycle:

```
  Uninitialized ──► Initializing ──► Ready ──► ShuttingDown ──► Shutdown
                        │                          ▲
                        │ (error)                  │
                        ▼                          │
                      Error ───────────────────────┘
                        │
                        │ (retry)
                        ▼
                    Initializing
```

Transitions:
- `Uninitialized` -> `Initializing` (initialize() called)
- `Initializing` -> `Ready` (extension host started, extensions loaded)
- `Initializing` -> `Error` (host startup timeout, spawn failure, or scan failure)
- `Ready` -> `ShuttingDown` (shutdown() called)
- `ShuttingDown` -> `Shutdown` (all extensions unloaded, host stopped)
- `Error` -> `Initializing` (retry initialization)
- `Error` -> `ShuttingDown` (give up and clean up)

During `Initializing`, the extension host process is spawned and IPC
connections are established. The session waits up to 30 seconds for the
`extension-host-ready` signal. If the host fails to start, the session
transitions to `Error` and can be retried.

## Feature Flags

| Feature  | Default | Description |
|----------|---------|-------------|
| `tauri`  | off     | Enables Tauri `AppHandle` integration for event emission, IPC, and file resolution. Required for full session functionality. |

When the `tauri` feature is disabled, the `ipc` and `ipc_providers` modules are
not compiled. The session crate can still be used in headless/testing contexts
for configuration, contributions, and extension scanning.

## Architecture

### SessionManager

The `SessionManager` struct is the top-level coordinator. It holds references to:

- **Session state** (`Arc<RwLock<SessionState>>`) -- workspace folders, active and
  installed extensions, available commands, status bar items.
- **Extension host** (`Arc<tokio::sync::Mutex<ExtensionHostManager>>`) -- the
  Node.js child process running extensions.
- **IPC manager** (`Arc<IpcManager>`) -- bidirectional communication with the
  extension host.
- **LSP pool** (`Arc<RwLock<LspServerPool>>`) -- language server instances.
- **Debug pool** (`Arc<RwLock<DebugAdapterPool>>`) -- debug adapter instances.
- **Configuration** (`Arc<RwLock<ConfigurationStore>>`) -- persistent settings.
- **Path validator** (`Arc<RwLock<PathValidator>>`) -- filesystem access control.
- **Secret storage** (`Arc<SecretStorage>`) -- OS keyring integration.

### Extension Contributions

Extensions declare contributions in their `package.json` manifest. The
`ExtensionContributes` struct parses these into typed Rust structures:

- **Commands** -- `command`, `title`, `category`, `icon`, `enablement`
- **Languages** -- `id`, `extensions`, `filenames`, `firstLine`, `aliases`
- **Grammars** -- `language`, `scopeName`, `path`, `embeddedLanguages`
- **Themes** -- `label`, `uiTheme`, `path`
- **Icon themes** -- `id`, `label`, `path`
- **Keybindings** -- `key`, `command`, `when`, `mac`/`linux`/`win` overrides
- **Menus** -- context menus with `command`, `when`, `group`, `alt`
- **Snippets** -- `language`, `path`
- **Configuration** -- `title`, `properties` (typed with defaults and enums)
- **Debuggers** -- `type`, `label`, `program`, `runtime`, `languages`,
  configuration attributes/snippets
- **Views** -- sidebar view contributions
- **Terminal profiles** -- `id`, `title`, `icon`, `when`

### Workspace Operations

The session provides workspace operations used by the extension host IPC:

- **File search** -- glob-based include/exclude patterns with `.gitignore` respect
- **Text search** -- regex or literal search across workspace files with
  case-sensitivity and word-match options
- **File system operations** -- read, stat, readdir, create directory, write,
  delete, rename, copy (all validated through `PathValidator`)
- **File watchers** -- register/unregister filesystem watchers with glob patterns
  and event type filtering

### Configuration Store

`ConfigurationStore` provides hierarchical JSON settings:

```rust
let mut store = ConfigurationStore::default_in_dir(&dir)?;

// Read a section
let editor_config = store.snapshot(Some("editor"));

// Update a value (dot-notation keys)
let changed = store.update(Some("editor"), "tabSize", json!(4))?;

// Delete a value (pass null)
store.update(None, "editor.tabSize", Value::Null)?;
```

Changes are persisted to disk automatically.

### Document Management

The session tracks open documents with version counters and provides:

- **Open document** -- read file, detect language ID, return content + version
- **Persist document** -- write content to disk, bump version, emit change event
- **Apply text edits** -- apply LSP-style `TextEdit` operations (range-based)
- **Snippet expansion** -- convert VS Code snippets to plain text
- **Decoration management** -- register, update, and dispose editor decorations

## Usage

### Creating a session (with Tauri feature)

```rust
use dscode_session::{
    SessionManager, ExtensionHostManager, IpcManager,
    PathValidator, SecretStorage,
};
use dscode_core::AppDirectories;

let app_dirs = AppDirectories::new(&app_handle)?;
let path_validator = Arc::new(RwLock::new(PathValidator::new()));

let session = SessionManager::new(app_handle, app_dirs, path_validator);

// Initialize the session (starts extension host, loads extensions)
session.initialize().await?;

// Get current state
let state = session.get_state().await;
println!("Workspace folders: {:?}", state.workspace_folders);
println!("Active extensions: {:?}", state.active_extensions);

// Shutdown
session.shutdown().await?;
```

### Extension lifecycle

```rust
// Install from VSIX
let extension = session.install_vsix_package("/path/to/extension.vsix".into()).await?;

// Install from marketplace
let extension = session
    .install_marketplace_extension("publisher".into(), "name".into(), "1.0.0".into())
    .await?;

// Load (activate) an extension
session.load_extension("publisher.name").await?;

// Unload (deactivate) an extension
session.unload_extension("publisher.name").await?;

// Delete an extension
session.delete_extension("publisher.name").await?;

// List installed extensions with full details
let extensions = session.list_extensions_detailed().await?;

// Get extension contributions
let contributions = session.get_extension_contributions_detailed().await?;
```

### Workspace management

```rust
// Add a workspace folder
session.add_workspace_folder(PathBuf::from("/project")).await?;

// Remove a workspace folder
session.remove_workspace_folder(&PathBuf::from("/project")).await?;

// Get detailed workspace folder info
let folders = session.get_workspace_folders_detailed().await;
```

### Handling incoming IPC requests

When the `tauri` feature is enabled, the session automatically handles 70+ IPC
request types from the extension host, including:

| Category | Request types |
|----------|---------------|
| **Extension lifecycle** | `extension-host-ready`, `activate-extension`, `deactivate-extension`, `uninstall-extension`, `reload-extensions` |
| **Commands** | `command-registered`, `command-unregistered`, `executeCommandRequest` |
| **Status bar** | `updateStatusBarItem`, `hideStatusBarItem`, `disposeStatusBarItem`, `window-set-status-bar-message`, `window-clear-status-bar-message` |
| **Documents** | `openTextDocument`, `saveDocument`, `applyEdits`, `insertSnippet`, `setDecorations`, `disposeDecorationType`, `registerDecorationType` |
| **File system** | `fsReadFile`, `fsStat`, `fsReadDirectory`, `fsCreateDirectory`, `fsWriteFile`, `fsDelete`, `fsRename`, `fsCopy` |
| **Configuration** | `workspace-get-configuration`, `workspace-update-configuration` |
| **Secrets** | `secretGet`, `secretStore`, `secretDelete` |
| **UI** | `window-show-message`, `window-show-quick-pick`, `window-show-input-box`, `showOpenFolderDialog`, `showOpenFileDialog` |
| **Language features** | `registerCompletionProvider`, `registerHoverProvider`, `registerDefinitionProvider`, ... (20+ provider types) |
| **Output** | `output-channel-append`, `output-channel-clear`, `output-channel-show`, `output-channel-hide`, `output-channel-dispose` |
| **Watchers** | `workspace-register-watcher`, `workspace-unregister-watcher` |
| **Diagnostics** | `updateDiagnostics`, `clearDiagnostics`, `clearAllDiagnostics` |
| **Context** | `setContext` |

### Resolving UI interactions

The session manages pending UI requests from the extension host that require
user interaction:

```rust
// Resolve a window message action
session.resolve_window_message("request-id", Some("Ok".into())).await?;

// Resolve a quick pick selection
session.resolve_quick_pick("request-id", Some(json!("selected-item"))).await?;

// Resolve an input box value
session.resolve_input_box("request-id", Some("user input".into())).await?;

// Resolve an execute command result
session.resolve_execute_command("request-id", Some(json!({"result": 42}))).await?;
```

## API Surface

### SessionLifecycle

| Variant | Description |
|---------|-------------|
| `Uninitialized` | Initial state, no resources allocated. |
| `Initializing` | Extension host starting, IPC connecting, extensions loading. |
| `Ready` | Normal operation, all subsystems running. |
| `ShuttingDown` | Graceful shutdown in progress. |
| `Shutdown` | Terminal state, all resources released. |
| `Error` | Initialization or runtime error. Can retry or give up. |

### SessionState

| Field | Description |
|-------|-------------|
| `workspace_folders` | Active workspace folder paths. |
| `active_extensions` | Currently loaded/activated extensions. |
| `installed_extensions` | All installed extensions. |
| `available_commands` | All registered commands from extensions and core. |
| `status_bar_items` | Current status bar item states. |

### SessionManager Methods

| Method | Description |
|--------|-------------|
| `new(app_handle, app_dirs, path_validator)` | Create a new session manager. |
| `initialize()` | Start extension host, load extensions, transition to Ready. |
| `shutdown()` | Unload all extensions, stop host, transition to Shutdown. |
| `lifecycle()` | Get current `SessionLifecycle`. |
| `get_state()` | Get current `SessionState`. |
| `ipc_manager()` | Get a reference to the `IpcManager`. |
| `lsp_pool()` | Get a reference to the `LspServerPool`. |
| `debug_pool()` | Get a reference to the `DebugAdapterPool`. |
| `load_extension(id)` | Activate an extension via IPC. |
| `unload_extension(id)` | Deactivate an extension via IPC. |
| `delete_extension(id)` | Unload, uninstall, and delete an extension from disk. |
| `install_vsix_package(path)` | Install extension from a `.vsix` file. |
| `install_marketplace_extension(publisher, name, version)` | Download and install from marketplace. |
| `scan_and_emit_extensions()` | Rescan extensions directory and emit change event. |
| `list_extensions_detailed()` | List all installed extensions with full details. |
| `get_extension_contributions_detailed()` | Get contributions for all installed extensions. |
| `execute_extension_command(command, args)` | Execute a command, activating the owning extension if needed. |
| `add_workspace_folder(path)` | Add a folder to the workspace. |
| `remove_workspace_folder(path)` | Remove a folder from the workspace. |
| `resolve_window_message(id, action)` | Resolve a pending message dialog. |
| `resolve_quick_pick(id, selection)` | Resolve a pending quick pick. |
| `resolve_input_box(id, value)` | Resolve a pending input box. |
| `resolve_execute_command(id, result)` | Resolve a pending command execution. |

### ExtensionContributes

All contribution types from extension manifests:

| Field | Type | Description |
|-------|------|-------------|
| `commands` | `Vec<CommandContribution>` | Command palette entries. |
| `languages` | `Vec<LanguageContribution>` | Language identifiers and file associations. |
| `grammars` | `Vec<GrammarContribution>` | TextMate grammars. |
| `themes` | `Vec<ThemeContribution>` | Color themes. |
| `icon_themes` | `Vec<IconThemeContribution>` | File icon themes. |
| `keybindings` | `Vec<KeybindingContribution>` | Keyboard shortcuts. |
| `menus` | `HashMap<String, Vec<MenuItemContribution>>` | Context menu items. |
| `snippets` | `Vec<SnippetContribution>` | Language snippets. |
| `configuration` | `Option<ConfigurationContribution>` | Settings schema. |
| `debuggers` | `Vec<DebuggerContribution>` | Debug adapter configurations. |
| `breakpoints` | `Vec<BreakpointContribution>` | Breakpoint types. |
| `views` | `HashMap<String, Vec<ViewContribution>>` | Sidebar views. |
| `terminal_profiles` | `Vec<TerminalProfileContribution>` | Terminal profiles. |

### SessionEvent

All events emitted to the UI frontend:

| Event | Data |
|-------|------|
| `ExtensionInstalled` | extension_id |
| `ExtensionLoaded` | extension_id |
| `ExtensionUnloaded` | extension_id |
| `ExtensionDeleted` | extension_id |
| `ExtensionsChanged` | extensions list |
| `WorkspaceFolderAdded` | path |
| `WorkspaceFolderRemoved` | path |
| `StateChanged` | full SessionState |
| `CommandsChanged` | commands list |
| `StatusBarItems` | items list |
| `WindowMessage` | level, message, actions |
| `QuickPickRequest` | id, items, canPickMany, placeholder, title |
| `InputBoxRequest` | id, prompt, placeholder, value, password |
| `OutputChannelAppended` | channel, value |
| `DiagnosticsUpdated` | uri, diagnostics |
| `ConfigurationChanged` | section, key |
| `ProviderRegistered` | provider_type, provider_id, owner, selector |

## Related Crates

- `dscode-extension-host` -- Process lifecycle, IPC, sandbox, and permissions for the Node.js extension host.
- `dscode-dap` -- Debug Adapter Protocol client, manager, and pool.
- `dscode-lsp` -- Language Server Protocol client and pool.
- `dscode-terminal` -- PTY-backed terminal manager.
- `dscode-core` -- Core types including `AppDirectories`.

## License

MIT