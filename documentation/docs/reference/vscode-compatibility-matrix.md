---
title: VS Code Compatibility Matrix
description: Detailed compatibility matrix showing which VS Code APIs, namespaces, and contribution points are supported in DSCode.
---

# VS Code Compatibility Matrix

DSCode targets approximately **75% coverage** of the VS Code extension API. This page provides a detailed breakdown of compatibility by namespace, individual API surface, and contribution point.

!!! info "How to read the status badges"
    - <span class="status-badge full">Full</span> -- All APIs in this namespace are implemented and pass conformance tests
    - <span class="status-badge partial">Partial</span> -- Core APIs work; some advanced or rarely used APIs are missing
    - <span class="status-badge planned">Planned</span> -- Not yet implemented; on the roadmap for a future release

---

## API Namespace Status

The following table summarizes compatibility for each top-level `vscode.*` namespace.

| Namespace | API Count | Implemented | Status | Notes |
|---|---|---|---|---|
| `vscode.commands` | 6 | 6 | <span class="status-badge full">Full</span> | `registerCommand`, `registerTextEditorCommand`, `executeCommand`, `getCommands` all supported |
| `vscode.window` | 42 | 34 | <span class="status-badge partial">Partial</span> | Core UI APIs work. Missing: `registerCustomEditorProvider`, `registerTerminalProfileProvider`, `registerUriHandler`, some `createTreeView` options |
| `vscode.workspace` | 38 | 28 | <span class="status-badge partial">Partial</span> | File system, configuration, and workspace folders work. Missing: `registerFileSystemProvider` (virtual FS), `registerNotebookSerializer`, some workspace edit features |
| `vscode.languages` | 32 | 32 | <span class="status-badge full">Full</span> | All 24 provider types implemented (completions, hover, diagnostics, formatting, code actions, code lenses, document links, folding, semantic tokens, inlay hints, etc.) |
| `vscode.debug` | 12 | 8 | <span class="status-badge partial">Partial</span> | DAP client works. `registerDebugAdapterDescriptorFactory`, `startDebugging`, breakpoints, and debug console functional. Missing: some advanced breakpoint types, `registerDebugConfigurationProvider` options |
| `vscode.env` | 10 | 10 | <span class="status-badge full">Full</span> | `appName`, `appRoot`, `language`, `machineId`, `sessionId`, `uriScheme`, `clipboard`, `openExternal`, `shell`, `isNewAppInstall` |
| `vscode.extensions` | 4 | 4 | <span class="status-badge full">Full</span> | `getExtension`, `all`, `onDidChange` fully supported |
| `vscode.tasks` | 8 | 5 | <span class="status-badge partial">Partial</span> | `registerTaskProvider`, `executeTask`, `taskExecutions` work. Missing: custom problem matchers, `onDidStartTaskProcess` / `onDidEndTaskProcess` |
| `vscode.scm` | 6 | 4 | <span class="status-badge partial">Partial</span> | `createSourceControl`, resource groups, and input box work. Missing: `SourceControlResourceDecorations` advanced options, some history APIs |
| `vscode.notebooks` | 18 | 0 | <span class="status-badge planned">Planned</span> | Notebook support is planned for a future release. No APIs currently implemented |
| `vscode.tests` | 10 | 6 | <span class="status-badge partial">Partial</span> | `createTestController`, test items, test runs, and result reporting work. Missing: test coverage, continuous run, and some discovery APIs |
| `vscode.comments` | 6 | 0 | <span class="status-badge planned">Planned</span> | Comment thread API planned for a future release |
| `vscode.authentication` | 4 | 0 | <span class="status-badge planned">Planned</span> | Authentication provider API planned; extensions needing OAuth should use external browser flows for now |
| `vscode.l10n` | 4 | 0 | <span class="status-badge planned">Planned</span> | Localization API planned. Extensions can use their own i18n libraries in the meantime |

---

## Detailed `vscode.window` Coverage

Since `vscode.window` is the largest and most varied namespace, here is a detailed breakdown:

