/**
 * Languages API
 *
 * Provides language-specific features like completion, hover, etc.
 */

import { ExtensionHostBridge } from '../bridge';
import { TextDocument, Position, Range } from './textDocument';
import { WorkspaceEdit } from './textEditor';
import {
  CancellationToken,
  CodeLens,
  DocumentLink,
  DocumentHighlight,
  InlayHint,
  InlayHintLabelPart,
  SelectionRange,
  ColorInformation,
  ColorPresentation,
  FoldingRange,
  SemanticTokens,
  CallHierarchyItem,
  TypeHierarchyItem,
  InlineCompletionItem,
  InlineCompletionList,
  WorkspaceSymbol,
  SymbolKind,
  SymbolTag,
  SignatureHelp,
  Color,
  LanguageStatusItem,
  LanguageStatusSeverity,
} from './common';

export interface DocumentFilter {
  language?: string;
  scheme?: string;
  pattern?: string;
}

export type DocumentSelector = DocumentFilter[];

function normalizeSelector(selector: DocumentSelector | string | string[]): DocumentFilter[] {
  if (Array.isArray(selector)) {
    if (selector.length === 0) return [];
    if (typeof selector[0] === 'string') {
      return (selector as string[]).map((lang) => ({ language: lang }));
    }
    return selector as DocumentFilter[];
  }
  if (typeof selector === 'string') {
    return [{ language: selector }];
  }
  return [];
}

function selectorMatchesLanguage(
  selector: DocumentFilter[],
  languageId: string,
  uri?: string
): boolean {
  return selector.some((filter) => {
    if (filter.language && filter.language !== languageId && filter.language !== '*') {
      return false;
    }
    if (filter.scheme && uri && !uri.startsWith(filter.scheme + ':')) {
      return false;
    }
    if (filter.pattern && uri && !globMatch(filter.pattern, uri)) {
      return false;
    }
    return true;
  });
}

function globMatch(pattern: string, text: string): boolean {
  if (pattern === '*') return true;
  if (pattern.includes('*')) {
    const parts = pattern.split('*');
    if (parts.length === 2) {
      return text.startsWith(parts[0]) && text.endsWith(parts[1]);
    }
  }
  return pattern === text;
}

function selectorToLanguagesArray(selector: DocumentSelector | string | string[]): string[] {
  const filters = normalizeSelector(selector);
  return [...new Set(filters.map((f) => f.language).filter((l): l is string => !!l))];
}

interface ProviderEntry<T> {
  provider: T;
  selector: DocumentFilter[];
}

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
  command?: { title: string; command: string; arguments?: unknown[] };
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

  edit?: WorkspaceEdit;
  command?: { title: string; command: string; arguments?: unknown[] };
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
    token?: CancellationToken,
    context?: { triggerKind: number; triggerCharacter?: string }
  ): CompletionItem[] | CompletionList | Promise<CompletionItem[] | CompletionList>;
}

export interface HoverProvider {
  provideHover(
    document: TextDocument,
    position: Position,
    token?: CancellationToken
  ): Hover | null | undefined | Promise<Hover | null | undefined>;
}

export interface DefinitionProvider {
  provideDefinition(
    document: TextDocument,
    position: Position,
    token?: CancellationToken
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
    token?: CancellationToken
  ): Location[] | null | undefined | Promise<Location[] | null | undefined>;
}

export interface CodeActionProvider {
  provideCodeActions(
    document: TextDocument,
    range: Range,
    context: { diagnostics: Diagnostic[] },
    token?: CancellationToken
  ): CodeAction[] | null | undefined | Promise<CodeAction[] | null | undefined>;
}

export interface DocumentSymbolProvider {
  provideDocumentSymbols(
    document: TextDocument,
    token?: CancellationToken
  ): DocumentSymbol[] | null | undefined | Promise<DocumentSymbol[] | null | undefined>;
}

