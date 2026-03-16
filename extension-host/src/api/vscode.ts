/**
 * VS Code API
 *
 * Main entry point for the vscode module that extensions import
 */

import { ExtensionHostBridge } from '../bridge';
import { WindowAPI, OutputChannel } from './window';
import { CommandsAPI } from './commands';
import { WorkspaceAPI } from './workspace';
import { TextDocumentAPI, TextDocument, Position, Range } from './textDocument';
import { TextEditorAPI, TextEditor, Selection, TextEdit, WorkspaceEdit, SnippetString, TextEditorRevealType, TextEditorCursorStyle, TextEditorLineNumbersStyle } from './textEditor';
import { LanguagesAPI, CompletionItem, Hover, Definition, Location, CodeAction, Diagnostic, DocumentSymbol } from './languages';
import { UIAPI, StatusBarItem, StatusBarAlignment, TreeView, TreeDataProvider, TreeItem, TreeItemCollapsibleState, WebviewPanel, Webview, WebviewOptions, WebviewPanelOptions } from './ui';
import { TerminalAPI, Terminal, TerminalOptions, Pseudoterminal, TerminalDimensions, TerminalExitReason, TerminalExitStatus } from './terminal';
import { EnvironmentAPIImpl, ExtensionsAPI, EnvironmentAPI, Extension, ExtensionKind, UIKind, LogLevel } from './env';
import { DebugAPI, TasksAPI, DebugSession, DebugConfiguration, Task, TaskProvider, TaskDefinition, TaskGroup, ProcessExecution, ShellExecution, CustomExecution, TaskRevealKind, TaskPanelKind, Breakpoint, SourceBreakpoint, FunctionBreakpoint } from './debug';
import { FileSystemAPI, FileSystemWatcher } from './fs';
import { EventEmitter, Event, Disposable } from './events';
import { Uri } from './uri';
import { SCMAPI, SourceControl, SourceControlResourceGroup, SourceControlResourceState, SourceControlInputBox } from './scm';

// Global API instances
let windowAPI: WindowAPI;
let commandsAPI: CommandsAPI;
let workspaceAPI: WorkspaceAPI;
let textDocumentAPI: TextDocumentAPI;
let textEditorAPI: TextEditorAPI;
let languagesAPI: LanguagesAPI;
let uiAPI: UIAPI;
let terminalAPI: TerminalAPI;
let environmentAPI: EnvironmentAPIImpl;
let extensionsAPI: ExtensionsAPI;
let debugAPI: DebugAPI;
let tasksAPI: TasksAPI;
let fileSystemAPI: FileSystemAPI;
let scmAPI: SCMAPI;

/**
 * Initialize the vscode API with the bridge
 */
export function initializeAPI(bridge: ExtensionHostBridge) {
  windowAPI = new WindowAPI(bridge);
  commandsAPI = new CommandsAPI(bridge);
  workspaceAPI = new WorkspaceAPI(bridge);
  textDocumentAPI = new TextDocumentAPI(bridge);
  textEditorAPI = new TextEditorAPI(bridge);
  languagesAPI = new LanguagesAPI(bridge);
  uiAPI = new UIAPI(bridge);
  terminalAPI = new TerminalAPI(bridge);
  environmentAPI = new EnvironmentAPIImpl(bridge);
  extensionsAPI = new ExtensionsAPI(bridge);
  debugAPI = new DebugAPI(bridge);
  tasksAPI = new TasksAPI(bridge);
  fileSystemAPI = new FileSystemAPI(bridge);
  scmAPI = new SCMAPI(bridge);
}

/**
 * vscode.window namespace
 */
export const window = {
  get activeTextEditor() {
    return textEditorAPI.getActiveEditor();
  },
  get visibleTextEditors() {
    return textEditorAPI.getAllEditors();
  },
  get showInformationMessage() {
    return windowAPI.showInformationMessage.bind(windowAPI);
  },
  get showWarningMessage() {
    return windowAPI.showWarningMessage.bind(windowAPI);
  },
  get showErrorMessage() {
    return windowAPI.showErrorMessage.bind(windowAPI);
  },
  get showInputBox() {
    return windowAPI.showInputBox.bind(windowAPI);
  },
  get showQuickPick() {
    return windowAPI.showQuickPick.bind(windowAPI);
  },
  get createOutputChannel() {
    return windowAPI.createOutputChannel.bind(windowAPI);
  },
  get setStatusBarMessage() {
    return windowAPI.setStatusBarMessage.bind(windowAPI);
  },
  // UI Components
  get createStatusBarItem() {
    return uiAPI.createStatusBarItem.bind(uiAPI);
  },
  get createTreeView() {
    return uiAPI.createTreeView.bind(uiAPI);
  },
  get createWebviewPanel() {
    return uiAPI.createWebviewPanel.bind(uiAPI);
  },
  get registerWebviewPanelSerializer() {
    return uiAPI.registerWebviewPanelSerializer.bind(uiAPI);
  },
  // Terminal
  get terminals() {
    return terminalAPI.terminals;
  },
  get activeTerminal() {
    return terminalAPI.activeTerminal;
  },
  get createTerminal() {
    return terminalAPI.createTerminal.bind(terminalAPI);
  },
  get onDidOpenTerminal() {
    return terminalAPI.onDidOpenTerminal;
  },
  get onDidCloseTerminal() {
    return terminalAPI.onDidCloseTerminal;
  },
  get onDidChangeActiveTerminal() {
    return terminalAPI.onDidChangeActiveTerminal;
  },
};

