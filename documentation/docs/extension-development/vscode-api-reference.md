---
title: VS Code API Reference
description: Complete reference of the VS Code Extension API as implemented in DSCode, organized by namespace with implementation status.
---

# VS Code API Reference

DSCode implements approximately **75%** of the VS Code Extension API. This page documents every namespace, its key methods and properties, and their current implementation status in DSCode.

!!! info "Status legend"
    | Badge | Meaning |
    |-------|---------|
    | :material-check-circle:{ .status-full } **Implemented** | Fully functional, matches VS Code behavior |
    | :material-circle-half-full:{ .status-partial } **Partial** | Core functionality works; some overloads or options may be missing |
    | :material-circle-outline:{ .status-planned } **Planned** | Not yet implemented; on the roadmap |

---

## `vscode.commands`

Register, execute, and query commands. Commands are the primary way extensions expose functionality.

```typescript
import * as vscode from 'vscode';

// Register a command
const disposable = vscode.commands.registerCommand(
    'myExtension.doSomething',
    (arg1: string, arg2: number) => {
        console.log(`Called with ${arg1}, ${arg2}`);
    }
);

// Execute a command (built-in or from another extension)
const result = await vscode.commands.executeCommand(
    'vscode.openFolder',
    vscode.Uri.file('/path/to/folder')
);

// Get all registered commands
const commands = await vscode.commands.getCommands(/* filterInternal */ true);
```

| API | Status | Notes |
|-----|--------|-------|
| `registerCommand(id, handler)` | :material-check-circle:{ .status-full } Implemented | |
| `registerTextEditorCommand(id, handler)` | :material-check-circle:{ .status-full } Implemented | |
| `executeCommand(id, ...args)` | :material-check-circle:{ .status-full } Implemented | |
| `getCommands(filterInternal?)` | :material-check-circle:{ .status-full } Implemented | |

---

## `vscode.window`

Control the editor UI -- show messages, create status bar items, manage editors, display pickers, and create webview panels.

### Messages and Dialogs

```typescript
// Information, warning, and error messages
const action = await vscode.window.showInformationMessage(
    'Save changes?',
    'Save',
    'Don\'t Save',
    'Cancel'
);

await vscode.window.showWarningMessage('This operation cannot be undone.');
await vscode.window.showErrorMessage('Failed to compile project.');

// Input box
const name = await vscode.window.showInputBox({
    prompt: 'Enter a name',
    placeHolder: 'e.g. my-component',
    validateInput: (value) => {
        return value.length < 3 ? 'Name must be at least 3 characters' : null;
    }
});

// Quick pick
const language = await vscode.window.showQuickPick(
    ['TypeScript', 'JavaScript', 'Rust', 'Python'],
    {
        placeHolder: 'Select a language',
        canPickMany: false
    }
);
```

| API | Status | Notes |
|-----|--------|-------|
| `showInformationMessage(msg, ...items)` | :material-check-circle:{ .status-full } Implemented | |
| `showWarningMessage(msg, ...items)` | :material-check-circle:{ .status-full } Implemented | |
| `showErrorMessage(msg, ...items)` | :material-check-circle:{ .status-full } Implemented | |
| `showInputBox(options?)` | :material-check-circle:{ .status-full } Implemented | |
| `showQuickPick(items, options?)` | :material-check-circle:{ .status-full } Implemented | |
| `showOpenDialog(options)` | :material-check-circle:{ .status-full } Implemented | |
| `showSaveDialog(options)` | :material-check-circle:{ .status-full } Implemented | |
| `showWorkspaceFolderPick(options?)` | :material-circle-half-full:{ .status-partial } Partial | |
| `withProgress(options, task)` | :material-check-circle:{ .status-full } Implemented | |

### Status Bar

```typescript
const item = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    10 // priority
);
item.text = '$(sync~spin) Building...';
item.tooltip = 'Build in progress';
item.color = new vscode.ThemeColor('statusBarItem.warningForeground');
item.command = 'myExtension.cancelBuild';
item.show();
```

| API | Status | Notes |
|-----|--------|-------|
| `createStatusBarItem(alignment?, priority?)` | :material-check-circle:{ .status-full } Implemented | Supports Codicon icons via `$(icon)` syntax |

### Text Editors