| API | Status | Notes |
|---|---|---|
| `showInformationMessage` | <span class="status-badge full">Full</span> | All overloads supported |
| `showWarningMessage` | <span class="status-badge full">Full</span> | All overloads supported |
| `showErrorMessage` | <span class="status-badge full">Full</span> | All overloads supported |
| `showQuickPick` | <span class="status-badge full">Full</span> | Includes multi-select, custom buttons |
| `showInputBox` | <span class="status-badge full">Full</span> | Validation, placeholders, and password mode |
| `showOpenDialog` | <span class="status-badge full">Full</span> | File and folder selection |
| `showSaveDialog` | <span class="status-badge full">Full</span> | Save dialogs with filters |
| `createStatusBarItem` | <span class="status-badge full">Full</span> | Alignment, priority, tooltip, command |
| `createOutputChannel` | <span class="status-badge full">Full</span> | Standard and log output channels |
| `createWebviewPanel` | <span class="status-badge full">Full</span> | Webview lifecycle, messaging, serialization |
| `createTreeView` | <span class="status-badge partial">Partial</span> | Basic tree views work. Missing: drag-and-drop, badge |
| `createTerminal` | <span class="status-badge full">Full</span> | Shell, process, and renderer terminals |
| `setStatusBarMessage` | <span class="status-badge full">Full</span> | Timed messages with auto-dispose |
| `withProgress` | <span class="status-badge full">Full</span> | Notification and status bar progress |
| `showTextDocument` | <span class="status-badge full">Full</span> | All view columns and options |
| `activeTextEditor` | <span class="status-badge full">Full</span> | Read-only property |
| `visibleTextEditors` | <span class="status-badge full">Full</span> | Read-only property |
| `onDidChangeActiveTextEditor` | <span class="status-badge full">Full</span> | Event |
| `onDidChangeVisibleTextEditors` | <span class="status-badge full">Full</span> | Event |
| `onDidChangeTextEditorSelection` | <span class="status-badge full">Full</span> | Event |
| `registerCustomEditorProvider` | <span class="status-badge planned">Planned</span> | Custom binary/visual editors not yet supported |
| `registerTerminalProfileProvider` | <span class="status-badge planned">Planned</span> | Terminal profiles are hardcoded for now |
| `registerUriHandler` | <span class="status-badge planned">Planned</span> | URI handling will be added with authentication support |
| `registerWebviewViewProvider` | <span class="status-badge partial">Partial</span> | Sidebar webview panels work; some options missing |

---

## Detailed `vscode.languages` Coverage

All 24 language provider types are implemented. DSCode routes these through the `language_features_registry` in the Rust backend.

| Provider | Status | Notes |
|---|---|---|
| `registerCompletionItemProvider` | <span class="status-badge full">Full</span> | Trigger characters, resolve support |
| `registerHoverProvider` | <span class="status-badge full">Full</span> | Markdown content supported |
| `registerSignatureHelpProvider` | <span class="status-badge full">Full</span> | Retrigger characters |
| `registerDefinitionProvider` | <span class="status-badge full">Full</span> | Multi-location results |
| `registerTypeDefinitionProvider` | <span class="status-badge full">Full</span> | |
| `registerImplementationProvider` | <span class="status-badge full">Full</span> | |
| `registerReferenceProvider` | <span class="status-badge full">Full</span> | Include declaration option |
| `registerDocumentHighlightProvider` | <span class="status-badge full">Full</span> | Read/write highlights |
| `registerDocumentSymbolProvider` | <span class="status-badge full">Full</span> | Hierarchical symbols |
| `registerCodeActionsProvider` | <span class="status-badge full">Full</span> | Quick fixes, refactoring, source actions |
| `registerCodeLensProvider` | <span class="status-badge full">Full</span> | Resolve support |
| `registerDocumentFormattingEditProvider` | <span class="status-badge full">Full</span> | |
| `registerDocumentRangeFormattingEditProvider` | <span class="status-badge full">Full</span> | |
| `registerOnTypeFormattingEditProvider` | <span class="status-badge full">Full</span> | Trigger characters |
| `registerRenameProvider` | <span class="status-badge full">Full</span> | Prepare rename support |
| `registerDocumentLinkProvider` | <span class="status-badge full">Full</span> | Resolve support |
| `registerColorProvider` | <span class="status-badge full">Full</span> | Color presentations |
| `registerFoldingRangeProvider` | <span class="status-badge full">Full</span> | |
| `registerSelectionRangeProvider` | <span class="status-badge full">Full</span> | |
| `registerCallHierarchyProvider` | <span class="status-badge full">Full</span> | Incoming/outgoing calls |
| `registerTypeHierarchyProvider` | <span class="status-badge full">Full</span> | Super/sub types |
| `registerSemanticTokensProvider` | <span class="status-badge full">Full</span> | Full and delta providers |
| `registerInlayHintsProvider` | <span class="status-badge full">Full</span> | Resolve support |
| `registerLinkedEditingRangeProvider` | <span class="status-badge full">Full</span> | |
| `createDiagnosticCollection` | <span class="status-badge full">Full</span> | |
| `setLanguageConfiguration` | <span class="status-badge full">Full</span> | Brackets, comments, indentation |
| `registerDocumentSemanticTokensProvider` | <span class="status-badge full">Full</span> | Legend and token encoding |

