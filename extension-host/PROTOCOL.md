# Extension Host IPC Protocol

This document describes the Inter-Process Communication (IPC) protocol between the DSCode Extension Host and the host application.

## Overview

The extension host uses NNG (nanomsg-next-generation) for bidirectional IPC:

- **REP socket** (Reply): Extension host listens for requests from the host application
- **REQ socket** (Request): Extension host sends requests to the host application

All messages are JSON-encoded with the following structure:

```typescript
interface IPCMessage {
  id: string;      // Unique message ID (UUID)
  type: string;    // Message type identifier
  payload: any;    // Message-specific payload
}
```

## Connection Setup

### Environment Variables

| Variable | Description | Platform Default |
|----------|-------------|------------------|
| `DSCODE_IPC_URL` | URL where extension host listens | Linux/macOS: `ipc:///tmp/dscode-extension-host.ipc`<br>Windows: `ipc://\\.\pipe\dscode-extension-host` |
| `DSCODE_INCOMING_IPC_URL` | URL where extension host connects to | Linux/macOS: `ipc:///tmp/dscode-incoming-extension-host.ipc`<br>Windows: `ipc://\\.\pipe\dscode-incoming-extension-host` |

### Handshake

1. Host application creates REP socket and listens on `DSCODE_INCOMING_IPC_URL`
2. Host application creates REQ socket and waits for extension host
3. Extension host starts and creates REP socket on `DSCODE_IPC_URL`
4. Extension host connects REQ socket to `DSCODE_INCOMING_IPC_URL`
5. Extension host sends `extension-host-ready` message
6. Host application can now send requests to extension host

## Message Types

### Extension Lifecycle

#### `activate-extension`
Activate a specific extension.

**Request:**
```json
{
  "extensionId": "publisher.extension-name"
}
```

**Response:**
```json
{
  "success": true
}
```

#### `deactivate-extension`
Deactivate a specific extension.

**Request:**
```json
{
  "extensionId": "publisher.extension-name"
}
```

**Response:**
```json
{
  "success": true
}
```

#### `list-extensions`
Get list of all loaded extensions.

**Request:**
```json
{}
```

**Response:**
```json
{
  "extensions": [
    {
      "id": "publisher.name",
      "name": "Extension Name",
      "displayName": "Display Name",
      "version": "1.0.0",
      "publisher": "publisher",
      "description": "Extension description",
      "path": "/path/to/extension",
      "isActive": true,
      "activationEvents": ["onLanguage:javascript"],
      "contributes": {}
    }
  ]
}
```

#### `reload-extensions`
Reload the extension registry (after install/uninstall).

**Request:**
```json
{}
```

**Response:**
```json
{
  "success": true
}
```

#### `uninstall-extension`
Uninstall an extension.

**Request:**
```json
{
  "extensionId": "publisher.extension-name"
}
```

**Response:**
```json
{
  "success": true
}
```

### Activation Events

#### `signal-startup-finished`
Signal that startup is complete (triggers `onStartupFinished` extensions).

**Request:**
```json
{}
```

**Response:**
```json
{
  "success": true
}
```

#### `trigger-on-language`
Trigger `onLanguage:<id>` activation event.

**Request:**
```json
{
  "languageId": "javascript"
}
```

**Response:**
```json
{
  "success": true
}
```

#### `trigger-on-command`
Trigger `onCommand:<id>` activation event.

**Request:**
```json
{
  "commandId": "extension.doSomething"
}
```

**Response:**
```json
{
  "success": true
}
```

#### `trigger-on-view`
Trigger `onView:<id>` activation event.

**Request:**
```json
{
  "viewId": "myExtension.treeView"
}
```

**Response:**
```json
{
  "success": true
}
```

#### `trigger-on-debug`
Trigger `onDebug` or `onDebug:<type>` activation event.

**Request:**
```json
{
  "debugType": "node"  // Optional
}
```

**Response:**
```json
{
  "success": true
}
```

#### `trigger-on-uri`
Trigger `onUri` activation event.

**Request:**
```json
{
  "scheme": "vscode"  // Optional
}
```

**Response:**
```json
{
  "success": true
}
```

