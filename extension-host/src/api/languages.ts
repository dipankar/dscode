/**
 * Languages API
 *
 * Provides language-specific features like completion, hover, etc.
 */

import { ExtensionHostBridge } from '../bridge';
import { TextDocument, Position, Range } from './textDocument';

export class CompletionItem {
  constructor(
    public label: string,
    public kind?: number
  ) {}

  detail?: string;
  documentation?: string | { value: string };
  sortText?: string;
  filterText?: string;
  insertText?: string;
  range?: Range;
  command?: { title: string; command: string; arguments?: any[] };
}

export interface CompletionList {
  isIncomplete: boolean;
  items: CompletionItem[];
}

export class Hover {
  constructor(
    public contents: string | string[] | { language: string; value: string }[],
    public range?: Range
  ) {}
}

export interface Definition {
  uri: { path: string; fsPath: string };
  range: Range;
}

export class Location {
  constructor(
    public uri: { path: string; fsPath: string },
    public range: Range
  ) {}
}

export class CodeAction {
  constructor(
    public title: string,
    public kind?: string
  ) {}

  edit?: any; // WorkspaceEdit
  command?: { title: string; command: string; arguments?: any[] };
  diagnostics?: Diagnostic[];
  isPreferred?: boolean;
}

export class Diagnostic {
  constructor(
    public range: Range,
    public message: string,
    public severity: number = 0 // 0=Error, 1=Warning, 2=Info, 3=Hint
  ) {}

  source?: string;
  code?: string | number;
  relatedInformation?: { location: Location; message: string }[];
}

export class DocumentSymbol {
  constructor(
    public name: string,
    public detail: string,
    public kind: number,
    public range: Range,
    public selectionRange: Range
  ) {}

  children?: DocumentSymbol[];
}

// Provider interfaces
export interface CompletionItemProvider {
  provideCompletionItems(
    document: TextDocument,
    position: Position,
    token?: any,
    context?: any
  ): CompletionItem[] | CompletionList | Promise<CompletionItem[] | CompletionList>;
}

export interface HoverProvider {
  provideHover(
    document: TextDocument,
    position: Position,
    token?: any
  ): Hover | null | undefined | Promise<Hover | null | undefined>;
}

export interface DefinitionProvider {
  provideDefinition(
    document: TextDocument,
    position: Position,
    token?: any
  ): Definition | Definition[] | null | undefined | Promise<Definition | Definition[] | null | undefined>;
}

export interface ReferenceProvider {
  provideReferences(
    document: TextDocument,
    position: Position,
    context: { includeDeclaration: boolean },
    token?: any
  ): Location[] | null | undefined | Promise<Location[] | null | undefined>;
}

export interface CodeActionProvider {
  provideCodeActions(
    document: TextDocument,
    range: Range,
    context: { diagnostics: Diagnostic[] },
    token?: any
  ): CodeAction[] | null | undefined | Promise<CodeAction[] | null | undefined>;
}

export interface DocumentSymbolProvider {
  provideDocumentSymbols(
    document: TextDocument,
    token?: any
  ): DocumentSymbol[] | null | undefined | Promise<DocumentSymbol[] | null | undefined>;
}

export interface DocumentFormattingEditProvider {
  provideDocumentFormattingEdits(
    document: TextDocument,
    options: { tabSize: number; insertSpaces: boolean },
    token?: any
  ): { range: Range; newText: string }[] | null | undefined | Promise<{ range: Range; newText: string }[] | null | undefined>;
}

// Additional provider interfaces
export interface RenameProvider {
  provideRenameEdits(document: TextDocument, position: Position, newName: string, token?: any): any;
  prepareRename?(document: TextDocument, position: Position, token?: any): Range | { range: Range; placeholder: string } | null | undefined | Promise<Range | { range: Range; placeholder: string } | null | undefined>;
}

export interface SignatureHelpProvider {
  provideSignatureHelp(document: TextDocument, position: Position, token?: any, context?: any): any;
}

export interface CodeLensProvider {
  provideCodeLenses(document: TextDocument, token?: any): any[];
  resolveCodeLens?(codeLens: any, token?: any): any;
}