---

## Contribution Points Compatibility

Contribution points are static declarations in an extension's `package.json` that register commands, menus, keybindings, themes, and other features. The following table lists support for each contribution type.

| Contribution Point | Status | Notes |
|---|---|---|
| `commands` | <span class="status-badge full">Full</span> | Title, category, icon, enablement. Parsed by the `command_registry` module |
| `menus` | <span class="status-badge full">Full</span> | All menu locations supported: `editor/context`, `editor/title`, `view/title`, `view/item/context`, `commandPalette`, `explorer/context`, `scm/title`, `scm/resourceGroup/context`, `scm/resourceState/context`. Parsed by `extension_menu_parser` |
| `keybindings` | <span class="status-badge full">Full</span> | `key`, `mac`, `linux`, `win`, `when`, `command`, `args`. Parsed by `extension_keybinding_parser` |
| `languages` | <span class="status-badge full">Full</span> | Language ID, extensions, aliases, configuration, first line pattern |
| `grammars` | <span class="status-badge full">Full</span> | TextMate grammars (`.tmLanguage`, `.tmLanguage.json`, `.plist`). Injected and embedded grammars supported |
| `themes` | <span class="status-badge full">Full</span> | Color themes (`.json` and `.tmTheme`). Managed by `theme_registry` |
| `iconThemes` | <span class="status-badge full">Full</span> | File icon themes. Managed by `theme_registry` |
| `snippets` | <span class="status-badge full">Full</span> | VS Code snippet JSON format. Language-scoped and global snippets |
| `viewsContainers` | <span class="status-badge full">Full</span> | Activity bar and panel containers. Managed by `activity_bar_registry` |
| `views` | <span class="status-badge full">Full</span> | Tree views and webview views. `when` clauses and visibility conditions |
| `configuration` | <span class="status-badge full">Full</span> | Extension settings with JSON Schema validation. Managed by `configuration_registry` |
| `debuggers` | <span class="status-badge partial">Partial</span> | Debug adapter contributions work. Missing: some `configurationAttributes` schema features, `initialConfigurations` dynamic providers |
| `taskDefinitions` | <span class="status-badge partial">Partial</span> | Task type registration works. Missing: problem matcher definitions, custom task presentations |
| `customEditors` | <span class="status-badge planned">Planned</span> | Custom editor contributions not yet supported |
| `notebookRenderer` | <span class="status-badge planned">Planned</span> | Notebook renderers planned with notebook support |
| `walkthroughs` | <span class="status-badge planned">Planned</span> | Getting started walkthroughs not yet supported |
| `terminal` | <span class="status-badge planned">Planned</span> | Terminal profile contributions planned |
| `authentication` | <span class="status-badge planned">Planned</span> | Authentication provider contributions planned |

---

## Event API Coverage

DSCode implements the core event system that extensions rely on for reactive behavior.