/**
 * vscode.commands namespace
 */
export const commands = {
  get registerCommand() {
    return commandsAPI.registerCommand.bind(commandsAPI);
  },
  get registerTextEditorCommand() {
    return commandsAPI.registerTextEditorCommand.bind(commandsAPI);
  },
  get executeCommand() {
    return commandsAPI.executeCommand.bind(commandsAPI);
  },
  get getCommands() {
    return commandsAPI.getCommands.bind(commandsAPI);
  },
};

/**
 * vscode.workspace namespace
 */
export const workspace = {
  get workspaceFolders() {
    return workspaceAPI.folders;
  },
  get name() {
    return workspaceAPI.name;
  },
  get rootPath() {
    return workspaceAPI.rootPath;
  },
  get findFiles() {
    return workspaceAPI.findFiles.bind(workspaceAPI);
  },
  get openTextDocument() {
    return workspaceAPI.openTextDocument.bind(workspaceAPI);
  },
  get saveTextDocument() {
    return workspaceAPI.saveTextDocument.bind(workspaceAPI);
  },
  get getConfiguration() {
    return workspaceAPI.getConfiguration.bind(workspaceAPI);
  },
  get createFileSystemWatcher() {
    return fileSystemAPI.createFileSystemWatcher.bind(fileSystemAPI);
  },
};

/**
 * vscode.languages namespace
 */
export const languages = {
  get registerCompletionItemProvider() {
    return languagesAPI.registerCompletionItemProvider.bind(languagesAPI);
  },
  get registerHoverProvider() {
    return languagesAPI.registerHoverProvider.bind(languagesAPI);
  },
  get registerDefinitionProvider() {
    return languagesAPI.registerDefinitionProvider.bind(languagesAPI);
  },
  get registerReferenceProvider() {
    return languagesAPI.registerReferenceProvider.bind(languagesAPI);
  },
  get registerCodeActionsProvider() {
    return languagesAPI.registerCodeActionsProvider.bind(languagesAPI);
  },
  get registerDocumentSymbolProvider() {
    return languagesAPI.registerDocumentSymbolProvider.bind(languagesAPI);
  },
  get registerDocumentFormattingEditProvider() {
    return languagesAPI.registerDocumentFormattingEditProvider.bind(languagesAPI);
  },
  get createDiagnosticCollection() {
    return languagesAPI.createDiagnosticCollection.bind(languagesAPI);
  },
  // Additional providers
  get registerRenameProvider() {
    return languagesAPI.registerRenameProvider.bind(languagesAPI);
  },
  get registerSignatureHelpProvider() {
    return languagesAPI.registerSignatureHelpProvider.bind(languagesAPI);
  },
  get registerCodeLensProvider() {
    return languagesAPI.registerCodeLensProvider.bind(languagesAPI);
  },
  get registerDocumentLinkProvider() {
    return languagesAPI.registerDocumentLinkProvider.bind(languagesAPI);
  },
  get registerColorProvider() {
    return languagesAPI.registerColorProvider.bind(languagesAPI);
  },
  get registerFoldingRangeProvider() {
    return languagesAPI.registerFoldingRangeProvider.bind(languagesAPI);
  },
  get registerSelectionRangeProvider() {
    return languagesAPI.registerSelectionRangeProvider.bind(languagesAPI);
  },
  get registerCallHierarchyProvider() {
    return languagesAPI.registerCallHierarchyProvider.bind(languagesAPI);
  },
  get registerTypeHierarchyProvider() {
    return languagesAPI.registerTypeHierarchyProvider.bind(languagesAPI);
  },
  get registerDocumentSemanticTokensProvider() {
    return languagesAPI.registerDocumentSemanticTokensProvider.bind(languagesAPI);
  },
  get registerInlineCompletionItemProvider() {
    return languagesAPI.registerInlineCompletionItemProvider.bind(languagesAPI);
  },
  get registerDocumentRangeFormattingEditProvider() {
    return languagesAPI.registerDocumentRangeFormattingEditProvider.bind(languagesAPI);
  },
  get registerOnTypeFormattingEditProvider() {
    return languagesAPI.registerOnTypeFormattingEditProvider.bind(languagesAPI);
  },
  get registerWorkspaceSymbolProvider() {
    return languagesAPI.registerWorkspaceSymbolProvider.bind(languagesAPI);
  },
  get registerDocumentHighlightProvider() {
    return languagesAPI.registerDocumentHighlightProvider.bind(languagesAPI);
  },
  get registerImplementationProvider() {
    return languagesAPI.registerImplementationProvider.bind(languagesAPI);
  },
  get registerTypeDefinitionProvider() {
    return languagesAPI.registerTypeDefinitionProvider.bind(languagesAPI);
  },
  get registerDeclarationProvider() {
    return languagesAPI.registerDeclarationProvider.bind(languagesAPI);
  },
  get setLanguageConfiguration() {
    return languagesAPI.setLanguageConfiguration.bind(languagesAPI);
  },
};