```typescript
// Active editor
const editor = vscode.window.activeTextEditor;
if (editor) {
    const selection = editor.selection;
    const text = editor.document.getText(selection);

    // Replace selected text
    await editor.edit(editBuilder => {
        editBuilder.replace(selection, text.toUpperCase());
    });
}

// Listen for editor changes
vscode.window.onDidChangeActiveTextEditor(editor => {
    if (editor) {
        console.log(`Switched to: ${editor.document.fileName}`);
    }
});
```

| API | Status | Notes |
|-----|--------|-------|
| `activeTextEditor` | :material-check-circle:{ .status-full } Implemented | |
| `visibleTextEditors` | :material-check-circle:{ .status-full } Implemented | |
| `onDidChangeActiveTextEditor` | :material-check-circle:{ .status-full } Implemented | |
| `onDidChangeVisibleTextEditors` | :material-check-circle:{ .status-full } Implemented | |
| `onDidChangeTextEditorSelection` | :material-check-circle:{ .status-full } Implemented | |
| `onDidChangeTextEditorOptions` | :material-circle-half-full:{ .status-partial } Partial | |
| `showTextDocument(uri, options?)` | :material-check-circle:{ .status-full } Implemented | |

### Output Channels

```typescript
const output = vscode.window.createOutputChannel('My Extension');
output.appendLine('Starting process...');
output.show(); // Reveal the output panel

// Log output channel (adds timestamps + log levels)
const log = vscode.window.createOutputChannel('My Extension', { log: true });
log.info('Server started');
log.warn('Deprecation notice');
log.error('Connection failed');
```

| API | Status | Notes |
|-----|--------|-------|
| `createOutputChannel(name)` | :material-check-circle:{ .status-full } Implemented | |
| `createOutputChannel(name, {log: true})` | :material-check-circle:{ .status-full } Implemented | |

### Tree Views

```typescript
const treeDataProvider = new MyTreeDataProvider();
const treeView = vscode.window.createTreeView('myExtension.sidebar', {
    treeDataProvider,
    showCollapseAll: true,
    canSelectMany: false
});
```

| API | Status | Notes |
|-----|--------|-------|
| `createTreeView(viewId, options)` | :material-check-circle:{ .status-full } Implemented | |
| `registerTreeDataProvider(viewId, provider)` | :material-check-circle:{ .status-full } Implemented | |

### Webview Panels

```typescript
const panel = vscode.window.createWebviewPanel(
    'myExtension.preview',
    'Preview',
    vscode.ViewColumn.Beside,
    {
        enableScripts: true,
        retainContextWhenHidden: true
    }
);
panel.webview.html = '<html>...</html>';
```

| API | Status | Notes |
|-----|--------|-------|
| `createWebviewPanel(viewType, title, column, options)` | :material-check-circle:{ .status-full } Implemented | See [Webview Extensions](webview-extensions.md) |
| `registerWebviewViewProvider(viewType, provider)` | :material-check-circle:{ .status-full } Implemented | Sidebar webviews |
| `registerWebviewPanelSerializer(viewType, serializer)` | :material-circle-half-full:{ .status-partial } Partial | |

### Terminals

```typescript
const terminal = vscode.window.createTerminal({
    name: 'My Task',
    shellPath: '/bin/bash',
    shellArgs: ['-c', 'npm run build'],
    cwd: '/path/to/project'
});
terminal.show();
terminal.sendText('echo "done"');
```

| API | Status | Notes |
|-----|--------|-------|
| `createTerminal(options)` | :material-check-circle:{ .status-full } Implemented | Backed by portable-pty |
| `activeTerminal` | :material-check-circle:{ .status-full } Implemented | |
| `terminals` | :material-check-circle:{ .status-full } Implemented | |
| `onDidOpenTerminal` | :material-check-circle:{ .status-full } Implemented | |
| `onDidCloseTerminal` | :material-check-circle:{ .status-full } Implemented | |
| `onDidChangeActiveTerminal` | :material-check-circle:{ .status-full } Implemented | |

### Decorations

```typescript
const decorationType = vscode.window.createTextEditorDecorationType({
    backgroundColor: 'rgba(255, 255, 0, 0.3)',
    border: '1px solid yellow'
});

editor.setDecorations(decorationType, [
    new vscode.Range(0, 0, 0, 10)
]);
```

| API | Status | Notes |
|-----|--------|-------|
| `createTextEditorDecorationType(options)` | :material-check-circle:{ .status-full } Implemented | |
| `TextEditor.setDecorations(type, ranges)` | :material-check-circle:{ .status-full } Implemented | |

---

## `vscode.workspace`