#### `trigger-on-filesystem`
Trigger `onFileSystem:<scheme>` activation event.

**Request:**
```json
{
  "scheme": "sftp"
}
```

**Response:**
```json
{
  "success": true
}
```

#### `trigger-on-webview-panel`
Trigger `onWebviewPanel:<viewType>` activation event.

**Request:**
```json
{
  "viewType": "myExtension.preview"
}
```

**Response:**
```json
{
  "success": true
}
```

#### `trigger-on-custom-editor`
Trigger `onCustomEditor:<viewType>` activation event.

**Request:**
```json
{
  "viewType": "myExtension.editor"
}
```

**Response:**
```json
{
  "success": true
}
```

#### `trigger-on-notebook`
Trigger `onNotebook:<type>` activation event.

**Request:**
```json
{
  "notebookType": "jupyter-notebook"
}
```

**Response:**
```json
{
  "success": true
}
```

#### `trigger-on-authentication`
Trigger `onAuthenticationRequest:<providerId>` activation event.

**Request:**
```json
{
  "providerId": "github"
}
```

**Response:**
```json
{
  "success": true
}
```

#### `trigger-on-terminal-profile`
Trigger `onTerminalProfile:<profileId>` activation event.

**Request:**
```json
{
  "profileId": "myShell"
}
```

**Response:**
```json
{
  "success": true
}
```

#### `trigger-workspace-contains`
Trigger `workspaceContains:<pattern>` activation event.

**Request:**
```json
{
  "pattern": "**/package.json"
}
```

**Response:**
```json
{
  "success": true
}
```

#### `get-pending-activations`
Get list of pending activations (for debugging).

**Request:**
```json
{}
```

**Response:**
```json
{
  "activations": {
    "onLanguage:javascript": ["ext1", "ext2"],
    "onCommand:myCommand": ["ext3"]
  }
}
```

### Language Providers

#### `provideHover`
Request hover information at a position.

**Request:**
```json
{
  "languageId": "typescript",
  "document": {
    "uri": "/path/to/file.ts",
    "languageId": "typescript",
    "version": 1,
    "content": "..."
  },
  "position": {
    "line": 10,
    "character": 5
  }
}
```

**Response:**
```json
{
  "contents": ["Hover content"],
  "range": {
    "start": { "line": 10, "character": 0 },
    "end": { "line": 10, "character": 10 }
  }
}
```

#### `provideDefinition`
Request definition locations.

**Request:**
```json
{
  "languageId": "typescript",
  "document": { ... },
  "position": { "line": 10, "character": 5 }
}
```

**Response:**
```json
{
  "definitions": [
    {
      "uri": "/path/to/definition.ts",
      "range": {
        "start": { "line": 5, "character": 0 },
        "end": { "line": 5, "character": 20 }
      }
    }
  ]
}
```

#### `provideReferences`
Request reference locations.

**Request:**
```json
{
  "languageId": "typescript",
  "document": { ... },
  "position": { "line": 10, "character": 5 },
  "includeDeclaration": true
}
```

**Response:**
```json
{
  "references": [
    {
      "uri": "/path/to/file.ts",
      "range": { ... }
    }
  ]
}
```

#### `provideCompletion`
Request completion items.

**Request:**
```json
{
  "languageId": "typescript",
  "document": { ... },
  "position": { "line": 10, "character": 5 },
  "context": {
    "triggerKind": 1,
    "triggerCharacter": "."
  }
}
```

**Response:**
```json
{
  "items": [
    {
      "label": "toString",
      "kind": 1,
      "detail": "Convert to string",
      "insertText": "toString()"
    }
  ]
}
```

#### `provideCodeActions`
Request code actions for a range.

**Request:**
```json
{
  "languageId": "typescript",
  "document": { ... },
  "range": {
    "start": { "line": 10, "character": 0 },
    "end": { "line": 10, "character": 20 }
  },
  "diagnostics": []
}
```

**Response:**
```json
{
  "actions": [
    {
      "title": "Extract to function",
      "kind": "refactor.extract",
      "command": { "title": "Extract", "command": "..." }
    }
  ]
}
```

#### `provideDocumentSymbols`
Request document symbols.