export interface DocumentLinkProvider {
  provideDocumentLinks(document: TextDocument, token?: any): any[];
  resolveDocumentLink?(link: any, token?: any): any;
}

export interface DocumentColorProvider {
  provideDocumentColors(document: TextDocument, token?: any): any[];
  provideColorPresentations(color: any, context: any, token?: any): any[];
}

export interface FoldingRangeProvider {
  provideFoldingRanges(document: TextDocument, context: any, token?: any): any[];
}

export interface SelectionRangeProvider {
  provideSelectionRanges(document: TextDocument, positions: Position[], token?: any): any[];
}

export interface CallHierarchyProvider {
  prepareCallHierarchy(document: TextDocument, position: Position, token?: any): any;
  provideCallHierarchyIncomingCalls(item: any, token?: any): any[];
  provideCallHierarchyOutgoingCalls(item: any, token?: any): any[];
}

export interface TypeHierarchyProvider {
  prepareTypeHierarchy(document: TextDocument, position: Position, token?: any): any;
  provideTypeHierarchySupertypes(item: any, token?: any): any[];
  provideTypeHierarchySubtypes(item: any, token?: any): any[];
}

export interface SemanticTokensLegend {
  tokenTypes: string[];
  tokenModifiers: string[];
}

export interface DocumentSemanticTokensProvider {
  provideDocumentSemanticTokens(document: TextDocument, token?: any): any;
  provideDocumentSemanticTokensEdits?(document: TextDocument, previousResultId: string, token?: any): any;
}

export interface InlineCompletionItemProvider {
  provideInlineCompletionItems(document: TextDocument, position: Position, context: any, token?: any): any;
}

export interface DocumentRangeFormattingEditProvider {
  provideDocumentRangeFormattingEdits(document: TextDocument, range: Range, options: any, token?: any): any[];
}

export interface OnTypeFormattingEditProvider {
  provideOnTypeFormattingEdits(document: TextDocument, position: Position, ch: string, options: any, token?: any): any[];
}

export interface WorkspaceSymbolProvider {
  provideWorkspaceSymbols(query: string, token?: any): any[];
  resolveWorkspaceSymbol?(symbol: any, token?: any): any;
}

export interface DocumentHighlightProvider {
  provideDocumentHighlights(document: TextDocument, position: Position, token?: any): any[];
}

export interface ImplementationProvider {
  provideImplementation(document: TextDocument, position: Position, token?: any): Definition | Definition[] | null | undefined | Promise<Definition | Definition[] | null | undefined>;
}

export interface TypeDefinitionProvider {
  provideTypeDefinition(document: TextDocument, position: Position, token?: any): Definition | Definition[] | null | undefined | Promise<Definition | Definition[] | null | undefined>;
}

export interface DeclarationProvider {
  provideDeclaration(document: TextDocument, position: Position, token?: any): Definition | Definition[] | null | undefined | Promise<Definition | Definition[] | null | undefined>;
}

export interface LanguageConfiguration {
  comments?: {
    lineComment?: string;
    blockComment?: [string, string];
  };
  brackets?: [string, string][];
  wordPattern?: RegExp;
  indentationRules?: {
    increaseIndentPattern: RegExp;
    decreaseIndentPattern: RegExp;
  };
}

/**
 * LanguagesAPI manages language providers
 */
export class LanguagesAPI {
  private completionProviders = new Map<string, CompletionItemProvider[]>();
  private hoverProviders = new Map<string, HoverProvider[]>();
  private definitionProviders = new Map<string, DefinitionProvider[]>();
  private referenceProviders = new Map<string, ReferenceProvider[]>();
  private codeActionProviders = new Map<string, CodeActionProvider[]>();
  private symbolProviders = new Map<string, DocumentSymbolProvider[]>();
  private formattingProviders = new Map<string, DocumentFormattingEditProvider[]>();

  constructor(private bridge: ExtensionHostBridge) {
    this.setupMessageHandlers();
  }