export interface DocumentFormattingEditProvider {
  provideDocumentFormattingEdits(
    document: TextDocument,
    options: { tabSize: number; insertSpaces: boolean },
    token?: CancellationToken
  ):
    | { range: Range; newText: string }[]
    | null
    | undefined
    | Promise<{ range: Range; newText: string }[] | null | undefined>;
}

// Additional provider interfaces
export interface RenameProvider {
  provideRenameEdits(
    document: TextDocument,
    position: Position,
    newName: string,
    token?: CancellationToken
  ): WorkspaceEdit | null | undefined | Promise<WorkspaceEdit | null | undefined>;
  prepareRename?(
    document: TextDocument,
    position: Position,
    token?: CancellationToken
  ):
    | Range
    | { range: Range; placeholder: string }
    | null
    | undefined
    | Promise<Range | { range: Range; placeholder: string } | null | undefined>;
}

export interface SignatureHelpProvider {
  provideSignatureHelp(
    document: TextDocument,
    position: Position,
    token?: CancellationToken,
    context?: { triggerKind: number; triggerCharacter?: string }
  ): SignatureHelp | null | undefined | Promise<SignatureHelp | null | undefined>;
}

export interface CodeLensProvider {
  provideCodeLenses(document: TextDocument, token?: CancellationToken): CodeLens[];
  resolveCodeLens?(codeLens: CodeLens, token?: CancellationToken): CodeLens;
}

export interface DocumentLinkProvider {
  provideDocumentLinks(document: TextDocument, token?: CancellationToken): DocumentLink[];
  resolveDocumentLink?(link: DocumentLink, token?: CancellationToken): DocumentLink;
}

export interface DocumentColorProvider {
  provideDocumentColors(document: TextDocument, token?: CancellationToken): ColorInformation[];
  provideColorPresentations(
    color: Color,
    context: { document: TextDocument; range: Range },
    token?: CancellationToken
  ): ColorPresentation[];
}

export interface FoldingRangeProvider {
  provideFoldingRanges(
    document: TextDocument,
    context: { maxRanges?: number },
    token?: CancellationToken
  ): FoldingRange[];
}

export interface SelectionRangeProvider {
  provideSelectionRanges(
    document: TextDocument,
    positions: Position[],
    token?: CancellationToken
  ): SelectionRange[];
}

export interface CallHierarchyProvider {
  prepareCallHierarchy(
    document: TextDocument,
    position: Position,
    token?: CancellationToken
  ):
    | CallHierarchyItem
    | CallHierarchyItem[]
    | null
    | undefined
    | Promise<CallHierarchyItem | CallHierarchyItem[] | null | undefined>;
  provideCallHierarchyIncomingCalls(
    item: CallHierarchyItem,
    token?: CancellationToken
  ): { from: CallHierarchyItem; fromRanges: Range[] }[];
  provideCallHierarchyOutgoingCalls(
    item: CallHierarchyItem,
    token?: CancellationToken
  ): { to: CallHierarchyItem; fromRanges: Range[] }[];
}

export interface TypeHierarchyProvider {
  prepareTypeHierarchy(
    document: TextDocument,
    position: Position,
    token?: CancellationToken
  ):
    | TypeHierarchyItem
    | TypeHierarchyItem[]
    | null
    | undefined
    | Promise<TypeHierarchyItem | TypeHierarchyItem[] | null | undefined>;
  provideTypeHierarchySupertypes(
    item: TypeHierarchyItem,
    token?: CancellationToken
  ): TypeHierarchyItem[];
  provideTypeHierarchySubtypes(
    item: TypeHierarchyItem,
    token?: CancellationToken
  ): TypeHierarchyItem[];
}

export interface SemanticTokensLegend {
  tokenTypes: string[];
  tokenModifiers: string[];
}

export interface DocumentSemanticTokensProvider {
  provideDocumentSemanticTokens(
    document: TextDocument,
    token?: CancellationToken
  ): SemanticTokens | null | undefined | Promise<SemanticTokens | null | undefined>;
  provideDocumentSemanticTokensEdits?(
    document: TextDocument,
    previousResultId: string,
    token?: CancellationToken
  ):
    | SemanticTokens
    | { edits: { start: number; deleteCount: number; data?: number[] }[] }
    | null
    | undefined;
}