Access workspace folders, documents, configuration, and file system operations.

### Workspace Folders and Documents

```typescript
// Get workspace folders
const folders = vscode.workspace.workspaceFolders;
if (folders) {
    console.log(`Root: ${folders[0].uri.fsPath}`);
}

// Open a document
const doc = await vscode.workspace.openTextDocument(
    vscode.Uri.file('/path/to/file.ts')
);

// Open an untitled document
const untitled = await vscode.workspace.openTextDocument({
    language: 'typescript',
    content: 'const x = 42;'
});

// Apply a workspace edit
const edit = new vscode.WorkspaceEdit();
edit.insert(doc.uri, new vscode.Position(0, 0), '// Header\n');
await vscode.workspace.applyEdit(edit);
```

| API | Status | Notes |
|-----|--------|-------|
| `workspaceFolders` | :material-check-circle:{ .status-full } Implemented | |
| `name` | :material-check-circle:{ .status-full } Implemented | |
| `workspaceFile` | :material-check-circle:{ .status-full } Implemented | |
| `openTextDocument(uri)` | :material-check-circle:{ .status-full } Implemented | |
| `openTextDocument(options)` | :material-check-circle:{ .status-full } Implemented | Untitled docs |
| `applyEdit(edit)` | :material-check-circle:{ .status-full } Implemented | |
| `textDocuments` | :material-check-circle:{ .status-full } Implemented | |
| `onDidOpenTextDocument` | :material-check-circle:{ .status-full } Implemented | |
| `onDidCloseTextDocument` | :material-check-circle:{ .status-full } Implemented | |
| `onDidChangeTextDocument` | :material-check-circle:{ .status-full } Implemented | |
| `onDidSaveTextDocument` | :material-check-circle:{ .status-full } Implemented | |
| `onWillSaveTextDocument` | :material-circle-half-full:{ .status-partial } Partial | |

### File System

```typescript
// Watch for file changes
const watcher = vscode.workspace.createFileSystemWatcher(
    '**/*.{ts,js}',
    false, // ignoreCreateEvents
    false, // ignoreChangeEvents
    false  // ignoreDeleteEvents
);

watcher.onDidCreate(uri => console.log(`Created: ${uri.fsPath}`));
watcher.onDidChange(uri => console.log(`Changed: ${uri.fsPath}`));
watcher.onDidDelete(uri => console.log(`Deleted: ${uri.fsPath}`));

// Find files in the workspace
const tsFiles = await vscode.workspace.findFiles(
    '**/*.ts',           // include pattern
    '**/node_modules/**' // exclude pattern
);
```

| API | Status | Notes |
|-----|--------|-------|
| `createFileSystemWatcher(glob)` | :material-check-circle:{ .status-full } Implemented | Backed by Rust `notify` crate |
| `findFiles(include, exclude?)` | :material-check-circle:{ .status-full } Implemented | Backed by Rust `ignore` crate |
| `fs` (FileSystem API) | :material-circle-half-full:{ .status-partial } Partial | `readFile`, `writeFile`, `stat` implemented |
| `registerFileSystemProvider(scheme, provider)` | :material-circle-outline:{ .status-planned } Planned | |

### Configuration

```typescript
// Read configuration
const config = vscode.workspace.getConfiguration('myExtension');
const timeout = config.get<number>('timeout', 5000);
const enabled = config.get<boolean>('enabled', true);

// Update configuration
await config.update('timeout', 10000, vscode.ConfigurationTarget.Global);

// Watch for changes
vscode.workspace.onDidChangeConfiguration(event => {
    if (event.affectsConfiguration('myExtension.timeout')) {
        console.log('Timeout setting changed');
    }
});
```

| API | Status | Notes |
|-----|--------|-------|
| `getConfiguration(section?)` | :material-check-circle:{ .status-full } Implemented | |
| `onDidChangeConfiguration` | :material-check-circle:{ .status-full } Implemented | |
| `Configuration.get(key, default?)` | :material-check-circle:{ .status-full } Implemented | |
| `Configuration.update(key, value, target)` | :material-check-circle:{ .status-full } Implemented | |
| `Configuration.has(key)` | :material-check-circle:{ .status-full } Implemented | |
| `Configuration.inspect(key)` | :material-circle-half-full:{ .status-partial } Partial | |

### Tasks