  private setupMessageHandlers(): void {
    // Handle requests from main process to trigger providers

    // Completion provider
    this.bridge.on('provideCompletion', async (data: any) => {
      const providers = this.completionProviders.get(data.languageId) || [];
      const results: CompletionItem[] = [];

      for (const provider of providers) {
        try {
          const result = await provider.provideCompletionItems(
            data.document,
            data.position,
            data.token,
            data.context
          );

          if (Array.isArray(result)) {
            results.push(...result);
          } else if (result && 'items' in result) {
            results.push(...result.items);
          }
        } catch (error) {
          console.error('[Languages] Completion provider error:', error);
        }
      }

      this.bridge.send('completionResult', { requestId: data.requestId, items: results });
    });

    // Hover provider
    this.bridge.on('provideHover', async (data: any, respond: Function) => {
      const providers = this.hoverProviders.get(data.languageId) || [];

      for (const provider of providers) {
        try {
          const result = await provider.provideHover(
            data.document,
            new Position(data.position.line, data.position.character),
            data.token
          );

          if (result) {
            respond({
              contents: result.contents,
              range: result.range
            });
            return;
          }
        } catch (error) {
          console.error('[Languages] Hover provider error:', error);
        }
      }

      respond(null);
    });

    // Definition provider
    this.bridge.on('provideDefinition', async (data: any, respond: Function) => {
      const providers = this.definitionProviders.get(data.languageId) || [];

      for (const provider of providers) {
        try {
          const result = await provider.provideDefinition(
            data.document,
            new Position(data.position.line, data.position.character),
            data.token
          );

          if (result) {
            // Normalize result to array
            const definitions = Array.isArray(result) ? result : [result];
            respond({
              definitions: definitions.map(d => ({
                uri: d.uri,
                range: d.range
              }))
            });
            return;
          }
        } catch (error) {
          console.error('[Languages] Definition provider error:', error);
        }
      }

      respond({ definitions: [] });
    });

    // Reference provider
    this.bridge.on('provideReferences', async (data: any, respond: Function) => {
      const providers = this.referenceProviders.get(data.languageId) || [];

      for (const provider of providers) {
        try {
          const result = await provider.provideReferences(
            data.document,
            new Position(data.position.line, data.position.character),
            { includeDeclaration: data.includeDeclaration ?? true },
            data.token
          );

          if (result) {
            respond({
              references: result.map(r => ({
                uri: r.uri,
                range: r.range
              }))
            });
            return;
          }
        } catch (error) {
          console.error('[Languages] Reference provider error:', error);
        }
      }

      respond({ references: [] });
    });

    // Code action provider
    this.bridge.on('provideCodeActions', async (data: any, respond: Function) => {
      const providers = this.codeActionProviders.get(data.languageId) || [];
      const allActions: CodeAction[] = [];

      for (const provider of providers) {
        try {
          const result = await provider.provideCodeActions(
            data.document,
            new Range(
              new Position(data.range.start.line, data.range.start.character),
              new Position(data.range.end.line, data.range.end.character)
            ),
            { diagnostics: data.diagnostics || [] },
            data.token
          );

          if (result) {
            allActions.push(...result);
          }
        } catch (error) {
          console.error('[Languages] Code action provider error:', error);
        }
      }

      respond({
        actions: allActions.map(a => ({
          title: a.title,
          kind: a.kind,
          edit: a.edit,
          command: a.command,
          isPreferred: a.isPreferred
        }))
      });
    });

    // Document symbol provider
    this.bridge.on('provideDocumentSymbols', async (data: any, respond: Function) => {
      const providers = this.symbolProviders.get(data.languageId) || [];

      for (const provider of providers) {
        try {
          const result = await provider.provideDocumentSymbols(
            data.document,
            data.token
          );

          if (result) {
            respond({
              symbols: result.map(s => this.serializeDocumentSymbol(s))
            });
            return;
          }
        } catch (error) {
          console.error('[Languages] Document symbol provider error:', error);
        }
      }

      respond({ symbols: [] });
    });

    // Document formatting provider
    this.bridge.on('provideDocumentFormatting', async (data: any, respond: Function) => {
      const providers = this.formattingProviders.get(data.languageId) || [];

      for (const provider of providers) {
        try {
          const result = await provider.provideDocumentFormattingEdits(
            data.document,
            { tabSize: data.options?.tabSize || 4, insertSpaces: data.options?.insertSpaces ?? true },
            data.token
          );

          if (result) {
            respond({
              edits: result.map(e => ({
                range: e.range,
                newText: e.newText
              }))
            });
            return;
          }
        } catch (error) {
          console.error('[Languages] Formatting provider error:', error);
        }
      }

      respond({ edits: [] });
    });
  }