| Event | Status | Notes |
|---|---|---|
| `workspace.onDidOpenTextDocument` | <span class="status-badge full">Full</span> | |
| `workspace.onDidCloseTextDocument` | <span class="status-badge full">Full</span> | |
| `workspace.onDidChangeTextDocument` | <span class="status-badge full">Full</span> | Full change events with ranges and text |
| `workspace.onDidSaveTextDocument` | <span class="status-badge full">Full</span> | |
| `workspace.onWillSaveTextDocument` | <span class="status-badge full">Full</span> | Supports `waitUntil` for edits |
| `workspace.onDidChangeConfiguration` | <span class="status-badge full">Full</span> | Scoped to affected keys |
| `workspace.onDidChangeWorkspaceFolders` | <span class="status-badge full">Full</span> | |
| `workspace.onDidCreateFiles` | <span class="status-badge full">Full</span> | |
| `workspace.onDidDeleteFiles` | <span class="status-badge full">Full</span> | |
| `workspace.onDidRenameFiles` | <span class="status-badge full">Full</span> | |
| `window.onDidChangeActiveTextEditor` | <span class="status-badge full">Full</span> | |
| `window.onDidChangeVisibleTextEditors` | <span class="status-badge full">Full</span> | |
| `window.onDidChangeTextEditorSelection` | <span class="status-badge full">Full</span> | |
| `window.onDidChangeActiveTerminal` | <span class="status-badge full">Full</span> | |
| `window.onDidOpenTerminal` | <span class="status-badge full">Full</span> | |
| `window.onDidCloseTerminal` | <span class="status-badge full">Full</span> | |
| `debug.onDidStartDebugSession` | <span class="status-badge full">Full</span> | |
| `debug.onDidTerminateDebugSession` | <span class="status-badge full">Full</span> | |
| `debug.onDidChangeActiveDebugSession` | <span class="status-badge full">Full</span> | |
| `debug.onDidChangeBreakpoints` | <span class="status-badge partial">Partial</span> | Basic breakpoint change events |
| `extensions.onDidChange` | <span class="status-badge full">Full</span> | |

---

## Activation Events

DSCode supports the following activation events in `package.json`:

| Event | Status | Description |
|---|---|---|
| `onCommand:<command>` | <span class="status-badge full">Full</span> | Activate when a command is executed |
| `onLanguage:<language>` | <span class="status-badge full">Full</span> | Activate when a file of the given language is opened |
| `onView:<viewId>` | <span class="status-badge full">Full</span> | Activate when a view is shown in the sidebar/panel |
| `onUri` | <span class="status-badge planned">Planned</span> | Activate on URI handler |
| `onDebug` | <span class="status-badge full">Full</span> | Activate when a debug session starts |
| `onDebugResolve:<type>` | <span class="status-badge full">Full</span> | Activate to resolve a debug configuration |
| `onDebugAdapterProtocolTracker:<type>` | <span class="status-badge partial">Partial</span> | Basic tracker support |
| `workspaceContains:<glob>` | <span class="status-badge full">Full</span> | Activate when the workspace contains a matching file |
| `onFileSystem:<scheme>` | <span class="status-badge planned">Planned</span> | Virtual file systems not yet supported |
| `onStartupFinished` | <span class="status-badge full">Full</span> | Activate after the editor has finished starting |
| `*` | <span class="status-badge full">Full</span> | Always activate (use sparingly) |

---

## Known Gaps and Workarounds

??? info "Missing: `vscode.notebooks` API"
    **Workaround:** Use the Jupyter extension's REST API or a separate Jupyter server. Notebook support is the highest-priority planned feature.

??? info "Missing: `vscode.authentication` API"
    **Workaround:** Extensions that need OAuth can launch an external browser flow and pass tokens back through a local HTTP callback. DSCode will provide `registerAuthenticationProvider` in a future release.

??? info "Missing: `registerCustomEditorProvider`"
    **Workaround:** Use webview panels (`createWebviewPanel`) for custom UI. Custom editors that need to replace the default text editor will be supported in a future release.

??? info "Missing: `registerFileSystemProvider` (virtual file systems)"
    **Workaround:** Extensions can use real temporary directories and file watchers. Virtual file systems (e.g., for remote or in-memory files) are planned alongside remote development support.

---

## Version Mapping

DSCode versions map to VS Code API levels as follows:

| DSCode Version | VS Code API Level | Key Additions |
|---|---|---|
| 0.1.0 (current) | ~1.85.0 | Core editor, commands, languages, window, workspace |
| 0.2.0 (planned) | ~1.88.0 | Authentication, URI handlers, custom editors |
| 0.3.0 (planned) | ~1.90.0 | Notebooks, comments, l10n |

---

## See Also

- [Extension Development: Getting Started](../extension-development/getting-started.md) -- build your first extension
- [VS Code API Reference](../extension-development/vscode-api-reference.md) -- detailed API documentation
- [Tauri Commands](tauri-commands.md) -- the Rust backend command surface
