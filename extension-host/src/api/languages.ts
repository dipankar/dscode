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
  ):
    | Definition
    | Definition[]
    | null
    | undefined
    | Promise<Definition | Definition[] | null | undefined>;
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
  ):
    | { range: Range; newText: string }[]
    | null
    | undefined
    | Promise<{ range: Range; newText: string }[] | null | undefined>;
}

// Additional provider interfaces
export interface RenameProvider {
  provideRenameEdits(document: TextDocument, position: Position, newName: string, token?: any): any;
  prepareRename?(
    document: TextDocument,
    position: Position,
    token?: any
  ):
    | Range
    | { range: Range; placeholder: string }
    | null
    | undefined
    | Promise<Range | { range: Range; placeholder: string } | null | undefined>;
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
  provideDocumentSemanticTokensEdits?(
    document: TextDocument,
    previousResultId: string,
    token?: any
  ): any;
}

export interface InlineCompletionItemProvider {
  provideInlineCompletionItems(
    document: TextDocument,
    position: Position,
    context: any,
    token?: any
  ): any;
}

export interface DocumentRangeFormattingEditProvider {
  provideDocumentRangeFormattingEdits(
    document: TextDocument,
    range: Range,
    options: any,
    token?: any
  ): any[];
}

export interface OnTypeFormattingEditProvider {
  provideOnTypeFormattingEdits(
    document: TextDocument,
    position: Position,
    ch: string,
    options: any,
    token?: any
  ): any[];
}

export interface WorkspaceSymbolProvider {
  provideWorkspaceSymbols(query: string, token?: any): any[];
  resolveWorkspaceSymbol?(symbol: any, token?: any): any;
}

export interface DocumentHighlightProvider {
  provideDocumentHighlights(document: TextDocument, position: Position, token?: any): any[];
}

export interface ImplementationProvider {
  provideImplementation(
    document: TextDocument,
    position: Position,
    token?: any
  ):
    | Definition
    | Definition[]
    | null
    | undefined
    | Promise<Definition | Definition[] | null | undefined>;
}

export interface TypeDefinitionProvider {
  provideTypeDefinition(
    document: TextDocument,
    position: Position,
    token?: any
  ):
    | Definition
    | Definition[]
    | null
    | undefined
    | Promise<Definition | Definition[] | null | undefined>;
}

export interface DeclarationProvider {
  provideDeclaration(
    document: TextDocument,
    position: Position,
    token?: any
  ):
    | Definition
    | Definition[]
    | null
    | undefined
    | Promise<Definition | Definition[] | null | undefined>;
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
  private renameProviders = new Map<string, RenameProvider[]>();
  private signatureHelpProviders = new Map<string, SignatureHelpProvider[]>();
  private codeLensProviders = new Map<string, CodeLensProvider[]>();
  private documentColorProviders = new Map<string, DocumentColorProvider[]>();
  private foldingRangeProviders = new Map<string, FoldingRangeProvider[]>();
  private selectionRangeProviders = new Map<string, SelectionRangeProvider[]>();
  private workspaceSymbolProviders: WorkspaceSymbolProvider[] = [];
  private documentHighlightProviders = new Map<string, DocumentHighlightProvider[]>();
  private rangeFormattingProviders = new Map<string, DocumentRangeFormattingEditProvider[]>();
  private onTypeFormattingProviders = new Map<string, OnTypeFormattingEditProvider[]>();
  private semanticTokensProviders = new Map<string, DocumentSemanticTokensProvider[]>();
  private implementationProviders = new Map<string, ImplementationProvider[]>();
  private typeDefinitionProviders = new Map<string, TypeDefinitionProvider[]>();
  private declarationProviders = new Map<string, DeclarationProvider[]>();
  private documentLinkProviders = new Map<string, DocumentLinkProvider[]>();
  private inlineCompletionProviders = new Map<string, InlineCompletionItemProvider[]>();

  constructor(private bridge: ExtensionHostBridge) {
    this.setupMessageHandlers();
  }

  private getLanguages(selector: string | string[]): string[] {
    return Array.isArray(selector) ? selector : [selector];
  }

  private addProviders<T>(
    store: Map<string, T[]>,
    selector: string | string[],
    provider: T
  ): string[] {
    const languages = this.getLanguages(selector);
    for (const language of languages) {
      if (!store.has(language)) {
        store.set(language, []);
      }
      store.get(language)!.push(provider);
    }
    return languages;
  }