  /**
   * Serialize a DocumentSymbol to plain object (including children)
   */
  private serializeDocumentSymbol(symbol: DocumentSymbol): any {
    return {
      name: symbol.name,
      detail: symbol.detail,
      kind: symbol.kind,
      range: symbol.range,
      selectionRange: symbol.selectionRange,
      children: symbol.children?.map(c => this.serializeDocumentSymbol(c)) || []
    };
  }

  registerCompletionItemProvider(
    selector: string | string[],
    provider: CompletionItemProvider,
    ...triggerCharacters: string[]
  ): { dispose(): void } {
    const languages = Array.isArray(selector) ? selector : [selector];

    for (const lang of languages) {
      if (!this.completionProviders.has(lang)) {
        this.completionProviders.set(lang, []);
      }
      this.completionProviders.get(lang)!.push(provider);
    }

    this.bridge.send('registerCompletionProvider', {
      languages,
      triggerCharacters
    });

    return {
      dispose: () => {
        for (const lang of languages) {
          const providers = this.completionProviders.get(lang);
          if (providers) {
            const index = providers.indexOf(provider);
            if (index !== -1) {
              providers.splice(index, 1);
            }
          }
        }
      }
    };
  }

  registerHoverProvider(
    selector: string | string[],
    provider: HoverProvider
  ): { dispose(): void } {
    const languages = Array.isArray(selector) ? selector : [selector];

    for (const lang of languages) {
      if (!this.hoverProviders.has(lang)) {
        this.hoverProviders.set(lang, []);
      }
      this.hoverProviders.get(lang)!.push(provider);
    }

    this.bridge.send('registerHoverProvider', { languages });

    return {
      dispose: () => {
        for (const lang of languages) {
          const providers = this.hoverProviders.get(lang);
          if (providers) {
            const index = providers.indexOf(provider);
            if (index !== -1) {
              providers.splice(index, 1);
            }
          }
        }
      }
    };
  }

  registerDefinitionProvider(
    selector: string | string[],
    provider: DefinitionProvider
  ): { dispose(): void } {
    const languages = Array.isArray(selector) ? selector : [selector];

    for (const lang of languages) {
      if (!this.definitionProviders.has(lang)) {
        this.definitionProviders.set(lang, []);
      }
      this.definitionProviders.get(lang)!.push(provider);
    }

    this.bridge.send('registerDefinitionProvider', { languages });

    return {
      dispose: () => {
        for (const lang of languages) {
          const providers = this.definitionProviders.get(lang);
          if (providers) {
            const index = providers.indexOf(provider);
            if (index !== -1) {
              providers.splice(index, 1);
            }
          }
        }
      }
    };
  }

  registerReferenceProvider(
    selector: string | string[],
    provider: ReferenceProvider
  ): { dispose(): void } {
    const languages = Array.isArray(selector) ? selector : [selector];

    for (const lang of languages) {
      if (!this.referenceProviders.has(lang)) {
        this.referenceProviders.set(lang, []);
      }
      this.referenceProviders.get(lang)!.push(provider);
    }

    this.bridge.send('registerReferenceProvider', { languages });

    return {
      dispose: () => {
        for (const lang of languages) {
          const providers = this.referenceProviders.get(lang);
          if (providers) {
            const index = providers.indexOf(provider);
            if (index !== -1) {
              providers.splice(index, 1);
            }
          }
        }
      }
    };
  }

  registerCodeActionsProvider(
    selector: string | string[],
    provider: CodeActionProvider,
    metadata?: any
  ): { dispose(): void } {
    const languages = Array.isArray(selector) ? selector : [selector];

    for (const lang of languages) {
      if (!this.codeActionProviders.has(lang)) {
        this.codeActionProviders.set(lang, []);
      }
      this.codeActionProviders.get(lang)!.push(provider);
    }

    this.bridge.send('registerCodeActionsProvider', { languages, metadata });

    return {
      dispose: () => {
        for (const lang of languages) {
          const providers = this.codeActionProviders.get(lang);
          if (providers) {
            const index = providers.indexOf(provider);
            if (index !== -1) {
              providers.splice(index, 1);
            }
          }
        }
      }
    };
  }