**Request:**
```json
{
  "languageId": "typescript",
  "document": { ... }
}
```

**Response:**
```json
{
  "symbols": [
    {
      "name": "MyClass",
      "kind": 4,
      "range": { ... },
      "selectionRange": { ... },
      "children": []
    }
  ]
}
```

#### `provideDocumentFormatting`
Request formatting edits.

**Request:**
```json
{
  "languageId": "typescript",
  "document": { ... },
  "options": {
    "tabSize": 2,
    "insertSpaces": true
  }
}
```

**Response:**
```json
{
  "edits": [
    {
      "range": { ... },
      "newText": "formatted code"
    }
  ]
}
```

### Command Execution

#### `executeCommand`
Execute a registered command.

**Request:**
```json
{
  "command": "extension.doSomething",
  "args": ["arg1", "arg2"]
}
```

**Response:**
```json
{
  "success": true,
  "result": "command result"
}
```

### Tree Views

#### `treeView:getChildren`
Get children of a tree item.

**Request:**
```json
{
  "viewId": "myExtension.treeView",
  "element": null  // null for root, or item ID
}
```

**Response:**
```json
{
  "children": [
    {
      "id": "item1",
      "label": "Item 1",
      "collapsibleState": 1,
      "iconPath": "/path/to/icon.svg"
    }
  ]
}
```

#### `treeView:event`
Handle tree view events (selection, expansion, etc.).

**Request:**
```json
{
  "viewId": "myExtension.treeView",
  "event": "select",
  "element": "item1"
}
```

**Response:**
```json
{
  "success": true
}
```

### Configuration

#### `configuration-changed`
Notify extension host of configuration changes.

**Request:**
```json
{
  "section": "myExtension",
  "key": "setting"
}
```

**Response:**
```json
{
  "success": true
}
```

### File System Watcher

#### `fsWatcher:event`
Notify extension host of file system events.

**Request:**
```json
{
  "id": "watcher_123",
  "event": "changed",  // "created" | "changed" | "deleted"
  "path": "/path/to/file.txt"
}
```

**Response:**
```json
{
  "success": true
}
```

## Messages FROM Extension Host

The following messages are sent from the extension host TO the host application:

### Ready Signal

#### `extension-host-ready`
Sent when extension host is ready to receive requests.

```json
{
  "ready": true
}
```

### Command Registration

#### `command-registered`
Sent when an extension registers a command.

```json
{
  "command": "extension.myCommand",
  "owner": "publisher.extension-name"
}
```

#### `command-unregistered`
Sent when a command is unregistered.

```json
{
  "command": "extension.myCommand",
  "owner": "publisher.extension-name"
}
```

### Provider Registration

#### `registerCompletionProvider`
```json
{
  "languages": ["javascript", "typescript"],
  "triggerCharacters": [".", "("]
}
```

#### `registerHoverProvider`
```json
{
  "languages": ["javascript"]
}
```

#### `registerDefinitionProvider`
```json
{
  "languages": ["typescript"]
}
```

### UI Messages

#### `window-show-message`
Show a notification message.

```json
{
  "type": "info",  // "info" | "warning" | "error"
  "message": "Hello, World!"
}
```

#### `window-show-message-with-actions`
Show message with action buttons.

**Request:**
```json
{
  "type": "info",
  "message": "Do you want to proceed?",
  "actions": ["Yes", "No"]
}
```

**Response:**
```json
{
  "action": "Yes"
}
```

#### `window-show-input-box`
Show input dialog.

**Request:**
```json
{
  "prompt": "Enter your name",
  "placeHolder": "Name",
  "value": "",
  "password": false
}
```

**Response:**
```json
{
  "value": "John Doe"
}
```

#### `window-show-quick-pick`
Show quick pick menu.

**Request:**
```json
{
  "items": [
    { "label": "Option 1", "description": "First option" },
    { "label": "Option 2", "description": "Second option" }
  ],
  "options": {
    "placeHolder": "Select an option",
    "canPickMany": false
  }
}
```

**Response:**
```json
{
  "selected": { "label": "Option 1", "description": "First option" }
}
```

#### `window-show-open-dialog`
Show file open dialog.