  private removeProviders<T>(store: Map<string, T[]>, languages: string[], provider: T): void {
    for (const language of languages) {
      const providers = store.get(language);
      if (!providers) {
        continue;
      }

      const index = providers.indexOf(provider);
      if (index !== -1) {
        providers.splice(index, 1);
      }

      if (providers.length === 0) {
        store.delete(language);
      }
    }
  }

  private createTextDocument(rawDocument: any, languageId: string): TextDocument {
    const rawUri = rawDocument?.uri;
    const uriString = typeof rawUri === 'string' ? rawUri : rawUri?.fsPath || rawUri?.path || '';
    const fsPath = uriString.replace(/^file:\/\//, '');

    return new TextDocument(
      this.bridge,
      {
        path: rawUri?.path || fsPath,
        fsPath,
        scheme: rawUri?.scheme || 'file',
      },
      fsPath,
      rawDocument?.languageId || languageId || 'plaintext',
      rawDocument?.version || 1,
      rawDocument?.text || '',
      Boolean(rawDocument?.isDirty),
      Boolean(rawDocument?.isClosed),
      rawDocument?.eol || 1
    );
  }

  private createPosition(rawPosition: any): Position {
    return new Position(rawPosition?.line || 0, rawPosition?.character || 0);
  }

  private createRange(rawRange: any): Range {
    return new Range(this.createPosition(rawRange?.start), this.createPosition(rawRange?.end));
  }

  private async getDocumentFromRequest(data: any): Promise<TextDocument> {
    const document = data?.document || {};
    if (typeof document.text === 'string') {
      return this.createTextDocument(document, data?.languageId);
    }

    const rawUri = document.uri;
    const uriString =
      typeof rawUri === 'string' ? rawUri : rawUri?.fsPath || rawUri?.path || data?.uri || '';

    if (!uriString) {
      return this.createTextDocument(document, data?.languageId);
    }

    try {
      const opened = (await this.bridge.request('openTextDocument', { path: uriString })) as any;
      return this.createTextDocument(
        {
          ...document,
          ...opened,
          uri: opened?.uri || uriString,
          languageId: opened?.languageId || data?.languageId,
          text: opened?.text || document.text || '',
        },
        data?.languageId
      );
    } catch {
      return this.createTextDocument(document, data?.languageId);
    }
  }

  private setupMessageHandlers(): void {
    // Handle requests from main process to trigger providers

    // Completion provider
    this.bridge.on('provideCompletion', async (data: any) => {
      const providers = this.completionProviders.get(data.languageId) || [];
      const results: CompletionItem[] = [];
      const document = await this.getDocumentFromRequest(data);
      const position = this.createPosition(data.position);

      for (const provider of providers) {
        try {
          const result = await provider.provideCompletionItems(
            document,
            position,
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
      const document = await this.getDocumentFromRequest(data);
      const position = this.createPosition(data.position);

      for (const provider of providers) {
        try {
          const result = await provider.provideHover(document, position, data.token);

          if (result) {
            respond({
              contents: result.contents,
              range: result.range,
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
      const document = await this.getDocumentFromRequest(data);
      const position = this.createPosition(data.position);

      for (const provider of providers) {
        try {
          const result = await provider.provideDefinition(document, position, data.token);

          if (result) {
            // Normalize result to array
            const definitions = Array.isArray(result) ? result : [result];
            respond({
              definitions: definitions.map((d) => ({
                uri: d.uri,
                range: d.range,
              })),
            });
            return;
          }
        } catch (error) {
          console.error('[Languages] Definition provider error:', error);
        }
      }

      respond({ definitions: [] });
    });

    this.bridge.on('provideImplementation', async (data: any, respond: Function) => {
      const providers = this.implementationProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);
      const position = this.createPosition(data.position);

      for (const provider of providers) {
        try {
          const result = await provider.provideImplementation(document, position, data.token);
          if (result) {
            const definitions = Array.isArray(result) ? result : [result];
            respond(definitions.map((d) => ({ uri: d.uri, range: d.range })));
            return;
          }
        } catch (error) {
          console.error('[Languages] Implementation provider error:', error);
        }
      }

      respond([]);
    });

    this.bridge.on('provideTypeDefinition', async (data: any, respond: Function) => {
      const providers = this.typeDefinitionProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);
      const position = this.createPosition(data.position);

      for (const provider of providers) {
        try {
          const result = await provider.provideTypeDefinition(document, position, data.token);
          if (result) {
            const definitions = Array.isArray(result) ? result : [result];
            respond(definitions.map((d) => ({ uri: d.uri, range: d.range })));
            return;
          }
        } catch (error) {
          console.error('[Languages] Type definition provider error:', error);
        }
      }

      respond([]);
    });

    this.bridge.on('provideDeclaration', async (data: any, respond: Function) => {
      const providers = this.declarationProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);
      const position = this.createPosition(data.position);

      for (const provider of providers) {
        try {
          const result = await provider.provideDeclaration(document, position, data.token);
          if (result) {
            const definitions = Array.isArray(result) ? result : [result];
            respond(definitions.map((d) => ({ uri: d.uri, range: d.range })));
            return;
          }
        } catch (error) {
          console.error('[Languages] Declaration provider error:', error);
        }
      }

      respond([]);
    });

    // Reference provider
    this.bridge.on('provideReferences', async (data: any, respond: Function) => {
      const providers = this.referenceProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);
      const position = this.createPosition(data.position);

      for (const provider of providers) {
        try {
          const result = await provider.provideReferences(
            document,
            position,
            { includeDeclaration: data.includeDeclaration ?? true },
            data.token
          );

          if (result) {
            respond({
              references: result.map((r) => ({
                uri: r.uri,
                range: r.range,
              })),
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
      const document = await this.getDocumentFromRequest(data);
      const range = this.createRange(data.range);

      for (const provider of providers) {
        try {
          const result = await provider.provideCodeActions(
            document,
            range,
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
        actions: allActions.map((a) => ({
          title: a.title,
          kind: a.kind,
          edit: a.edit,
          command: a.command,
          isPreferred: a.isPreferred,
        })),
      });
    });

    // Document symbol provider
    this.bridge.on('provideDocumentSymbols', async (data: any, respond: Function) => {
      const providers = this.symbolProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);

      for (const provider of providers) {
        try {
          const result = await provider.provideDocumentSymbols(document, data.token);

          if (result) {
            respond({
              symbols: result.map((s) => this.serializeDocumentSymbol(s)),
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
      const document = await this.getDocumentFromRequest(data);

      for (const provider of providers) {
        try {
          const result = await provider.provideDocumentFormattingEdits(
            document,
            {
              tabSize: data.options?.tabSize || 4,
              insertSpaces: data.options?.insertSpaces ?? true,
            },
            data.token
          );

          if (result) {
            respond({
              edits: result.map((e) => ({
                range: e.range,
                newText: e.newText,
              })),
            });
            return;
          }
        } catch (error) {
          console.error('[Languages] Formatting provider error:', error);
        }
      }

      respond({ edits: [] });
    });

    this.bridge.on('provideSignatureHelp', async (data: any, respond: Function) => {
      const providers = this.signatureHelpProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);
      const position = this.createPosition(data.position);

      for (const provider of providers) {
        try {
          const result = await provider.provideSignatureHelp(
            document,
            position,
            data.token,
            data.context
          );
          if (result) {
            respond(result);
            return;
          }
        } catch (error) {
          console.error('[Languages] Signature help provider error:', error);
        }
      }

      respond(null);
    });

    this.bridge.on('provideRename', async (data: any, respond: Function) => {
      const providers = this.renameProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);
      const position = this.createPosition(data.position);

      for (const provider of providers) {
        try {
          const result = await provider.provideRenameEdits(
            document,
            position,
            data.newName,
            data.token
          );
          if (result) {
            respond(result);
            return;
          }
        } catch (error) {
          console.error('[Languages] Rename provider error:', error);
        }
      }

      respond(null);
    });

    this.bridge.on('prepareRename', async (data: any, respond: Function) => {
      const providers = this.renameProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);
      const position = this.createPosition(data.position);

      for (const provider of providers) {
        if (!provider.prepareRename) {
          continue;
        }

        try {
          const result = await provider.prepareRename(document, position, data.token);
          if (result) {
            respond(result);
            return;
          }
        } catch (error) {
          console.error('[Languages] Prepare rename provider error:', error);
        }
      }

      respond(null);
    });

    this.bridge.on('provideCodeLenses', async (data: any, respond: Function) => {
      const providers = this.codeLensProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);
      const lenses: any[] = [];

      for (const provider of providers) {
        try {
          const result = await provider.provideCodeLenses(document, data.token);
          if (Array.isArray(result)) {
            lenses.push(...result);
          }
        } catch (error) {
          console.error('[Languages] Code lens provider error:', error);
        }
      }

      respond(lenses);
    });

    this.bridge.on('provideDocumentLinks', async (data: any, respond: Function) => {
      const providers = this.documentLinkProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);
      const links: any[] = [];

      for (const provider of providers) {
        try {
          const result = await provider.provideDocumentLinks(document, data.token);
          if (Array.isArray(result)) {
            links.push(...result);
          }
        } catch (error) {
          console.error('[Languages] Document link provider error:', error);
        }
      }

      respond(links);
    });

    this.bridge.on('provideDocumentHighlights', async (data: any, respond: Function) => {
      const providers = this.documentHighlightProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);
      const position = this.createPosition(data.position);

      for (const provider of providers) {
        try {
          const result = await provider.provideDocumentHighlights(document, position, data.token);
          if (result) {
            respond(result);
            return;
          }
        } catch (error) {
          console.error('[Languages] Document highlight provider error:', error);
        }
      }

      respond([]);
    });

    this.bridge.on('provideFoldingRanges', async (data: any, respond: Function) => {
      const providers = this.foldingRangeProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);

      for (const provider of providers) {
        try {
          const result = await provider.provideFoldingRanges(
            document,
            data.context || {},
            data.token
          );
          if (result) {
            respond(result);
            return;
          }
        } catch (error) {
          console.error('[Languages] Folding range provider error:', error);
        }
      }

      respond([]);
    });

    this.bridge.on('provideSelectionRanges', async (data: any, respond: Function) => {
      const providers = this.selectionRangeProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);
      const positions = (data.positions || []).map((position: any) =>
        this.createPosition(position)
      );

      for (const provider of providers) {
        try {
          const result = await provider.provideSelectionRanges(document, positions, data.token);
          if (result) {
            respond(result);
            return;
          }
        } catch (error) {
          console.error('[Languages] Selection range provider error:', error);
        }
      }

      respond([]);
    });

    this.bridge.on('provideWorkspaceSymbols', async (data: any, respond: Function) => {
      for (const provider of this.workspaceSymbolProviders) {
        try {
          const result = await provider.provideWorkspaceSymbols(data.query || '', data.token);
          if (result) {
            respond(result);
            return;
          }
        } catch (error) {
          console.error('[Languages] Workspace symbol provider error:', error);
        }
      }

      respond([]);
    });

    this.bridge.on('provideRangeFormatting', async (data: any, respond: Function) => {
      const providers = this.rangeFormattingProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);
      const range = this.createRange(data.range);

      for (const provider of providers) {
        try {
          const result = await provider.provideDocumentRangeFormattingEdits(
            document,
            range,
            data.options || {},
            data.token
          );
          if (result) {
            respond(result);
            return;
          }
        } catch (error) {
          console.error('[Languages] Range formatting provider error:', error);
        }
      }

      respond([]);
    });

    this.bridge.on('provideOnTypeFormatting', async (data: any, respond: Function) => {
      const providers = this.onTypeFormattingProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);
      const position = this.createPosition(data.position);

      for (const provider of providers) {
        try {
          const result = await provider.provideOnTypeFormattingEdits(
            document,
            position,
            data.ch || '',
            data.options || {},
            data.token
          );
          if (result) {
            respond(result);
            return;
          }
        } catch (error) {
          console.error('[Languages] On-type formatting provider error:', error);
        }
      }

      respond([]);
    });

    this.bridge.on('provideInlineCompletionItems', async (data: any, respond: Function) => {
      const providers = this.inlineCompletionProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);
      const position = this.createPosition(data.position);

      for (const provider of providers) {
        try {
          const result = await provider.provideInlineCompletionItems(
            document,
            position,
            data.context || {},
            data.token
          );
          if (result) {
            respond(result);
            return;
          }
        } catch (error) {
          console.error('[Languages] Inline completion provider error:', error);
        }
      }

      respond(null);
    });

    this.bridge.on('provideSemanticTokens', async (data: any, respond: Function) => {
      const providers = this.semanticTokensProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);

      for (const provider of providers) {
        try {
          const result = await provider.provideDocumentSemanticTokens(document, data.token);
          if (result) {
            respond(result);
            return;
          }
        } catch (error) {
          console.error('[Languages] Semantic tokens provider error:', error);
        }
      }

      respond(null);
    });

    this.bridge.on('provideDocumentColors', async (data: any, respond: Function) => {
      const providers = this.documentColorProviders.get(data.languageId) || [];
      const document = await this.getDocumentFromRequest(data);

      for (const provider of providers) {
        try {
          const result = await provider.provideDocumentColors(document, data.token);
          if (result) {
            respond(result);
            return;
          }
        } catch (error) {
          console.error('[Languages] Document colors provider error:', error);
        }
      }

      respond([]);
    });

    this.bridge.on('provideColorPresentations', async (data: any, respond: Function) => {
      const providers = this.documentColorProviders.get(data.languageId) || [];

      for (const provider of providers) {
        try {
          const result = await provider.provideColorPresentations(
            data.color,
            data.context,
            data.token
          );
          if (result) {
            respond(result);
            return;
          }
        } catch (error) {
          console.error('[Languages] Color presentations provider error:', error);
        }
      }

      respond([]);
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
      children: symbol.children?.map((c) => this.serializeDocumentSymbol(c)) || [],
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
      triggerCharacters,
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
      },
    };
  }

  registerHoverProvider(selector: string | string[], provider: HoverProvider): { dispose(): void } {
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
      },
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
      },
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
      },
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
      },
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
      },
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
      },
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
    const languages = this.addProviders(this.renameProviders, selector, provider);
    this.bridge.send('registerRenameProvider', {
      languages,
      prepareProvider: Boolean(provider.prepareRename),
    });
    return {
      dispose: () => {
        this.removeProviders(this.renameProviders, languages, provider);
      },
    };
  }

  registerSignatureHelpProvider(
    selector: string | string[],
    provider: SignatureHelpProvider,
    ...triggerCharacters: string[]
  ): { dispose(): void } {
    const languages = this.addProviders(this.signatureHelpProviders, selector, provider);
    this.bridge.send('registerSignatureHelpProvider', { languages, triggerCharacters });
    return {
      dispose: () => {
        this.removeProviders(this.signatureHelpProviders, languages, provider);
      },
    };
  }

  registerCodeLensProvider(
    selector: string | string[],
    provider: CodeLensProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.codeLensProviders, selector, provider);
    this.bridge.send('registerCodeLensProvider', { languages });
    return {
      dispose: () => {
        this.removeProviders(this.codeLensProviders, languages, provider);
      },
    };
  }

  registerDocumentLinkProvider(
    selector: string | string[],
    provider: DocumentLinkProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.documentLinkProviders, selector, provider);
    this.bridge.send('registerDocumentLinkProvider', { languages });
    return {
      dispose: () => {
        this.removeProviders(this.documentLinkProviders, languages, provider);
      },
    };
  }

  registerColorProvider(
    selector: string | string[],
    provider: DocumentColorProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.documentColorProviders, selector, provider);
    this.bridge.send('registerColorProvider', { languages });
    return {
      dispose: () => {
        this.removeProviders(this.documentColorProviders, languages, provider);
      },
    };
  }

  registerFoldingRangeProvider(
    selector: string | string[],
    provider: FoldingRangeProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.foldingRangeProviders, selector, provider);
    this.bridge.send('registerFoldingRangeProvider', { languages });
    return {
      dispose: () => {
        this.removeProviders(this.foldingRangeProviders, languages, provider);
      },
    };
  }

  registerSelectionRangeProvider(
    selector: string | string[],
    provider: SelectionRangeProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.selectionRangeProviders, selector, provider);
    this.bridge.send('registerSelectionRangeProvider', { languages });
    return {
      dispose: () => {
        this.removeProviders(this.selectionRangeProviders, languages, provider);
      },
    };
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
    const languages = this.addProviders(this.semanticTokensProviders, selector, provider);
    this.bridge.send('registerSemanticTokensProvider', { languages, legend });
    return {
      dispose: () => {
        this.removeProviders(this.semanticTokensProviders, languages, provider);
      },
    };
  }

  registerInlineCompletionItemProvider(
    selector: string | string[],
    provider: InlineCompletionItemProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.inlineCompletionProviders, selector, provider);
    this.bridge.send('registerInlineCompletionProvider', { languages });
    return {
      dispose: () => {
        this.removeProviders(this.inlineCompletionProviders, languages, provider);
      },
    };
  }

  registerDocumentRangeFormattingEditProvider(
    selector: string | string[],
    provider: DocumentRangeFormattingEditProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.rangeFormattingProviders, selector, provider);
    this.bridge.send('registerRangeFormattingProvider', { languages });
    return {
      dispose: () => {
        this.removeProviders(this.rangeFormattingProviders, languages, provider);
      },
    };
  }

  registerOnTypeFormattingEditProvider(
    selector: string | string[],
    provider: OnTypeFormattingEditProvider,
    firstTriggerCharacter: string,
    ...moreTriggerCharacters: string[]
  ): { dispose(): void } {
    const languages = this.addProviders(this.onTypeFormattingProviders, selector, provider);
    this.bridge.send('registerOnTypeFormattingProvider', {
      languages,
      triggerCharacters: [firstTriggerCharacter, ...moreTriggerCharacters],
    });
    return {
      dispose: () => {
        this.removeProviders(this.onTypeFormattingProviders, languages, provider);
      },
    };
  }

  registerWorkspaceSymbolProvider(provider: WorkspaceSymbolProvider): { dispose(): void } {
    this.workspaceSymbolProviders.push(provider);
    this.bridge.send('registerWorkspaceSymbolProvider', {});
    return {
      dispose: () => {
        this.workspaceSymbolProviders = this.workspaceSymbolProviders.filter(
          (entry) => entry !== provider
        );
      },
    };
  }

  registerDocumentHighlightProvider(
    selector: string | string[],
    provider: DocumentHighlightProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.documentHighlightProviders, selector, provider);
    this.bridge.send('registerDocumentHighlightProvider', { languages });
    return {
      dispose: () => {
        this.removeProviders(this.documentHighlightProviders, languages, provider);
      },
    };
  }

  registerImplementationProvider(
    selector: string | string[],
    provider: ImplementationProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.implementationProviders, selector, provider);
    this.bridge.send('registerImplementationProvider', { languages });
    return {
      dispose: () => {
        this.removeProviders(this.implementationProviders, languages, provider);
      },
    };
  }

  registerTypeDefinitionProvider(
    selector: string | string[],
    provider: TypeDefinitionProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.typeDefinitionProviders, selector, provider);
    this.bridge.send('registerTypeDefinitionProvider', { languages });
    return {
      dispose: () => {
        this.removeProviders(this.typeDefinitionProviders, languages, provider);
      },
    };
  }

  registerDeclarationProvider(
    selector: string | string[],
    provider: DeclarationProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.declarationProviders, selector, provider);
    this.bridge.send('registerDeclarationProvider', { languages });
    return {
      dispose: () => {
        this.removeProviders(this.declarationProviders, languages, provider);
      },
    };
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

  constructor(
    private bridge: ExtensionHostBridge,
    private name?: string
  ) {}

  set(uri: string | { path: string; fsPath: string }, diagnostics: Diagnostic[]): void {
    const uriStr = typeof uri === 'string' ? uri : uri.fsPath;
    this.diagnostics.set(uriStr, diagnostics);

    this.bridge.send('updateDiagnostics', {
      collection: this.name,
      uri: uriStr,
      diagnostics,
    });
  }

  delete(uri: string | { path: string; fsPath: string }): void {
    const uriStr = typeof uri === 'string' ? uri : uri.fsPath;
    this.diagnostics.delete(uriStr);

    this.bridge.send('clearDiagnostics', {
      collection: this.name,
      uri: uriStr,
    });
  }

  clear(): void {
    this.diagnostics.clear();

    this.bridge.send('clearAllDiagnostics', {
      collection: this.name,
    });
  }

  forEach(
    callback: (uri: string, diagnostics: Diagnostic[], collection: DiagnosticCollection) => void
  ): void {
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