  registerDocumentSymbolProvider(
    selector: string | string[],
    provider: DocumentSymbolProvider
  ): { dispose(): void } {
    const languages = Array.isArray(selector) ? selector : [selector];

    for (const lang of languages) {
      if (!this.symbolProviders.has(lang)) {
        this.symbolProviders.set(lang, []);
      }
      this.symbolProviders.get(lang)!.push(provider);
    }

    this.bridge.send('registerDocumentSymbolProvider', { languages });

    return {
      dispose: () => {
        for (const lang of languages) {
          const providers = this.symbolProviders.get(lang);
          if (providers) {
            const index = providers.indexOf(provider);
            if (index !== -1) {
              providers.splice(index, 1);
            }
          }
        }
      }
    };
  }

  registerDocumentFormattingEditProvider(
    selector: string | string[],
    provider: DocumentFormattingEditProvider
  ): { dispose(): void } {
    const languages = Array.isArray(selector) ? selector : [selector];

    for (const lang of languages) {
      if (!this.formattingProviders.has(lang)) {
        this.formattingProviders.set(lang, []);
      }
      this.formattingProviders.get(lang)!.push(provider);
    }

    this.bridge.send('registerFormattingProvider', { languages });

    return {
      dispose: () => {
        for (const lang of languages) {
          const providers = this.formattingProviders.get(lang);
          if (providers) {
            const index = providers.indexOf(provider);
            if (index !== -1) {
              providers.splice(index, 1);
            }
          }
        }
      }
    };
  }

  // Diagnostics collection
  createDiagnosticCollection(name?: string): DiagnosticCollection {
    return new DiagnosticCollection(this.bridge, name);
  }

  // Additional Language Providers
  registerRenameProvider(
    selector: string | string[],
    provider: RenameProvider
  ): { dispose(): void } {
    this.bridge.send('registerRenameProvider', { selector });
    return { dispose: () => {} };
  }

  registerSignatureHelpProvider(
    selector: string | string[],
    provider: SignatureHelpProvider,
    ...triggerCharacters: string[]
  ): { dispose(): void } {
    this.bridge.send('registerSignatureHelpProvider', { selector, triggerCharacters });
    return { dispose: () => {} };
  }

  registerCodeLensProvider(
    selector: string | string[],
    provider: CodeLensProvider
  ): { dispose(): void } {
    this.bridge.send('registerCodeLensProvider', { selector });
    return { dispose: () => {} };
  }

  registerDocumentLinkProvider(
    selector: string | string[],
    provider: DocumentLinkProvider
  ): { dispose(): void } {
    this.bridge.send('registerDocumentLinkProvider', { selector });
    return { dispose: () => {} };
  }

  registerColorProvider(
    selector: string | string[],
    provider: DocumentColorProvider
  ): { dispose(): void } {
    this.bridge.send('registerColorProvider', { selector });
    return { dispose: () => {} };
  }

  registerFoldingRangeProvider(
    selector: string | string[],
    provider: FoldingRangeProvider
  ): { dispose(): void } {
    this.bridge.send('registerFoldingRangeProvider', { selector });
    return { dispose: () => {} };
  }

  registerSelectionRangeProvider(
    selector: string | string[],
    provider: SelectionRangeProvider
  ): { dispose(): void } {
    this.bridge.send('registerSelectionRangeProvider', { selector });
    return { dispose: () => {} };
  }

  registerCallHierarchyProvider(
    selector: string | string[],
    provider: CallHierarchyProvider
  ): { dispose(): void } {
    this.bridge.send('registerCallHierarchyProvider', { selector });
    return { dispose: () => {} };
  }

  registerTypeHierarchyProvider(
    selector: string | string[],
    provider: TypeHierarchyProvider
  ): { dispose(): void } {
    this.bridge.send('registerTypeHierarchyProvider', { selector });
    return { dispose: () => {} };
  }

  registerDocumentSemanticTokensProvider(
    selector: string | string[],
    provider: DocumentSemanticTokensProvider,
    legend: SemanticTokensLegend
  ): { dispose(): void } {
    this.bridge.send('registerSemanticTokensProvider', { selector, legend });
    return { dispose: () => {} };
  }