**Request:**
```json
{
  "canSelectFiles": true,
  "canSelectFolders": false,
  "canSelectMany": false,
  "openLabel": "Open",
  "title": "Select a file",
  "filters": {
    "TypeScript": ["ts", "tsx"],
    "All Files": ["*"]
  }
}
```

**Response:**
```json
{
  "uris": ["/path/to/selected/file.ts"]
}
```

#### `window-show-save-dialog`
Show file save dialog.

**Request:**
```json
{
  "saveLabel": "Save",
  "title": "Save file as",
  "defaultUri": "/path/to/default.txt",
  "filters": {
    "Text Files": ["txt"]
  }
}
```

**Response:**
```json
{
  "uri": "/path/to/saved/file.txt"
}
```

### Workspace Operations

#### `workspace-find-files`
Find files matching a pattern.

**Request:**
```json
{
  "include": "**/*.ts",
  "exclude": "**/node_modules/**",
  "maxResults": 100
}
```

**Response:**
```json
{
  "files": ["/path/to/file1.ts", "/path/to/file2.ts"]
}
```

#### `workspace-apply-edit`
Apply a workspace edit.

**Request:**
```json
{
  "edits": [
    {
      "uri": "/path/to/file.ts",
      "range": {
        "start": { "line": 0, "character": 0 },
        "end": { "line": 0, "character": 10 }
      },
      "newText": "replacement text"
    }
  ],
  "createFiles": [],
  "deleteFiles": [],
  "renameFiles": []
}
```

**Response:**
```json
{
  "success": true
}
```

#### `workspace-get-configuration`
Get configuration values.

**Request:**
```json
{
  "section": "myExtension"
}
```

**Response:**
```json
{
  "config": {
    "setting1": true,
    "setting2": "value"
  }
}
```

#### `workspace-update-configuration`
Update a configuration value.

**Request:**
```json
{
  "section": "myExtension",
  "key": "setting1",
  "value": false,
  "target": "global"  // "global" | "workspace" | "workspaceFolder"
}
```

**Response:**
```json
{
  "success": true
}
```

### File System Watcher

#### `workspace-register-watcher`
Register a file system watcher.

**Request:**
```json
{
  "id": "watcher_123",
  "globPattern": "**/*.ts",
  "ignoreCreateEvents": false,
  "ignoreChangeEvents": false,
  "ignoreDeleteEvents": false
}
```

**Response:**
```json
{
  "success": true
}
```

#### `workspace-unregister-watcher`
Unregister a file system watcher.

**Request:**
```json
{
  "id": "watcher_123"
}
```

**Response:**
```json
{
  "success": true
}
```

### Secrets Storage

#### `secretGet`
Get a secret from secure storage.

**Request:**
```json
{
  "extensionId": "publisher.extension-name",
  "key": "api-token"
}
```

**Response:**
```json
{
  "value": "secret-value"
}
```

#### `secretStore`
Store a secret in secure storage.

**Request:**
```json
{
  "extensionId": "publisher.extension-name",
  "key": "api-token",
  "value": "secret-value"
}
```

**Response:**
```json
{
  "success": true
}
```

#### `secretDelete`
Delete a secret from secure storage.

**Request:**
```json
{
  "extensionId": "publisher.extension-name",
  "key": "api-token"
}
```

**Response:**
```json
{
  "success": true
}
```

## Error Handling

All error responses follow this format:

```json
{
  "success": false,
  "error": "Error message describing what went wrong"
}
```

## Input Validation

The extension host validates all incoming messages:

- **Extension IDs**: Alphanumeric with dots, dashes, underscores (max 256 chars)
- **Command IDs**: Alphanumeric with dots, colons, dashes, underscores (max 4096 chars)
- **View IDs**: Same as command IDs
- **File paths**: No path traversal (.. is rejected)
- **Payloads**: Must be valid JSON objects

## Security Considerations

1. **Input validation**: All inputs are validated before processing
2. **Path traversal prevention**: Extension IDs and paths cannot contain `..`
3. **Secrets isolation**: Each extension's secrets are namespaced by extension ID
4. **Command isolation**: Commands are tracked by owner extension
5. **IPC isolation**: Sockets use local-only IPC (no network exposure)