export interface InlineCompletionItemProvider {
  provideInlineCompletionItems(
    document: TextDocument,
    position: Position,
    context: { triggerKind: number; selectedCompletionInfo?: { range: Range; text: string } },
    token?: CancellationToken
  ):
    | InlineCompletionItem[]
    | InlineCompletionList
    | null
    | undefined
    | Promise<InlineCompletionItem[] | InlineCompletionList | null | undefined>;
}

export interface DocumentRangeFormattingEditProvider {
  provideDocumentRangeFormattingEdits(
    document: TextDocument,
    range: Range,
    options: { tabSize: number; insertSpaces: boolean },
    token?: CancellationToken
  ): { range: Range; newText: string }[];
}

export interface OnTypeFormattingEditProvider {
  provideOnTypeFormattingEdits(
    document: TextDocument,
    position: Position,
    ch: string,
    options: { tabSize: number; insertSpaces: boolean },
    token?: CancellationToken
  ): { range: Range; newText: string }[];
}

export interface WorkspaceSymbolProvider {
  provideWorkspaceSymbols(query: string, token?: CancellationToken): WorkspaceSymbol[];
  resolveWorkspaceSymbol?(symbol: WorkspaceSymbol, token?: CancellationToken): WorkspaceSymbol;
}

export interface DocumentHighlightProvider {
  provideDocumentHighlights(
    document: TextDocument,
    position: Position,
    token?: CancellationToken
  ): DocumentHighlight[];
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

export type IndentActionValue = 0 | 1 | 2 | 3;

export interface EnterAction {
  indentText: string;
  outdentText: string;
  appendText: string;
  removeText: number;
  indentAction: IndentActionValue;
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
    unIndentedLinePattern?: RegExp;
  };
  onEnterRules?: {
    beforeText: RegExp;
    afterText?: RegExp;
    previousLineText?: RegExp;
    action: EnterAction;
  }[];
  autoClosingPairs?: {
    open: string;
    close: string;
    notIn?: string[];
  }[];
  surroundingPairs?: {
    open: string;
    close: string;
  }[];
  folding?: {
    offSide?: boolean;
    markers?: {
      start: RegExp;
      end: RegExp;
    };
  };
}

/**
 * LanguagesAPI manages language providers
 */
export class LanguagesAPI {
  private completionProviders = new Map<string, ProviderEntry<CompletionItemProvider>[]>();
  private hoverProviders = new Map<string, ProviderEntry<HoverProvider>[]>();
  private definitionProviders = new Map<string, ProviderEntry<DefinitionProvider>[]>();
  private referenceProviders = new Map<string, ProviderEntry<ReferenceProvider>[]>();
  private codeActionProviders = new Map<string, ProviderEntry<CodeActionProvider>[]>();
  private symbolProviders = new Map<string, ProviderEntry<DocumentSymbolProvider>[]>();
  private formattingProviders = new Map<string, ProviderEntry<DocumentFormattingEditProvider>[]>();
  private renameProviders = new Map<string, ProviderEntry<RenameProvider>[]>();
  private signatureHelpProviders = new Map<string, ProviderEntry<SignatureHelpProvider>[]>();
  private codeLensProviders = new Map<string, ProviderEntry<CodeLensProvider>[]>();
  private documentColorProviders = new Map<string, ProviderEntry<DocumentColorProvider>[]>();
  private foldingRangeProviders = new Map<string, ProviderEntry<FoldingRangeProvider>[]>();
  private selectionRangeProviders = new Map<string, ProviderEntry<SelectionRangeProvider>[]>();
  private workspaceSymbolProviders: WorkspaceSymbolProvider[] = [];
  private documentHighlightProviders = new Map<
    string,
    ProviderEntry<DocumentHighlightProvider>[]
  >();
  private rangeFormattingProviders = new Map<
    string,
    ProviderEntry<DocumentRangeFormattingEditProvider>[]
  >();
  private onTypeFormattingProviders = new Map<
    string,
    ProviderEntry<OnTypeFormattingEditProvider>[]
  >();
  private semanticTokensProviders = new Map<
    string,
    ProviderEntry<DocumentSemanticTokensProvider>[]
  >();
  private implementationProviders = new Map<string, ProviderEntry<ImplementationProvider>[]>();
  private typeDefinitionProviders = new Map<string, ProviderEntry<TypeDefinitionProvider>[]>();
  private declarationProviders = new Map<string, ProviderEntry<DeclarationProvider>[]>();
  private documentLinkProviders = new Map<string, ProviderEntry<DocumentLinkProvider>[]>();
  private inlineCompletionProviders = new Map<
    string,
    ProviderEntry<InlineCompletionItemProvider>[]
  >();