```typescript
// Register a task provider
const taskProvider = vscode.tasks.registerTaskProvider('myBuild', {
    provideTasks: () => {
        const task = new vscode.Task(
            { type: 'myBuild' },
            vscode.TaskScope.Workspace,
            'Build',
            'my-extension',
            new vscode.ShellExecution('npm run build')
        );
        return [task];
    },
    resolveTask: (task) => task
});
```

| API | Status | Notes |
|-----|--------|-------|
| `registerTaskProvider(type, provider)` | :material-circle-half-full:{ .status-partial } Partial | Shell tasks work; custom execution is planned |
| `fetchTasks(filter?)` | :material-circle-half-full:{ .status-partial } Partial | |
| `executeTask(task)` | :material-circle-half-full:{ .status-partial } Partial | |
| `onDidStartTask` | :material-check-circle:{ .status-full } Implemented | |
| `onDidEndTask` | :material-check-circle:{ .status-full } Implemented | |

---

## `vscode.languages`

Register language features like IntelliSense, diagnostics, code actions, and more.

```typescript
// Register a completion provider
const completionProvider = vscode.languages.registerCompletionItemProvider(
    { language: 'typescript', scheme: 'file' },
    {
        provideCompletionItems(document, position) {
            const item = new vscode.CompletionItem('log');
            item.insertText = new vscode.SnippetString(
                'console.log(${1:message});'
            );
            item.kind = vscode.CompletionItemKind.Snippet;
            return [item];
        }
    },
    '.' // trigger character
);

// Diagnostics
const diagnostics = vscode.languages.createDiagnosticCollection('myLinter');
diagnostics.set(document.uri, [
    new vscode.Diagnostic(
        new vscode.Range(0, 0, 0, 5),
        'Unused variable',
        vscode.DiagnosticSeverity.Warning
    )
]);
```

| API | Status | Notes |
|-----|--------|-------|
| `registerCompletionItemProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerHoverProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerDefinitionProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerReferenceProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerDocumentSymbolProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerWorkspaceSymbolProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerCodeActionsProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerCodeLensProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerDocumentFormattingEditProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerDocumentRangeFormattingEditProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerOnTypeFormattingEditProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerSignatureHelpProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerRenameProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerDocumentHighlightProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerFoldingRangeProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerSelectionRangeProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerInlayHintsProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerDocumentSemanticTokensProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerLinkedEditingRangeProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerColorProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerDeclarationProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerTypeDefinitionProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerImplementationProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerDocumentLinkProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerCallHierarchyProvider` | :material-check-circle:{ .status-full } Implemented | |
| `registerTypeHierarchyProvider` | :material-check-circle:{ .status-full } Implemented | |
| `createDiagnosticCollection(name)` | :material-check-circle:{ .status-full } Implemented | |
| `setDiagnostics(uri, diagnostics)` | :material-check-circle:{ .status-full } Implemented | |
| `getDiagnostics(uri?)` | :material-check-circle:{ .status-full } Implemented | |
| `setLanguageConfiguration(language, config)` | :material-circle-half-full:{ .status-partial } Partial | Brackets and auto-closing pairs work |
| `match(selector, document)` | :material-check-circle:{ .status-full } Implemented | |

!!! tip "See the full guide"
    For detailed code examples of every language provider, see [Language Providers](language-providers.md).

---

## `vscode.env`

Environment information about the editor instance.

```typescript
console.log(vscode.env.appName);    // "DSCode"
console.log(vscode.env.appRoot);    // "/Applications/DSCode.app/..."
console.log(vscode.env.language);   // "en"
console.log(vscode.env.machineId);  // Unique anonymous machine identifier
console.log(vscode.env.uriScheme);  // "dscode"

// Clipboard access
await vscode.env.clipboard.writeText('Hello');
const text = await vscode.env.clipboard.readText();

// Open external URL
await vscode.env.openExternal(
    vscode.Uri.parse('https://dscode.dev')
);
```