  registerInlineCompletionItemProvider(
    selector: string | string[],
    provider: InlineCompletionItemProvider
  ): { dispose(): void } {
    this.bridge.send('registerInlineCompletionProvider', { selector });
    return { dispose: () => {} };
  }

  registerDocumentRangeFormattingEditProvider(
    selector: string | string[],
    provider: DocumentRangeFormattingEditProvider
  ): { dispose(): void } {
    this.bridge.send('registerRangeFormattingProvider', { selector });
    return { dispose: () => {} };
  }

  registerOnTypeFormattingEditProvider(
    selector: string | string[],
    provider: OnTypeFormattingEditProvider,
    firstTriggerCharacter: string,
    ...moreTriggerCharacters: string[]
  ): { dispose(): void } {
    this.bridge.send('registerOnTypeFormattingProvider', {
      selector,
      triggerCharacters: [firstTriggerCharacter, ...moreTriggerCharacters]
    });
    return { dispose: () => {} };
  }

  registerWorkspaceSymbolProvider(provider: WorkspaceSymbolProvider): { dispose(): void } {
    this.bridge.send('registerWorkspaceSymbolProvider', {});
    return { dispose: () => {} };
  }

  registerDocumentHighlightProvider(
    selector: string | string[],
    provider: DocumentHighlightProvider
  ): { dispose(): void } {
    this.bridge.send('registerDocumentHighlightProvider', { selector });
    return { dispose: () => {} };
  }

  registerImplementationProvider(
    selector: string | string[],
    provider: ImplementationProvider
  ): { dispose(): void } {
    this.bridge.send('registerImplementationProvider', { selector });
    return { dispose: () => {} };
  }

  registerTypeDefinitionProvider(
    selector: string | string[],
    provider: TypeDefinitionProvider
  ): { dispose(): void } {
    this.bridge.send('registerTypeDefinitionProvider', { selector });
    return { dispose: () => {} };
  }

  registerDeclarationProvider(
    selector: string | string[],
    provider: DeclarationProvider
  ): { dispose(): void } {
    this.bridge.send('registerDeclarationProvider', { selector });
    return { dispose: () => {} };
  }

  setLanguageConfiguration(
    language: string,
    configuration: LanguageConfiguration
  ): { dispose(): void } {
    this.bridge.send('setLanguageConfiguration', { language, configuration });
    return { dispose: () => {} };
  }
}

/**
 * DiagnosticCollection for managing diagnostics
 */
export class DiagnosticCollection {
  private diagnostics = new Map<string, Diagnostic[]>();

  constructor(private bridge: ExtensionHostBridge, private name?: string) {}

  set(uri: string | { path: string; fsPath: string }, diagnostics: Diagnostic[]): void {
    const uriStr = typeof uri === 'string' ? uri : uri.fsPath;
    this.diagnostics.set(uriStr, diagnostics);

    this.bridge.send('updateDiagnostics', {
      collection: this.name,
      uri: uriStr,
      diagnostics
    });
  }

  delete(uri: string | { path: string; fsPath: string }): void {
    const uriStr = typeof uri === 'string' ? uri : uri.fsPath;
    this.diagnostics.delete(uriStr);

    this.bridge.send('clearDiagnostics', {
      collection: this.name,
      uri: uriStr
    });
  }

  clear(): void {
    this.diagnostics.clear();

    this.bridge.send('clearAllDiagnostics', {
      collection: this.name
    });
  }

  forEach(callback: (uri: string, diagnostics: Diagnostic[], collection: DiagnosticCollection) => void): void {
    this.diagnostics.forEach((diagnostics, uri) => {
      callback(uri, diagnostics, this);
    });
  }

  get(uri: string | { path: string; fsPath: string }): Diagnostic[] | undefined {
    const uriStr = typeof uri === 'string' ? uri : uri.fsPath;
    return this.diagnostics.get(uriStr);
  }

  has(uri: string | { path: string; fsPath: string }): boolean {
    const uriStr = typeof uri === 'string' ? uri : uri.fsPath;
    return this.diagnostics.has(uriStr);
  }

  dispose(): void {
    this.clear();
    this.diagnostics.clear();
  }
}
