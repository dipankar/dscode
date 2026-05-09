# DSCode Extension Host

[![npm version](https://img.shields.io/npm/v/dscode-extension-host.svg)](https://www.npmjs.com/package/dscode-extension-host)
[![CI](https://github.com/dipankar/dscode/actions/workflows/ci.yml/badge.svg)](https://github.com/dipankar/dscode/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A **VS Code-compatible extension host** for running VS Code extensions in any editor or IDE. This package provides a Node.js runtime that implements the VS Code Extension API, enabling existing VS Code extensions to run with minimal or no modifications.

Part of [DSCode](https://github.com/dipankar/dscode) — a fully hackable Visual Studio Code alternative built in Rust.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Host Application                          │
│                     (Tauri/Electron/etc.)                        │
├─────────────────────────────────────────────────────────────────┤
│                        NNG IPC Layer                             │
│            (REQ/REP sockets for bidirectional RPC)               │
├─────────────────────────────────────────────────────────────────┤
│                    Extension Host (Node.js)                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  vscode.*   │  │  Extension  │  │  Activation │             │
│  │    APIs     │  │   Manager   │  │   Events    │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    VS Code Extensions                     │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │   │
│  │  │Ext 1     │  │Ext 2     │  │Ext 3     │  │Ext N     │ │   │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘ │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Features

- **VS Code API Compatibility**: Implements the core VS Code Extension API
- **Lazy Activation**: Full support for VS Code activation events
- **Language Providers**: Hover, completion, definition, references, and more
- **IPC Bridge**: NNG-based bidirectional communication with host application
- **Secure Secrets Storage**: OS-native keychain integration
- **Tree Views**: Support for custom tree view providers
- **Webviews**: Panel-based webview support
- **SCM Integration**: Source control management API
- **Testing API**: Test controller and test run support
- **Notebooks**: Jupyter-style notebook support

## Installation

```bash
npm install dscode-extension-host
```

## Quick Start

```typescript
import { ExtensionHost } from 'dscode-extension-host';

const host = new ExtensionHost();
await host.start();

// Extensions are loaded from ~/.dscode/extensions by default
```

## VS Code API Compatibility

### Namespaces

| Namespace | Status | Notes |
|-----------|--------|-------|
| `vscode.commands` | ✅ Full | Register, execute, getCommands |
| `vscode.window` | ✅ Full | Messages, dialogs, output channels, terminals, status bar |
| `vscode.workspace` | ✅ Full | Folders, configuration, file operations, applyEdit |
| `vscode.languages` | ✅ Full | All language providers supported |
| `vscode.env` | ✅ Full | Environment info, clipboard, external URIs |
| `vscode.extensions` | ✅ Full | Extension discovery and activation |
| `vscode.debug` | ⚠️ Partial | Configuration providers, basic session management |
| `vscode.tasks` | ⚠️ Partial | Task providers, basic execution |
| `vscode.scm` | ✅ Full | Source control, resource groups, input box |
| `vscode.authentication` | ✅ Full | Session management, providers |
| `vscode.notebooks` | ⚠️ Partial | Basic notebook controller support |
| `vscode.comments` | ✅ Full | Comment controllers and threads |
| `vscode.tests` | ✅ Full | Test controllers, runs, profiles |

### Activation Events

| Event | Status | Description |
|-------|--------|-------------|
| `*` | ✅ | Immediate activation |
| `onLanguage:<id>` | ✅ | When a file of language is opened |
| `onCommand:<id>` | ✅ | When a command is executed |
| `onView:<id>` | ✅ | When a view is revealed |
| `onDebug` | ✅ | When debug session starts |
| `onDebug:<type>` | ✅ | When specific debug type starts |
| `onDebugInitialConfigurations` | ✅ | When debug config is requested |
| `onDebugDynamicConfigurations:<type>` | ✅ | Dynamic debug configurations |
| `onUri` | ✅ | When URI scheme is opened |
| `onFileSystem:<scheme>` | ✅ | When filesystem scheme is accessed |
| `onWebviewPanel:<type>` | ✅ | When webview panel is restored |
| `onCustomEditor:<type>` | ✅ | When custom editor opens |
| `onNotebook:<type>` | ✅ | When notebook type opens |
| `onAuthenticationRequest:<id>` | ✅ | When authentication is requested |
| `onTerminalProfile:<id>` | ✅ | When terminal profile is selected |
| `workspaceContains:<pattern>` | ✅ | When workspace contains file pattern |
| `onStartupFinished` | ✅ | After all `*` extensions activate |

### Language Providers

| Provider | Status | Registration Method |
|----------|--------|---------------------|
| Completion | ✅ Full | `registerCompletionItemProvider` |
| Hover | ✅ Full | `registerHoverProvider` |
| Definition | ✅ Full | `registerDefinitionProvider` |
| References | ✅ Full | `registerReferenceProvider` |
| Code Actions | ✅ Full | `registerCodeActionsProvider` |
| Document Symbols | ✅ Full | `registerDocumentSymbolProvider` |
| Formatting | ✅ Full | `registerDocumentFormattingEditProvider` |
| Range Formatting | ✅ Full | `registerDocumentRangeFormattingEditProvider` |
| On-Type Formatting | ✅ Full | `registerOnTypeFormattingEditProvider` |
| Rename | ✅ Full | `registerRenameProvider` |
| Signature Help | ✅ Full | `registerSignatureHelpProvider` |
| Code Lens | ✅ Full | `registerCodeLensProvider` |
| Document Links | ✅ Full | `registerDocumentLinkProvider` |
| Color Provider | ✅ Full | `registerColorProvider` |
| Folding Range | ✅ Full | `registerFoldingRangeProvider` |
| Selection Range | ✅ Full | `registerSelectionRangeProvider` |
| Call Hierarchy | ✅ Full | `registerCallHierarchyProvider` |
| Type Hierarchy | ✅ Full | `registerTypeHierarchyProvider` |
| Semantic Tokens | ✅ Full | `registerDocumentSemanticTokensProvider` |
| Inline Completion | ✅ Full | `registerInlineCompletionItemProvider` |
| Workspace Symbols | ✅ Full | `registerWorkspaceSymbolProvider` |
| Document Highlight | ✅ Full | `registerDocumentHighlightProvider` |
| Implementation | ✅ Full | `registerImplementationProvider` |
| Type Definition | ✅ Full | `registerTypeDefinitionProvider` |
| Declaration | ✅ Full | `registerDeclarationProvider` |
| Diagnostics | ✅ Full | `createDiagnosticCollection` |

### Window API

| Feature | Status | Method |
|---------|--------|--------|
| Information Message | ✅ | `showInformationMessage` |
| Warning Message | ✅ | `showWarningMessage` |
| Error Message | ✅ | `showErrorMessage` |
| Input Box | ✅ | `showInputBox` |
| Quick Pick | ✅ | `showQuickPick` |
| Open Dialog | ✅ | `showOpenDialog` |
| Save Dialog | ✅ | `showSaveDialog` |
| Workspace Folder Pick | ✅ | `showWorkspaceFolderPick` |
| Output Channels | ✅ | `createOutputChannel` |
| Status Bar Items | ✅ | `createStatusBarItem` |
| Tree Views | ✅ | `createTreeView` |
| Webview Panels | ✅ | `createWebviewPanel` |
| Terminals | ✅ | `createTerminal` |
| Progress | ✅ | `withProgress` |

### Workspace API

| Feature | Status | Method |
|---------|--------|--------|
| Workspace Folders | ✅ | `workspaceFolders` |
| Find Files | ✅ | `findFiles` |
| Open Text Document | ✅ | `openTextDocument` |
| Save Text Document | ✅ | `saveTextDocument` |
| Apply Edit | ✅ | `applyEdit` |
| Configuration | ✅ | `getConfiguration` |
| File System Watcher | ✅ | `createFileSystemWatcher` |
| Workspace Trust | ✅ | `isTrusted` |
| FS API | ✅ | `workspace.fs.*` |

### Types and Enums

All standard VS Code types are exported:

- `Uri`, `Position`, `Range`, `Selection`
- `TextDocument`, `TextEditor`, `TextEdit`, `WorkspaceEdit`
- `DiagnosticSeverity`, `CompletionItemKind`, `SymbolKind`
- `StatusBarAlignment`, `TreeItemCollapsibleState`, `ViewColumn`
- `ProgressLocation`, `ConfigurationTarget`, `ExtensionMode`
- `FileType`, `FilePermission`, `ThemeColor`, `ThemeIcon`
- And many more...

### Built-in Commands

The following VS Code commands are implemented:

| Command | Description |
|---------|-------------|
| `vscode.open` | Open a URI in the editor |
| `vscode.diff` | Show diff between two files |
| `vscode.openFolder` | Open a folder in workspace |
| `vscode.setEditorLayout` | Configure editor layout |
| `workbench.action.openSettings` | Open settings UI |
| `workbench.action.openGlobalKeybindings` | Open keybindings |
| `editor.action.formatDocument` | Format current document |
| `editor.action.formatSelection` | Format selection |
| `editor.action.goToDefinition` | Go to definition |
| `editor.action.peekDefinition` | Peek definition |
| `editor.action.goToReferences` | Find all references |
| `editor.action.rename` | Rename symbol |
| `editor.action.quickFix` | Show quick fixes |
| `editor.action.triggerSuggest` | Trigger completions |
| `editor.action.triggerParameterHints` | Show parameter hints |
| `editor.action.commentLine` | Toggle line comment |
| `editor.action.blockComment` | Toggle block comment |
| `copyFilePath` | Copy file path |
| `copyRelativeFilePath` | Copy relative path |
| `revealFileInOS` | Reveal in file explorer |
| `workbench.action.closeActiveEditor` | Close active editor |
| `workbench.action.closeAllEditors` | Close all editors |

## IPC Protocol

The extension host communicates with the main application via NNG (nanomsg-next-generation) IPC sockets using a request/response pattern.

### Message Format

```typescript
interface IPCMessage {
  id: string;      // Unique message ID
  type: string;    // Message type
  payload: any;    // Message payload
}
```

### Key Message Types

#### Extension Lifecycle
- `activate-extension` - Activate an extension by ID
- `deactivate-extension` - Deactivate an extension
- `list-extensions` - List all loaded extensions
- `reload-extensions` - Reload extension registry
- `uninstall-extension` - Remove an extension

#### Activation Triggers
- `trigger-on-language` - Trigger `onLanguage:<id>` event
- `trigger-on-command` - Trigger `onCommand:<id>` event
- `trigger-on-view` - Trigger `onView:<id>` event
- `trigger-on-debug` - Trigger debug activation
- `signal-startup-finished` - Signal startup complete

#### Language Features
- `provideHover` - Request hover information
- `provideDefinition` - Request definition locations
- `provideReferences` - Request reference locations
- `provideCompletion` - Request completions
- `provideCodeActions` - Request code actions
- `provideDocumentSymbols` - Request document symbols
- `provideDocumentFormatting` - Request formatting edits

#### UI Messages
- `window-show-message` - Show notification
- `window-show-input-box` - Show input dialog
- `window-show-quick-pick` - Show quick pick
- `window-show-open-dialog` - Show file open dialog
- `window-show-save-dialog` - Show file save dialog

#### Workspace Operations
- `workspace-find-files` - Search for files
- `workspace-apply-edit` - Apply workspace edit
- `workspace-get-configuration` - Get configuration
- `workspace-update-configuration` - Update configuration

## Extension Context

Extensions receive an `ExtensionContext` on activation:

```typescript
interface ExtensionContext {
  subscriptions: Disposable[];     // Cleanup on deactivate
  extensionPath: string;           // Extension install path
  extensionUri: Uri;               // Extension URI
  globalState: Memento;            // Global persistent storage
  workspaceState: Memento;         // Workspace persistent storage
  secrets: SecretStorage;          // Secure secret storage
  extensionMode: ExtensionMode;    // Development/Production/Test
  storageUri?: Uri;                // Workspace storage location
  globalStorageUri: Uri;           // Global storage location
  logUri: Uri;                     // Log file location
  asAbsolutePath(path): string;    // Resolve relative paths
}
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DSCODE_IPC_URL` | Outgoing IPC URL | `ipc:///tmp/dscode-extension-host.ipc` |
| `DSCODE_INCOMING_IPC_URL` | Incoming IPC URL | `ipc:///tmp/dscode-incoming-extension-host.ipc` |
| `DSCODE_EXTENSIONS_DIR` | Extensions directory | `~/.dscode/extensions` |

## Testing

```bash
# Run all tests
npm test

# Run tests with coverage
npm run test:coverage

# Run tests in watch mode
npm run test:watch
```

## Building

```bash
# Build TypeScript
npm run build

# Watch mode for development
npm run dev
```

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass (`npm test`)
2. Code follows existing style
3. New features include tests
4. API additions maintain VS Code compatibility

## License

MIT

## Acknowledgments

This project implements the VS Code Extension API as documented at:
- [VS Code Extension API](https://code.visualstudio.com/api)
- [VS Code API Reference](https://code.visualstudio.com/api/references/vscode-api)