| API | Status | Notes |
|-----|--------|-------|
| `appName` | :material-check-circle:{ .status-full } Implemented | Returns `"DSCode"` |
| `appRoot` | :material-check-circle:{ .status-full } Implemented | |
| `language` | :material-check-circle:{ .status-full } Implemented | |
| `machineId` | :material-check-circle:{ .status-full } Implemented | |
| `sessionId` | :material-check-circle:{ .status-full } Implemented | |
| `uriScheme` | :material-check-circle:{ .status-full } Implemented | `"dscode"` |
| `clipboard` | :material-check-circle:{ .status-full } Implemented | read + write |
| `openExternal(uri)` | :material-check-circle:{ .status-full } Implemented | Opens in system browser |
| `isNewAppInstall` | :material-circle-half-full:{ .status-partial } Partial | |
| `isTelemetryEnabled` | :material-check-circle:{ .status-full } Implemented | |
| `remoteName` | :material-circle-outline:{ .status-planned } Planned | For future remote development |
| `shell` | :material-check-circle:{ .status-full } Implemented | |
| `uiKind` | :material-check-circle:{ .status-full } Implemented | Always `UIKind.Desktop` for now |

---

## `vscode.extensions`

Query installed extensions and access their exports.

```typescript
// Get a specific extension
const ext = vscode.extensions.getExtension('publisher.extensionId');
if (ext) {
    // Activate it if not already active
    if (!ext.isActive) {
        await ext.activate();
    }
    // Access its public API
    const api = ext.exports;
}

// List all extensions
const allExtensions = vscode.extensions.all;
console.log(`${allExtensions.length} extensions installed`);

// React to extension changes
vscode.extensions.onDidChange(() => {
    console.log('Extensions list changed');
});
```

| API | Status | Notes |
|-----|--------|-------|
| `getExtension(id)` | :material-check-circle:{ .status-full } Implemented | |
| `all` | :material-check-circle:{ .status-full } Implemented | |
| `onDidChange` | :material-check-circle:{ .status-full } Implemented | |

---

## `vscode.debug`

Debug adapter integration using the Debug Adapter Protocol (DAP).

```typescript
// Register a debug adapter
vscode.debug.registerDebugAdapterDescriptorFactory(
    'myDebugger',
    {
        createDebugAdapterDescriptor(session) {
            return new vscode.DebugAdapterExecutable(
                '/path/to/debug-adapter',
                ['--port', '4711']
            );
        }
    }
);

// Start a debug session
await vscode.debug.startDebugging(
    vscode.workspace.workspaceFolders?.[0],
    {
        type: 'myDebugger',
        request: 'launch',
        name: 'Debug App',
        program: '${workspaceFolder}/app.js'
    }
);

// Monitor active debug session
vscode.debug.onDidChangeActiveDebugSession(session => {
    if (session) {
        console.log(`Debugging: ${session.name}`);
    }
});
```

| API | Status | Notes |
|-----|--------|-------|
| `registerDebugAdapterDescriptorFactory` | :material-circle-half-full:{ .status-partial } Partial | Executable adapters work; server and inline adapters are in progress |
| `registerDebugConfigurationProvider` | :material-circle-half-full:{ .status-partial } Partial | |
| `startDebugging(folder, config)` | :material-circle-half-full:{ .status-partial } Partial | |
| `stopDebugging(session?)` | :material-circle-half-full:{ .status-partial } Partial | |
| `activeDebugSession` | :material-check-circle:{ .status-full } Implemented | |
| `breakpoints` | :material-circle-half-full:{ .status-partial } Partial | |
| `onDidChangeActiveDebugSession` | :material-check-circle:{ .status-full } Implemented | |
| `onDidStartDebugSession` | :material-check-circle:{ .status-full } Implemented | |
| `onDidTerminateDebugSession` | :material-check-circle:{ .status-full } Implemented | |
| `onDidChangeBreakpoints` | :material-circle-half-full:{ .status-partial } Partial | |
| `addBreakpoints(breakpoints)` | :material-circle-half-full:{ .status-partial } Partial | |
| `removeBreakpoints(breakpoints)` | :material-circle-half-full:{ .status-partial } Partial | |
| `registerDebugAdapterTrackerFactory` | :material-circle-outline:{ .status-planned } Planned | |

---

## `vscode.scm`

Source control management API for building Git-like integrations.

| API | Status | Notes |
|-----|--------|-------|
| `createSourceControl(id, label)` | :material-circle-half-full:{ .status-partial } Partial | Basic resource groups work |
| `inputBox` | :material-circle-half-full:{ .status-partial } Partial | |
| `onDidChangeActiveSourceControl` | :material-circle-outline:{ .status-planned } Planned | |

---

## `vscode.authentication`

Authentication provider API for OAuth and token-based auth flows.

| API | Status | Notes |
|-----|--------|-------|
| `getSession(providerId, scopes)` | :material-circle-outline:{ .status-planned } Planned | |
| `registerAuthenticationProvider` | :material-circle-outline:{ .status-planned } Planned | |
| `onDidChangeSessions` | :material-circle-outline:{ .status-planned } Planned | |