  constructor(private bridge: ExtensionHostBridge) {
    this.setupMessageHandlers();
  }

  private addProviders<T>(
    store: Map<string, ProviderEntry<T>[]>,
    selector: DocumentSelector | string | string[],
    provider: T
  ): string[] {
    const filters = normalizeSelector(selector);
    const languages = selectorToLanguagesArray(selector);
    for (const language of languages) {
      if (!store.has(language)) {
        store.set(language, []);
      }
      store.get(language)!.push({ provider, selector: filters });
    }
    return languages;
  }

  private removeProviders<T>(
    store: Map<string, ProviderEntry<T>[]>,
    languages: string[],
    provider: T
  ): void {
    for (const language of languages) {
      const entries = store.get(language);
      if (!entries) {
        continue;
      }

      const index = entries.findIndex((entry) => entry.provider === provider);
      if (index !== -1) {
        entries.splice(index, 1);
      }

      if (entries.length === 0) {
        store.delete(language);
      }
    }
  }

  private getProvidersForLanguage<T>(
    store: Map<string, ProviderEntry<T>[]>,
    languageId: string
  ): T[] {
    const entries = store.get(languageId) || [];
    return entries.map((entry) => entry.provider);
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
      const providers = this.getProvidersForLanguage(this.completionProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.hoverProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.definitionProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.implementationProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.typeDefinitionProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.declarationProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.referenceProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.codeActionProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.symbolProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.formattingProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.signatureHelpProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.renameProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.renameProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.codeLensProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.documentLinkProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(
        this.documentHighlightProviders,
        data.languageId
      );
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
      const providers = this.getProvidersForLanguage(this.foldingRangeProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.selectionRangeProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(
        this.rangeFormattingProviders,
        data.languageId
      );
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
      const providers = this.getProvidersForLanguage(
        this.onTypeFormattingProviders,
        data.languageId
      );
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
      const providers = this.getProvidersForLanguage(
        this.inlineCompletionProviders,
        data.languageId
      );
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
      const providers = this.getProvidersForLanguage(this.semanticTokensProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.documentColorProviders, data.languageId);
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
      const providers = this.getProvidersForLanguage(this.documentColorProviders, data.languageId);

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
    selector: DocumentSelector | string | string[],
    provider: CompletionItemProvider,
    ...triggerCharacters: string[]
  ): { dispose(): void } {
    const languages = this.addProviders(this.completionProviders, selector, provider);
    const normalizedSelector = normalizeSelector(selector);

    this.bridge.send('registerCompletionProvider', {
      languages: selectorToLanguagesArray(selector),
      triggerCharacters,
      selector: normalizedSelector,
    });

    return {
      dispose: () => {
        this.removeProviders(this.completionProviders, languages, provider);
      },
    };
  }

  registerHoverProvider(
    selector: DocumentSelector | string | string[],
    provider: HoverProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.hoverProviders, selector, provider);
    const normalizedSelector = normalizeSelector(selector);

    this.bridge.send('registerHoverProvider', {
      languages: selectorToLanguagesArray(selector),
      selector: normalizedSelector,
    });

    return {
      dispose: () => {
        this.removeProviders(this.hoverProviders, languages, provider);
      },
    };
  }

  registerDefinitionProvider(
    selector: DocumentSelector | string | string[],
    provider: DefinitionProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.definitionProviders, selector, provider);
    const normalizedSelector = normalizeSelector(selector);

    this.bridge.send('registerDefinitionProvider', {
      languages: selectorToLanguagesArray(selector),
      selector: normalizedSelector,
    });

    return {
      dispose: () => {
        this.removeProviders(this.definitionProviders, languages, provider);
      },
    };
  }