/**
 * vscode.env namespace
 */
export const env: EnvironmentAPI = {
  get appName() {
    return environmentAPI.appName;
  },
  get appRoot() {
    return environmentAPI.appRoot;
  },
  get appHost() {
    return environmentAPI.appHost;
  },
  get language() {
    return environmentAPI.language;
  },
  get clipboard() {
    return environmentAPI.clipboard;
  },
  get machineId() {
    return environmentAPI.machineId;
  },
  get sessionId() {
    return environmentAPI.sessionId;
  },
  get remoteName() {
    return environmentAPI.remoteName;
  },
  get shell() {
    return environmentAPI.shell;
  },
  get uiKind() {
    return environmentAPI.uiKind;
  },
  get uriScheme() {
    return environmentAPI.uriScheme;
  },
  get isNewAppInstall() {
    return environmentAPI.isNewAppInstall;
  },
  get isTelemetryEnabled() {
    return environmentAPI.isTelemetryEnabled;
  },
  get onDidChangeTelemetryEnabled() {
    return environmentAPI.onDidChangeTelemetryEnabled;
  },
  get logLevel() {
    return environmentAPI.logLevel;
  },
  get onDidChangeLogLevel() {
    return environmentAPI.onDidChangeLogLevel;
  },
  asExternalUri(target: any) {
    return environmentAPI.asExternalUri(target);
  },
  openExternal(target: any) {
    return environmentAPI.openExternal(target);
  },
};

/**
 * vscode.extensions namespace
 */
export const extensions = {
  get all() {
    return extensionsAPI.all;
  },
  get onDidChange() {
    return extensionsAPI.onDidChange;
  },
  getExtension<T = any>(extensionId: string) {
    return extensionsAPI.getExtension<T>(extensionId);
  },
};

/**
 * vscode.debug namespace
 */
export const debug = {
  get activeDebugSession() {
    return debugAPI.activeDebugSession;
  },
  get activeDebugConsole() {
    return debugAPI.activeDebugConsole;
  },
  get breakpoints() {
    return debugAPI.breakpoints;
  },
  get onDidChangeActiveDebugSession() {
    return debugAPI.onDidChangeActiveDebugSession;
  },
  get onDidStartDebugSession() {
    return debugAPI.onDidStartDebugSession;
  },
  get onDidReceiveDebugSessionCustomEvent() {
    return debugAPI.onDidReceiveDebugSessionCustomEvent;
  },
  get onDidTerminateDebugSession() {
    return debugAPI.onDidTerminateDebugSession;
  },
  get onDidChangeBreakpoints() {
    return debugAPI.onDidChangeBreakpoints;
  },
  get registerDebugConfigurationProvider() {
    return debugAPI.registerDebugConfigurationProvider.bind(debugAPI);
  },
  get registerDebugAdapterDescriptorFactory() {
    return debugAPI.registerDebugAdapterDescriptorFactory.bind(debugAPI);
  },
  get registerDebugAdapterTrackerFactory() {
    return debugAPI.registerDebugAdapterTrackerFactory.bind(debugAPI);
  },
  get startDebugging() {
    return debugAPI.startDebugging.bind(debugAPI);
  },
  get stopDebugging() {
    return debugAPI.stopDebugging.bind(debugAPI);
  },
  get addBreakpoints() {
    return debugAPI.addBreakpoints.bind(debugAPI);
  },
  get removeBreakpoints() {
    return debugAPI.removeBreakpoints.bind(debugAPI);
  },
  get asDebugSourceUri() {
    return debugAPI.asDebugSourceUri.bind(debugAPI);
  },
};