---

## `vscode.comments`

Comment thread API for code review and annotation workflows.

| API | Status | Notes |
|-----|--------|-------|
| `createCommentController(id, label)` | :material-circle-outline:{ .status-planned } Planned | |

---

## `vscode.notebooks`

Notebook API for Jupyter-like document support.

| API | Status | Notes |
|-----|--------|-------|
| `createNotebookController(id, viewType, label)` | :material-circle-outline:{ .status-planned } Planned | |
| `registerNotebookSerializer(viewType, serializer)` | :material-circle-outline:{ .status-planned } Planned | |
| `openNotebookDocument(uri)` | :material-circle-outline:{ .status-planned } Planned | |

---

## `vscode.tests`

Test explorer API for discovering and running tests.

| API | Status | Notes |
|-----|--------|-------|
| `createTestController(id, label)` | :material-circle-outline:{ .status-planned } Planned | |
| `TestRunRequest` | :material-circle-outline:{ .status-planned } Planned | |

---

## `ExtensionContext`

Passed to your `activate` function. Provides extension-specific paths, storage, and lifecycle management.

```typescript
export function activate(context: vscode.ExtensionContext) {
    // Extension metadata
    console.log(context.extensionPath);         // Absolute path to extension directory
    console.log(context.extensionUri);          // URI to extension directory
    console.log(context.extension.id);          // "publisher.extensionName"

    // Storage paths
    console.log(context.globalStoragePath);     // Global persistent storage
    console.log(context.storagePath);           // Workspace-specific storage
    console.log(context.logPath);               // Log file directory

    // Persistent state
    context.globalState.update('key', 'value');
    const val = context.globalState.get<string>('key');

    context.workspaceState.update('key', 'value');

    // Secrets storage (encrypted)
    await context.secrets.store('apiKey', 'secret123');
    const key = await context.secrets.get('apiKey');

    // Manage disposables
    context.subscriptions.push(someDisposable);
}
```

| API | Status | Notes |
|-----|--------|-------|
| `extensionPath` / `extensionUri` | :material-check-circle:{ .status-full } Implemented | |
| `globalStoragePath` / `globalStorageUri` | :material-check-circle:{ .status-full } Implemented | |
| `storagePath` / `storageUri` | :material-check-circle:{ .status-full } Implemented | |
| `logPath` / `logUri` | :material-check-circle:{ .status-full } Implemented | |
| `globalState` | :material-check-circle:{ .status-full } Implemented | Memento interface |
| `workspaceState` | :material-check-circle:{ .status-full } Implemented | Memento interface |
| `secrets` | :material-check-circle:{ .status-full } Implemented | Uses OS keychain via Tauri |
| `subscriptions` | :material-check-circle:{ .status-full } Implemented | |
| `extensionMode` | :material-check-circle:{ .status-full } Implemented | |
| `environmentVariableCollection` | :material-circle-half-full:{ .status-partial } Partial | |

---

## Summary: Implementation Coverage

| Namespace | Coverage | Key Gaps |
|-----------|----------|----------|
| `commands` | :material-check-circle:{ .status-full } **100%** | -- |
| `window` | :material-check-circle:{ .status-full } **~90%** | Some custom editor APIs pending |
| `workspace` | :material-check-circle:{ .status-full } **~85%** | `FileSystemProvider` planned |
| `languages` | :material-check-circle:{ .status-full } **~95%** | Full provider coverage |
| `env` | :material-check-circle:{ .status-full } **~90%** | `remoteName` planned |
| `extensions` | :material-check-circle:{ .status-full } **100%** | -- |
| `debug` | :material-circle-half-full:{ .status-partial } **~50%** | Server/inline adapters in progress |
| `scm` | :material-circle-half-full:{ .status-partial } **~30%** | Basic support only |
| `authentication` | :material-circle-outline:{ .status-planned } **0%** | On roadmap |
| `comments` | :material-circle-outline:{ .status-planned } **0%** | On roadmap |
| `notebooks` | :material-circle-outline:{ .status-planned } **0%** | On roadmap |
| `tests` | :material-circle-outline:{ .status-planned } **0%** | On roadmap |

!!! note "Compatibility tracking"
    For a line-by-line compatibility comparison with VS Code, see the [VS Code Compatibility Matrix](../reference/vscode-compatibility-matrix.md).