  registerReferenceProvider(
    selector: DocumentSelector | string | string[],
    provider: ReferenceProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.referenceProviders, selector, provider);
    this.bridge.send('registerReferenceProvider', {
      languages: selectorToLanguagesArray(selector),
      selector: normalizeSelector(selector),
    });
    return {
      dispose: () => {
        this.removeProviders(this.referenceProviders, languages, provider);
      },
    };
  }

  registerCodeActionsProvider(
    selector: DocumentSelector | string | string[],
    provider: CodeActionProvider,
    metadata?: any
  ): { dispose(): void } {
    const languages = this.addProviders(this.codeActionProviders, selector, provider);
    this.bridge.send('registerCodeActionsProvider', {
      languages: selectorToLanguagesArray(selector),
      selector: normalizeSelector(selector),
      metadata,
    });
    return {
      dispose: () => {
        this.removeProviders(this.codeActionProviders, languages, provider);
      },
    };
  }

  registerDocumentSymbolProvider(
    selector: DocumentSelector | string | string[],
    provider: DocumentSymbolProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.symbolProviders, selector, provider);
    this.bridge.send('registerDocumentSymbolProvider', {
      languages: selectorToLanguagesArray(selector),
      selector: normalizeSelector(selector),
    });
    return {
      dispose: () => {
        this.removeProviders(this.symbolProviders, languages, provider);
      },
    };
  }

  registerDocumentFormattingEditProvider(
    selector: DocumentSelector | string | string[],
    provider: DocumentFormattingEditProvider
  ): { dispose(): void } {
    const languages = this.addProviders(this.formattingProviders, selector, provider);
    this.bridge.send('registerFormattingProvider', {
      languages: selectorToLanguagesArray(selector),
      selector: normalizeSelector(selector),
    });

    return {
      dispose: () => {
        this.removeProviders(this.formattingProviders, languages, provider);
      },
    };
  }

  // Diagnostics collection
  createDiagnosticCollection(name?: string): DiagnosticCollection {
    return new DiagnosticCollection(this.bridge, name);
  }

  // Additional Language Providers
  registerRenameProvider(
    selector: DocumentSelector | string | string[],
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
    selector: DocumentSelector | string | string[],
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
    selector: DocumentSelector | string | string[],
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
    selector: DocumentSelector | string | string[],
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
    selector: DocumentSelector | string | string[],
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
    selector: DocumentSelector | string | string[],
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
    selector: DocumentSelector | string | string[],
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
    selector: DocumentSelector | string | string[],
    provider: CallHierarchyProvider
  ): { dispose(): void } {
    this.bridge.send('registerCallHierarchyProvider', { selector });
    return { dispose: () => {} };
  }

  registerTypeHierarchyProvider(
    selector: DocumentSelector | string | string[],
    provider: TypeHierarchyProvider
  ): { dispose(): void } {
    this.bridge.send('registerTypeHierarchyProvider', { selector });
    return { dispose: () => {} };
  }

  registerDocumentSemanticTokensProvider(
    selector: DocumentSelector | string | string[],
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
    selector: DocumentSelector | string | string[],
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
    selector: DocumentSelector | string | string[],
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
    selector: DocumentSelector | string | string[],
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
    selector: DocumentSelector | string | string[],
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
    selector: DocumentSelector | string | string[],
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
    selector: DocumentSelector | string | string[],
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
    selector: DocumentSelector | string | string[],
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

  createLanguageStatusItem(
    id: string,
    selector: DocumentSelector | string | string[]
  ): LanguageStatusItem {
    const item = new LanguageStatusItem(id, id);
    return item;
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