/**
 * vscode.tasks namespace
 */
export const tasks = {
  get onDidStartTask() {
    return tasksAPI.onDidStartTask;
  },
  get onDidEndTask() {
    return tasksAPI.onDidEndTask;
  },
  get onDidStartTaskProcess() {
    return tasksAPI.onDidStartTaskProcess;
  },
  get onDidEndTaskProcess() {
    return tasksAPI.onDidEndTaskProcess;
  },
  get fetchTasks() {
    return tasksAPI.fetchTasks.bind(tasksAPI);
  },
  get executeTask() {
    return tasksAPI.executeTask.bind(tasksAPI);
  },
  get registerTaskProvider() {
    return tasksAPI.registerTaskProvider.bind(tasksAPI);
  },
};

/**
 * vscode.scm namespace
 */
export const scm = {
  get onDidChangeActiveProvider() {
    return scmAPI.onDidChangeActiveProvider;
  },
  get createSourceControl() {
    return scmAPI.createSourceControl.bind(scmAPI);
  },
};

/**
 * Export types
 */
export {
  // Base types
  Event,
  EventEmitter,
  Disposable,
  Uri,
  // Window/UI types
  OutputChannel,
  StatusBarItem,
  StatusBarAlignment,
  TreeView,
  TreeDataProvider,
  TreeItem,
  TreeItemCollapsibleState,
  WebviewPanel,
  Webview,
  WebviewOptions,
  WebviewPanelOptions,
  // Document/Editor types
  TextDocument,
  TextEditor,
  Position,
  Range,
  Selection,
  TextEdit,
  WorkspaceEdit,
  SnippetString,
  TextEditorRevealType,
  TextEditorCursorStyle,
  TextEditorLineNumbersStyle,
  // Language types
  CompletionItem,
  Hover,
  Definition,
  Location,
  CodeAction,
  Diagnostic,
  DocumentSymbol,
  // Terminal types
  Terminal,
  TerminalOptions,
  Pseudoterminal,
  TerminalDimensions,
  TerminalExitStatus,
  TerminalExitReason,
  // Environment types
  Extension,
  ExtensionKind,
  UIKind,
  LogLevel,
  // Debug types
  DebugSession,
  DebugConfiguration,
  Breakpoint,
  SourceBreakpoint,
  FunctionBreakpoint,
  // Task types
  Task,
  TaskProvider,
  TaskDefinition,
  TaskGroup,
  ProcessExecution,
  ShellExecution,
  CustomExecution,
  TaskRevealKind,
  TaskPanelKind,
  // FileSystem types
  FileSystemWatcher,
  // SCM types
  SourceControl,
  SourceControlResourceGroup,
  SourceControlResourceState,
  SourceControlInputBox,
};

// Version info
export const version = '1.80.0'; // Pretend we're VS Code 1.80

// Extension context type
export interface ExtensionContext {
  subscriptions: { dispose(): any }[];
  extensionPath: string;
  globalState: any;
  workspaceState: any;
  asAbsolutePath(relativePath: string): string;
}

// Enums for compatibility
export enum DiagnosticSeverity {
  Error = 0,
  Warning = 1,
  Information = 2,
  Hint = 3
}

export enum CompletionItemKind {
  Text = 0,
  Method = 1,
  Function = 2,
  Constructor = 3,
  Field = 4,
  Variable = 5,
  Class = 6,
  Interface = 7,
  Module = 8,
  Property = 9,
  Unit = 10,
  Value = 11,
  Enum = 12,
  Keyword = 13,
  Snippet = 14,
  Color = 15,
  File = 16,
  Reference = 17,
  Folder = 18,
  EnumMember = 19,
  Constant = 20,
  Struct = 21,
  Event = 22,
  Operator = 23,
  TypeParameter = 24
}

export enum SymbolKind {
  File = 0,
  Module = 1,
  Namespace = 2,
  Package = 3,
  Class = 4,
  Method = 5,
  Property = 6,
  Field = 7,
  Constructor = 8,
  Enum = 9,
  Interface = 10,
  Function = 11,
  Variable = 12,
  Constant = 13,
  String = 14,
  Number = 15,
  Boolean = 16,
  Array = 17,
  Object = 18,
  Key = 19,
  Null = 20,
  EnumMember = 21,
  Struct = 22,
  Event = 23,
  Operator = 24,
  TypeParameter = 25
}
