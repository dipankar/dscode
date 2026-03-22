/**
 * Built-in VS Code Commands
 *
 * Implements core VS Code commands that extensions expect to be available.
 * These commands are registered during API initialization.
 */

import { ExtensionHostBridge } from '../bridge';
import { Uri } from './uri';
import { LanguagesAPI, Location, CompletionItem, Hover, CodeAction, DocumentSymbol } from './languages';
import { Position, Range, TextDocument } from './textDocument';

export interface BuiltinCommandsOptions {
  bridge: ExtensionHostBridge;
  languagesAPI: LanguagesAPI;
}

/**
 * Registers all built-in VS Code commands
 */
export function registerBuiltinCommands(
  registerCommand: (command: string, handler: (...args: any[]) => any) => { dispose: () => void },
  options: BuiltinCommandsOptions
): void {
  const { bridge, languagesAPI } = options;

  // ==================== File/URI Commands ====================

  /**
   * vscode.open - Opens a resource in the editor
   * @param uri - The URI to open
   * @param options - Optional view column and other options
   */
  registerCommand('vscode.open', async (uri: Uri | string, options?: any) => {
    const uriString = typeof uri === 'string' ? uri : uri.toString();
    console.log(`[BuiltinCommands] vscode.open: ${uriString}`);

    // Check if it's a URL (http/https)
    if (uriString.startsWith('http://') || uriString.startsWith('https://')) {
      await bridge.request('openExternal', { url: uriString });
      return;
    }

    // Otherwise open as file
    await bridge.request('openFile', {
      uri: uriString,
      options
    });
  });

  /**
   * vscode.openFolder - Opens a folder in the workspace
   */
  registerCommand('vscode.openFolder', async (uri?: Uri | string, options?: { forceNewWindow?: boolean }) => {
    const uriString = uri ? (typeof uri === 'string' ? uri : uri.toString()) : undefined;
    console.log(`[BuiltinCommands] vscode.openFolder: ${uriString}`);

    await bridge.request('openFolder', {
      uri: uriString,
      newWindow: options?.forceNewWindow ?? false
    });
  });

  /**
   * vscode.diff - Opens a diff editor
   * @param left - The left-side URI
   * @param right - The right-side URI
   * @param title - Optional title for the diff editor
   */
  registerCommand('vscode.diff', async (left: Uri, right: Uri, title?: string, options?: any) => {
    console.log(`[BuiltinCommands] vscode.diff: ${left.toString()} <-> ${right.toString()}`);

    await bridge.request('openDiff', {
      left: left.toString(),
      right: right.toString(),
      title: title || `${left.path} ↔ ${right.path}`,
      options
    });
  });

  // ==================== Language Feature Commands ====================

  /**
   * vscode.executeDocumentSymbolProvider - Get document symbols
   */
  registerCommand('vscode.executeDocumentSymbolProvider', async (uri: Uri): Promise<DocumentSymbol[]> => {
    console.log(`[BuiltinCommands] vscode.executeDocumentSymbolProvider: ${uri.toString()}`);

    const result = await bridge.request('provideDocumentSymbols', {
      uri: uri.toString(),
      languageId: 'unknown' // Will be determined by the provider
    });

    return (result as any)?.symbols || [];
  });

  /**
   * vscode.executeHoverProvider - Get hover information
   */
  registerCommand('vscode.executeHoverProvider', async (uri: Uri, position: Position): Promise<Hover[]> => {
    console.log(`[BuiltinCommands] vscode.executeHoverProvider: ${uri.toString()} at ${position.line}:${position.character}`);

    const result = await bridge.request('provideHover', {
      uri: uri.toString(),
      languageId: 'unknown',
      document: { uri: uri.toString() },
      position: { line: position.line, character: position.character }
    });

    if (!result) return [];
    return [result as Hover];
  });

  /**
   * vscode.executeDefinitionProvider - Get definition locations
   */
  registerCommand('vscode.executeDefinitionProvider', async (uri: Uri, position: Position): Promise<Location[]> => {
    console.log(`[BuiltinCommands] vscode.executeDefinitionProvider: ${uri.toString()} at ${position.line}:${position.character}`);

    const result = await bridge.request('provideDefinition', {
      uri: uri.toString(),
      languageId: 'unknown',
      document: { uri: uri.toString() },
      position: { line: position.line, character: position.character }
    });

    return (result as any)?.definitions || [];
  });

  /**
   * vscode.executeReferenceProvider - Get reference locations
   */
  registerCommand('vscode.executeReferenceProvider', async (uri: Uri, position: Position): Promise<Location[]> => {
    console.log(`[BuiltinCommands] vscode.executeReferenceProvider: ${uri.toString()} at ${position.line}:${position.character}`);

    const result = await bridge.request('provideReferences', {
      uri: uri.toString(),
      languageId: 'unknown',
      document: { uri: uri.toString() },
      position: { line: position.line, character: position.character },
      includeDeclaration: true
    });

    return (result as any)?.references || [];
  });

  /**
   * vscode.executeCompletionItemProvider - Get completion items
   */
  registerCommand('vscode.executeCompletionItemProvider', async (
    uri: Uri,
    position: Position,
    triggerCharacter?: string
  ): Promise<{ items: CompletionItem[] }> => {
    console.log(`[BuiltinCommands] vscode.executeCompletionItemProvider: ${uri.toString()} at ${position.line}:${position.character}`);

    const result = await bridge.request('provideCompletion', {
      uri: uri.toString(),
      languageId: 'unknown',
      document: { uri: uri.toString() },
      position: { line: position.line, character: position.character },
      context: {
        triggerKind: triggerCharacter ? 2 : 1,
        triggerCharacter
      }
    });

    const items = Array.isArray(result) ? result : (result as any)?.items || [];
    return { items };
  });

  /**
   * vscode.executeCodeActionProvider - Get code actions
   */
  registerCommand('vscode.executeCodeActionProvider', async (
    uri: Uri,
    rangeOrSelection: Range,
    kind?: string
  ): Promise<CodeAction[]> => {
    console.log(`[BuiltinCommands] vscode.executeCodeActionProvider: ${uri.toString()}`);

    const result = await bridge.request('provideCodeActions', {
      uri: uri.toString(),
      languageId: 'unknown',
      document: { uri: uri.toString() },
      range: {
        start: { line: rangeOrSelection.start.line, character: rangeOrSelection.start.character },
        end: { line: rangeOrSelection.end.line, character: rangeOrSelection.end.character }
      },
      diagnostics: []
    });

    return (result as any)?.actions || [];
  });

  /**
   * vscode.executeFormatDocumentProvider - Format a document
   */
  registerCommand('vscode.executeFormatDocumentProvider', async (
    uri: Uri,
    options?: { tabSize?: number; insertSpaces?: boolean }
  ): Promise<{ range: Range; newText: string }[]> => {
    console.log(`[BuiltinCommands] vscode.executeFormatDocumentProvider: ${uri.toString()}`);

    const result = await bridge.request('provideDocumentFormatting', {
      uri: uri.toString(),
      languageId: 'unknown',
      document: { uri: uri.toString() },
      options: {
        tabSize: options?.tabSize ?? 4,
        insertSpaces: options?.insertSpaces ?? true
      }
    });

    return (result as any)?.edits || [];
  });

  // ==================== Workbench Commands ====================

  /**
   * workbench.action.openSettings - Open settings UI
   */
  registerCommand('workbench.action.openSettings', async (query?: string) => {
    console.log(`[BuiltinCommands] workbench.action.openSettings: ${query || ''}`);
    await bridge.send('openSettings', { query });
  });

  /**
   * workbench.action.openSettingsJson - Open settings.json
   */
  registerCommand('workbench.action.openSettingsJson', async () => {
    console.log('[BuiltinCommands] workbench.action.openSettingsJson');
    await bridge.send('openSettingsJson', {});
  });

  /**
   * workbench.action.showCommands - Open command palette
   */
  registerCommand('workbench.action.showCommands', async () => {
    console.log('[BuiltinCommands] workbench.action.showCommands');
    await bridge.send('showCommandPalette', {});
  });

  /**
   * workbench.action.quickOpen - Open quick open (file picker)
   */
  registerCommand('workbench.action.quickOpen', async (prefix?: string) => {
    console.log(`[BuiltinCommands] workbench.action.quickOpen: ${prefix || ''}`);
    await bridge.send('showQuickOpen', { prefix });
  });

  /**
   * workbench.action.files.openFile - Open file dialog
   */
  registerCommand('workbench.action.files.openFile', async () => {
    console.log('[BuiltinCommands] workbench.action.files.openFile');
    await bridge.send('showOpenFileDialog', {});
  });

  /**
   * workbench.action.files.openFolder - Open folder dialog
   */
  registerCommand('workbench.action.files.openFolder', async () => {
    console.log('[BuiltinCommands] workbench.action.files.openFolder');
    await bridge.send('showOpenFolderDialog', {});
  });

  /**
   * workbench.action.files.save - Save current file
   */
  registerCommand('workbench.action.files.save', async () => {
    console.log('[BuiltinCommands] workbench.action.files.save');
    await bridge.send('saveCurrentFile', {});
  });

  /**
   * workbench.action.files.saveAll - Save all files
   */
  registerCommand('workbench.action.files.saveAll', async () => {
    console.log('[BuiltinCommands] workbench.action.files.saveAll');
    await bridge.send('saveAllFiles', {});
  });

  /**
   * workbench.action.closeActiveEditor - Close current editor
   */
  registerCommand('workbench.action.closeActiveEditor', async () => {
    console.log('[BuiltinCommands] workbench.action.closeActiveEditor');
    await bridge.send('closeActiveEditor', {});
  });

  /**
   * workbench.action.closeAllEditors - Close all editors
   */
  registerCommand('workbench.action.closeAllEditors', async () => {
    console.log('[BuiltinCommands] workbench.action.closeAllEditors');
    await bridge.send('closeAllEditors', {});
  });

  /**
   * workbench.action.reloadWindow - Reload the window
   */
  registerCommand('workbench.action.reloadWindow', async () => {
    console.log('[BuiltinCommands] workbench.action.reloadWindow');
    await bridge.send('reloadWindow', {});
  });

  /**
   * workbench.action.toggleSidebarVisibility - Toggle sidebar
   */
  registerCommand('workbench.action.toggleSidebarVisibility', async () => {
    console.log('[BuiltinCommands] workbench.action.toggleSidebarVisibility');
    await bridge.send('toggleSidebar', {});
  });

  /**
   * workbench.action.togglePanel - Toggle panel
   */
  registerCommand('workbench.action.togglePanel', async () => {
    console.log('[BuiltinCommands] workbench.action.togglePanel');
    await bridge.send('togglePanel', {});
  });

  // ==================== Editor Commands ====================

  /**
   * editor.action.triggerSuggest - Trigger autocomplete
   */
  registerCommand('editor.action.triggerSuggest', async () => {
    console.log('[BuiltinCommands] editor.action.triggerSuggest');
    await bridge.send('triggerSuggest', {});
  });

  /**
   * editor.action.formatDocument - Format the current document
   */
  registerCommand('editor.action.formatDocument', async () => {
    console.log('[BuiltinCommands] editor.action.formatDocument');
    await bridge.send('formatDocument', {});
  });

  /**
   * editor.action.goToDefinition - Go to definition
   */
  registerCommand('editor.action.goToDefinition', async () => {
    console.log('[BuiltinCommands] editor.action.goToDefinition');
    await bridge.send('goToDefinition', {});
  });

  /**
   * editor.action.revealDefinition - Reveal definition (peek)
   */
  registerCommand('editor.action.revealDefinition', async () => {
    console.log('[BuiltinCommands] editor.action.revealDefinition');
    await bridge.send('peekDefinition', {});
  });

  /**
   * editor.action.goToReferences - Find all references
   */
  registerCommand('editor.action.goToReferences', async () => {
    console.log('[BuiltinCommands] editor.action.goToReferences');
    await bridge.send('findReferences', {});
  });

  /**
   * editor.action.rename - Rename symbol
   */
  registerCommand('editor.action.rename', async () => {
    console.log('[BuiltinCommands] editor.action.rename');
    await bridge.send('renameSymbol', {});
  });

  /**
   * editor.action.quickFix - Show quick fixes
   */
  registerCommand('editor.action.quickFix', async () => {
    console.log('[BuiltinCommands] editor.action.quickFix');
    await bridge.send('showQuickFixes', {});
  });

  /**
   * editor.action.commentLine - Toggle line comment
   */
  registerCommand('editor.action.commentLine', async () => {
    console.log('[BuiltinCommands] editor.action.commentLine');
    await bridge.send('toggleLineComment', {});
  });

  /**
   * editor.action.blockComment - Toggle block comment
   */
  registerCommand('editor.action.blockComment', async () => {
    console.log('[BuiltinCommands] editor.action.blockComment');
    await bridge.send('toggleBlockComment', {});
  });

  // ==================== Clipboard Commands ====================

  /**
   * editor.action.clipboardCopyAction - Copy
   */
  registerCommand('editor.action.clipboardCopyAction', async () => {
    console.log('[BuiltinCommands] editor.action.clipboardCopyAction');
    await bridge.send('clipboardCopy', {});
  });

  /**
   * editor.action.clipboardCutAction - Cut
   */
  registerCommand('editor.action.clipboardCutAction', async () => {
    console.log('[BuiltinCommands] editor.action.clipboardCutAction');
    await bridge.send('clipboardCut', {});
  });

  /**
   * editor.action.clipboardPasteAction - Paste
   */
  registerCommand('editor.action.clipboardPasteAction', async () => {
    console.log('[BuiltinCommands] editor.action.clipboardPasteAction');
    await bridge.send('clipboardPaste', {});
  });

  // ==================== Terminal Commands ====================

  /**
   * workbench.action.terminal.new - Create new terminal
   */
  registerCommand('workbench.action.terminal.new', async () => {
    console.log('[BuiltinCommands] workbench.action.terminal.new');
    await bridge.send('createTerminal', {});
  });

  /**
   * workbench.action.terminal.toggleTerminal - Toggle terminal panel
   */
  registerCommand('workbench.action.terminal.toggleTerminal', async () => {
    console.log('[BuiltinCommands] workbench.action.terminal.toggleTerminal');
    await bridge.send('toggleTerminal', {});
  });

  // ==================== Utility Commands ====================

  /**
   * setContext - Set a context key for when clauses
   */
  registerCommand('setContext', async (key: string, value: any) => {
    console.log(`[BuiltinCommands] setContext: ${key} = ${value}`);
    await bridge.send('setContext', { key, value });
  });

  /**
   * _executeContributedCommand - Internal command to execute contributed commands
   */
  registerCommand('_executeContributedCommand', async (commandId: string, ...args: any[]) => {
    console.log(`[BuiltinCommands] _executeContributedCommand: ${commandId}`);
    // This is handled internally by the commands system
  });

  console.log('[BuiltinCommands] All built-in commands registered');
}
