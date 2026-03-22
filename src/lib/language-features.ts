import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import * as monaco from 'monaco-editor';

export interface DocumentFilter {
  language?: string;
  scheme?: string;
  pattern?: string;
}

export interface DocumentSelector {
  filters: DocumentFilter[];
}

export interface HoverProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface DefinitionProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface CompletionProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
  trigger_characters: string[];
}

export interface CodeActionProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
  code_action_kinds: string[];
}

export interface SignatureHelpProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
  trigger_characters: string[];
  retrigger_characters: string[];
}

export interface ReferencesProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface CodeLensProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface DocumentHighlightProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface FoldingRangeProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface RenameProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
  prepare_provider: boolean;
}

export interface DocumentSymbolsProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface WorkspaceSymbolsProvider {
  id: string;
  owner: string;
}

export interface DocumentFormattingProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface RangeFormattingProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface OnTypeFormattingProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
  trigger_characters: string[];
}

export interface SemanticTokensProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
  legend: SemanticTokensLegend;
}

export interface SemanticTokensLegend {
  token_types: string[];
  token_modifiers: string[];
}

export interface InlineValuesProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface ColorProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface SelectionRangeProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface LinkedEditingRangeProvider {
  id: string;
  owner: string;
  selector: DocumentSelector;
}

export interface Diagnostic {
  uri: string;
  range: Range;
  severity: DiagnosticSeverity;
  code?: string;
  source?: string;
  message: string;
  related_information?: DiagnosticRelatedInformation[];
}

export interface Range {
  start: Position;
  end: Position;
}

export interface Position {
  line: number;
  character: number;
}

export enum DiagnosticSeverity {
  Error = 1,
  Warning = 2,
  Information = 3,
  Hint = 4,
}

export interface DiagnosticRelatedInformation {
  location: Location;
  message: string;
}

export interface Location {
  uri: string;
  range: Range;
}

/**
 * Language Features Manager
 * Integrates extension language providers with Monaco editor
 */
export class LanguageFeaturesManager {
  private disposables: monaco.IDisposable[] = [];
  private monacoEditor: monaco.editor.IStandaloneCodeEditor | null = null;
  private initialized = false;

  /**
   * Initialize the language features manager
   */
  async initialize(editor: monaco.editor.IStandaloneCodeEditor) {
    if (this.initialized) return;

    this.monacoEditor = editor;
    this.initialized = true;

    // Listen for diagnostic changes
    await listen<[string, Diagnostic[]]>('diagnostics-changed', (event) => {
      const [uri, diagnostics] = event.payload;
      this.updateMonacoDiagnostics(uri, diagnostics);
    });

    await listen<string>('diagnostics-cleared', () => {
      // Clear all diagnostics
      monaco.editor.getModels().forEach((model) => {
        monaco.editor.setModelMarkers(model, 'extension', []);
      });
    });

    console.log('[LanguageFeatures] Initialized');
  }

  /**
   * Register a hover provider
   */
  async registerHoverProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    const provider: HoverProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await invoke<string>('register_hover_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerHoverProvider(filter.language, {
          provideHover: async (model, position) => {
            return this.provideHover(model.uri.toString(), position, providerId);
          },
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered hover provider:', id);
    return id;
  }

  /**
   * Register a definition provider
   */
  async registerDefinitionProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    const provider: DefinitionProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await invoke<string>('register_definition_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerDefinitionProvider(filter.language, {
          provideDefinition: async (model, position) => {
            return this.provideDefinition(model.uri.toString(), position, providerId);
          },
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered definition provider:', id);
    return id;
  }

  /**
   * Register a completion provider
   */
  async registerCompletionProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
    triggerCharacters: string[]
  ): Promise<string> {
    const provider: CompletionProvider = {
      id: providerId,
      owner,
      selector,
      trigger_characters: triggerCharacters,
    };

    const id = await invoke<string>('register_completion_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerCompletionItemProvider(filter.language, {
          triggerCharacters,
          provideCompletionItems: async (model, position) => {
            return this.provideCompletionItems(model.uri.toString(), position, providerId);
          },
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered completion provider:', id);
    return id;
  }

  /**
   * Register a signature help provider
   */
  async registerSignatureHelpProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
    triggerCharacters: string[],
    retriggerCharacters: string[] = []
  ): Promise<string> {
    const provider: SignatureHelpProvider = {
      id: providerId,
      owner,
      selector,
      trigger_characters: triggerCharacters,
      retrigger_characters: retriggerCharacters,
    };

    const id = await invoke<string>('register_signature_help_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerSignatureHelpProvider(filter.language, {
          signatureHelpTriggerCharacters: triggerCharacters,
          signatureHelpRetriggerCharacters: retriggerCharacters,
          provideSignatureHelp: async (model, position) => {
            return this.provideSignatureHelp(model.uri.toString(), position, providerId);
          },
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered signature help provider:', id);
    return id;
  }

  /**
   * Register a references provider
   */
  async registerReferencesProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    const provider: ReferencesProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await invoke<string>('register_references_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerReferenceProvider(filter.language, {
          provideReferences: async (model, position, context) => {
            return this.provideReferences(model.uri.toString(), position, providerId, context.includeDeclaration);
          },
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered references provider:', id);
    return id;
  }

  /**
   * Register a code lens provider
   */
  async registerCodeLensProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    const provider: CodeLensProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await invoke<string>('register_code_lens_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerCodeLensProvider(filter.language, {
          provideCodeLenses: async (model) => {
            return this.provideCodeLenses(model.uri.toString(), providerId);
          },
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered code lens provider:', id);
    return id;
  }

  /**
   * Register a document highlight provider
   */
  async registerDocumentHighlightProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    const provider: DocumentHighlightProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await invoke<string>('register_document_highlight_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerDocumentHighlightProvider(filter.language, {
          provideDocumentHighlights: async (model, position) => {
            return this.provideDocumentHighlights(model.uri.toString(), position, providerId);
          },
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered document highlight provider:', id);
    return id;
  }

  /**
   * Register a folding range provider
   */
  async registerFoldingRangeProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    const provider: FoldingRangeProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await invoke<string>('register_folding_range_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerFoldingRangeProvider(filter.language, {
          provideFoldingRanges: async (model) => {
            return this.provideFoldingRanges(model.uri.toString(), providerId);
          },
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered folding range provider:', id);
    return id;
  }

  /**
   * Register a rename provider
   */
  async registerRenameProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
    prepareProvider: boolean = false
  ): Promise<string> {
    const provider: RenameProvider = {
      id: providerId,
      owner,
      selector,
      prepare_provider: prepareProvider,
    };

    const id = await invoke<string>('register_rename_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerRenameProvider(filter.language, {
          provideRenameEdits: async (model, position, newName, _token) => {
            return this.provideRenameEdits(model.uri.toString(), position, newName, providerId);
          },
          resolveRenameLocation: prepareProvider
            ? (async (model: monaco.editor.ITextModel, position: monaco.Position, _token: monaco.CancellationToken) => {
                return this.prepareRename(model.uri.toString(), position, providerId);
              }) as any
            : undefined,
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered rename provider:', id);
    return id;
  }

  /**
   * Register a document symbols provider
   */
  async registerDocumentSymbolsProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    const provider: DocumentSymbolsProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await invoke<string>('register_document_symbols_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerDocumentSymbolProvider(filter.language, {
          provideDocumentSymbols: async (model) => {
            return this.provideDocumentSymbols(model.uri.toString(), providerId);
          },
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered document symbols provider:', id);
    return id;
  }

  /**
   * Register a workspace symbols provider
   * Note: Monaco doesn't have a registerWorkspaceSymbolProvider API,
   * so this just registers with the backend for extension host usage
   */
  async registerWorkspaceSymbolsProvider(
    providerId: string,
    owner: string
  ): Promise<string> {
    const provider: WorkspaceSymbolsProvider = {
      id: providerId,
      owner,
    };

    const id = await invoke<string>('register_workspace_symbols_provider', { provider });

    // Note: Monaco doesn't support workspace symbol providers directly
    // The extension host will handle these requests via IPC
    console.log('[LanguageFeatures] Registered workspace symbols provider:', id);
    return id;
  }

  /**
   * Register a document formatting provider
   */
  async registerDocumentFormattingProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    const provider: DocumentFormattingProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await invoke<string>('register_document_formatting_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerDocumentFormattingEditProvider(filter.language, {
          provideDocumentFormattingEdits: async (model, options) => {
            return this.provideDocumentFormattingEdits(model.uri.toString(), options, providerId);
          },
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered document formatting provider:', id);
    return id;
  }

  /**
   * Register a range formatting provider
   */
  async registerRangeFormattingProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    const provider: RangeFormattingProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await invoke<string>('register_range_formatting_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerDocumentRangeFormattingEditProvider(filter.language, {
          provideDocumentRangeFormattingEdits: async (model, range, options) => {
            return this.provideRangeFormattingEdits(model.uri.toString(), range, options, providerId);
          },
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered range formatting provider:', id);
    return id;
  }

  /**
   * Register an on-type formatting provider
   */
  async registerOnTypeFormattingProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
    triggerCharacters: string[]
  ): Promise<string> {
    const provider: OnTypeFormattingProvider = {
      id: providerId,
      owner,
      selector,
      trigger_characters: triggerCharacters,
    };

    const id = await invoke<string>('register_on_type_formatting_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerOnTypeFormattingEditProvider(filter.language, {
          autoFormatTriggerCharacters: triggerCharacters,
          provideOnTypeFormattingEdits: async (model, position, ch, options) => {
            return this.provideOnTypeFormattingEdits(model.uri.toString(), position, ch, options, providerId);
          },
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered on-type formatting provider:', id);
    return id;
  }

  /**
   * Register a semantic tokens provider
   */
  async registerSemanticTokensProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
    legend: SemanticTokensLegend
  ): Promise<string> {
    const provider: SemanticTokensProvider = {
      id: providerId,
      owner,
      selector,
      legend,
    };

    const id = await invoke<string>('register_semantic_tokens_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerDocumentSemanticTokensProvider(filter.language, {
          getLegend: () => ({
            tokenTypes: legend.token_types,
            tokenModifiers: legend.token_modifiers,
          }),
          provideDocumentSemanticTokens: async (model) => {
            return this.provideSemanticTokens(model.uri.toString(), providerId);
          },
          releaseDocumentSemanticTokens: () => {},
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered semantic tokens provider:', id);
    return id;
  }

  /**
   * Register a color provider
   */
  async registerColorProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    const provider: ColorProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await invoke<string>('register_color_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerColorProvider(filter.language, {
          provideDocumentColors: async (model) => {
            return this.provideDocumentColors(model.uri.toString(), providerId);
          },
          provideColorPresentations: async (model, colorInfo) => {
            return this.provideColorPresentations(model.uri.toString(), colorInfo, providerId);
          },
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered color provider:', id);
    return id;
  }

  /**
   * Register a selection range provider
   */
  async registerSelectionRangeProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    const provider: SelectionRangeProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await invoke<string>('register_selection_range_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerSelectionRangeProvider(filter.language, {
          provideSelectionRanges: async (model, positions) => {
            return this.provideSelectionRanges(model.uri.toString(), positions, providerId);
          },
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered selection range provider:', id);
    return id;
  }

  /**
   * Register a linked editing range provider
   */
  async registerLinkedEditingRangeProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    const provider: LinkedEditingRangeProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await invoke<string>('register_linked_editing_range_provider', { provider });

    // Register with Monaco
    selector.filters.forEach((filter) => {
      if (filter.language) {
        const disposable = monaco.languages.registerLinkedEditingRangeProvider(filter.language, {
          provideLinkedEditingRanges: async (model, position) => {
            return this.provideLinkedEditingRanges(model.uri.toString(), position, providerId);
          },
        });
        this.disposables.push(disposable);
      }
    });

    console.log('[LanguageFeatures] Registered linked editing range provider:', id);
    return id;
  }

  /**
   * Publish diagnostics for a document
   */
  async publishDiagnostics(uri: string, diagnostics: Diagnostic[]): Promise<void> {
    await invoke('publish_diagnostics', { uri, diagnostics });
    // The event listener will update Monaco
  }

  /**
   * Get diagnostics for a document
   */
  async getDiagnostics(uri: string): Promise<Diagnostic[]> {
    return await invoke<Diagnostic[]>('get_diagnostics', { uri });
  }

  /**
   * Clear diagnostics for a specific owner
   */
  async clearDiagnostics(owner: string): Promise<void> {
    await invoke('clear_diagnostics', { owner });
  }

  /**
   * Provide hover information (called by Monaco)
   */
  private async provideHover(
    uri: string,
    position: monaco.Position,
    providerId: string
  ): Promise<monaco.languages.Hover | null> {
    try {
      // Call extension host to get hover info
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideHover`,
        args: [
          uri,
          { line: position.lineNumber - 1, character: position.column - 1 },
        ],
      });

      if (result && result.contents) {
        return {
          contents: Array.isArray(result.contents)
            ? result.contents
            : [result.contents],
          range: result.range
            ? this.convertRangeToMonaco(result.range)
            : undefined,
        };
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Hover provider error:', error);
      return null;
    }
  }

  /**
   * Provide definition location (called by Monaco)
   */
  private async provideDefinition(
    uri: string,
    position: monaco.Position,
    providerId: string
  ): Promise<monaco.languages.Definition | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideDefinition`,
        args: [
          uri,
          { line: position.lineNumber - 1, character: position.column - 1 },
        ],
      });

      if (result) {
        if (Array.isArray(result)) {
          return result.map((loc) => this.convertLocationToMonaco(loc));
        } else {
          return this.convertLocationToMonaco(result);
        }
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Definition provider error:', error);
      return null;
    }
  }

  /**
   * Provide completion items (called by Monaco)
   */
  private async provideCompletionItems(
    uri: string,
    position: monaco.Position,
    providerId: string
  ): Promise<monaco.languages.CompletionList | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideCompletionItems`,
        args: [
          uri,
          { line: position.lineNumber - 1, character: position.column - 1 },
        ],
      });

      if (result && result.items) {
        return {
          suggestions: result.items.map((item: any) => ({
            label: item.label,
            kind: item.kind || monaco.languages.CompletionItemKind.Text,
            insertText: item.insertText || item.label,
            detail: item.detail,
            documentation: item.documentation,
            range: item.range
              ? this.convertRangeToMonaco(item.range)
              : undefined,
          })),
        };
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Completion provider error:', error);
      return null;
    }
  }

  /**
   * Provide signature help (called by Monaco)
   */
  private async provideSignatureHelp(
    uri: string,
    position: monaco.Position,
    providerId: string
  ): Promise<monaco.languages.SignatureHelpResult | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideSignatureHelp`,
        args: [
          uri,
          { line: position.lineNumber - 1, character: position.column - 1 },
        ],
      });

      if (result && result.signatures) {
        return {
          value: {
            signatures: result.signatures.map((sig: any) => ({
              label: sig.label,
              documentation: sig.documentation,
              parameters: sig.parameters || [],
              activeParameter: sig.activeParameter,
            })),
            activeSignature: result.activeSignature || 0,
            activeParameter: result.activeParameter || 0,
          },
          dispose: () => {},
        };
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Signature help provider error:', error);
      return null;
    }
  }

  /**
   * Provide references (called by Monaco)
   */
  private async provideReferences(
    uri: string,
    position: monaco.Position,
    providerId: string,
    includeDeclaration: boolean
  ): Promise<monaco.languages.Location[] | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideReferences`,
        args: [
          uri,
          { line: position.lineNumber - 1, character: position.column - 1 },
          { includeDeclaration },
        ],
      });

      if (result && Array.isArray(result)) {
        return result.map((loc) => this.convertLocationToMonaco(loc));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] References provider error:', error);
      return null;
    }
  }

  /**
   * Provide code lenses (called by Monaco)
   */
  private async provideCodeLenses(
    uri: string,
    providerId: string
  ): Promise<monaco.languages.CodeLensList | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideCodeLenses`,
        args: [uri],
      });

      if (result && Array.isArray(result)) {
        return {
          lenses: result.map((lens: any) => ({
            range: this.convertRangeToMonaco(lens.range),
            command: lens.command
              ? {
                  id: lens.command.command,
                  title: lens.command.title,
                  arguments: lens.command.arguments,
                }
              : undefined,
          })),
          dispose: () => {},
        };
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Code lens provider error:', error);
      return null;
    }
  }

  /**
   * Provide document highlights (called by Monaco)
   */
  private async provideDocumentHighlights(
    uri: string,
    position: monaco.Position,
    providerId: string
  ): Promise<monaco.languages.DocumentHighlight[] | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideDocumentHighlights`,
        args: [
          uri,
          { line: position.lineNumber - 1, character: position.column - 1 },
        ],
      });

      if (result && Array.isArray(result)) {
        return result.map((highlight: any) => ({
          range: this.convertRangeToMonaco(highlight.range),
          kind: highlight.kind || monaco.languages.DocumentHighlightKind.Text,
        }));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Document highlight provider error:', error);
      return null;
    }
  }

  /**
   * Provide folding ranges (called by Monaco)
   */
  private async provideFoldingRanges(
    uri: string,
    providerId: string
  ): Promise<monaco.languages.FoldingRange[] | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideFoldingRanges`,
        args: [uri],
      });

      if (result && Array.isArray(result)) {
        return result.map((range: any) => ({
          start: range.startLine + 1,
          end: range.endLine + 1,
          kind: range.kind,
        }));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Folding range provider error:', error);
      return null;
    }
  }

  /**
   * Provide rename edits (called by Monaco)
   */
  private async provideRenameEdits(
    uri: string,
    position: monaco.Position,
    newName: string,
    providerId: string
  ): Promise<monaco.languages.WorkspaceEdit | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideRenameEdits`,
        args: [
          uri,
          { line: position.lineNumber - 1, character: position.column - 1 },
          newName,
        ],
      });

      if (result && result.changes) {
        const edits: monaco.languages.WorkspaceEdit = {
          edits: [],
        };

        // Convert changes to Monaco format
        for (const [changeUri, textEdits] of Object.entries(result.changes)) {
          edits.edits.push({
            resource: monaco.Uri.parse(changeUri as string),
            versionId: undefined,
            textEdit: {
              range: this.convertRangeToMonaco((textEdits as any)[0].range),
              text: (textEdits as any)[0].newText,
            },
          } as any);
        }

        return edits;
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Rename provider error:', error);
      return null;
    }
  }

  /**
   * Prepare rename (called by Monaco)
   */
  private async prepareRename(
    uri: string,
    position: monaco.Position,
    providerId: string
  ): Promise<monaco.languages.RenameLocation | monaco.languages.Rejection | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.prepareRename`,
        args: [
          uri,
          { line: position.lineNumber - 1, character: position.column - 1 },
        ],
      });

      if (result && result.range) {
        return {
          range: this.convertRangeToMonaco(result.range),
          text: result.placeholder || '',
        };
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Prepare rename error:', error);
      return null;
    }
  }

  /**
   * Provide document symbols (called by Monaco)
   */
  private async provideDocumentSymbols(
    uri: string,
    providerId: string
  ): Promise<monaco.languages.DocumentSymbol[] | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideDocumentSymbols`,
        args: [uri],
      });

      if (result && Array.isArray(result)) {
        return result.map((symbol: any) => this.convertDocumentSymbol(symbol));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Document symbols provider error:', error);
      return null;
    }
  }

  /**
   * Provide workspace symbols (called internally, Monaco doesn't support this directly)
   */
  private async provideWorkspaceSymbols(
    query: string,
    providerId: string
  ): Promise<any[] | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideWorkspaceSymbols`,
        args: [query],
      });

      if (result && Array.isArray(result)) {
        return result.map((symbol: any) => ({
          name: symbol.name,
          kind: symbol.kind || monaco.languages.SymbolKind.Variable,
          containerName: symbol.containerName,
          location: {
            uri: symbol.location.uri,
            range: symbol.location.range,
          },
        }));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Workspace symbols provider error:', error);
      return null;
    }
  }

  /**
   * Provide document formatting edits (called by Monaco)
   */
  private async provideDocumentFormattingEdits(
    uri: string,
    options: monaco.languages.FormattingOptions,
    providerId: string
  ): Promise<monaco.languages.TextEdit[] | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideDocumentFormattingEdits`,
        args: [uri, options],
      });

      if (result && Array.isArray(result)) {
        return result.map((edit: any) => ({
          range: this.convertRangeToMonaco(edit.range),
          text: edit.newText,
        }));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Document formatting provider error:', error);
      return null;
    }
  }

  /**
   * Provide range formatting edits (called by Monaco)
   */
  private async provideRangeFormattingEdits(
    uri: string,
    range: monaco.IRange,
    options: monaco.languages.FormattingOptions,
    providerId: string
  ): Promise<monaco.languages.TextEdit[] | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideRangeFormattingEdits`,
        args: [
          uri,
          {
            start: { line: range.startLineNumber - 1, character: range.startColumn - 1 },
            end: { line: range.endLineNumber - 1, character: range.endColumn - 1 },
          },
          options,
        ],
      });

      if (result && Array.isArray(result)) {
        return result.map((edit: any) => ({
          range: this.convertRangeToMonaco(edit.range),
          text: edit.newText,
        }));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Range formatting provider error:', error);
      return null;
    }
  }

  /**
   * Provide on-type formatting edits (called by Monaco)
   */
  private async provideOnTypeFormattingEdits(
    uri: string,
    position: monaco.Position,
    ch: string,
    options: monaco.languages.FormattingOptions,
    providerId: string
  ): Promise<monaco.languages.TextEdit[] | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideOnTypeFormattingEdits`,
        args: [
          uri,
          { line: position.lineNumber - 1, character: position.column - 1 },
          ch,
          options,
        ],
      });

      if (result && Array.isArray(result)) {
        return result.map((edit: any) => ({
          range: this.convertRangeToMonaco(edit.range),
          text: edit.newText,
        }));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] On-type formatting provider error:', error);
      return null;
    }
  }

  /**
   * Provide semantic tokens (called by Monaco)
   */
  private async provideSemanticTokens(
    uri: string,
    providerId: string
  ): Promise<monaco.languages.SemanticTokens | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideSemanticTokens`,
        args: [uri],
      });

      if (result && result.data) {
        return {
          data: new Uint32Array(result.data),
          resultId: result.resultId,
        };
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Semantic tokens provider error:', error);
      return null;
    }
  }

  /**
   * Provide document colors (called by Monaco)
   */
  private async provideDocumentColors(
    uri: string,
    providerId: string
  ): Promise<monaco.languages.IColorInformation[] | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideDocumentColors`,
        args: [uri],
      });

      if (result && Array.isArray(result)) {
        return result.map((color: any) => ({
          range: this.convertRangeToMonaco(color.range),
          color: {
            red: color.color.red,
            green: color.color.green,
            blue: color.color.blue,
            alpha: color.color.alpha,
          },
        }));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Document colors provider error:', error);
      return null;
    }
  }

  /**
   * Provide color presentations (called by Monaco)
   */
  private async provideColorPresentations(
    uri: string,
    colorInfo: monaco.languages.IColorInformation,
    providerId: string
  ): Promise<monaco.languages.IColorPresentation[] | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideColorPresentations`,
        args: [
          uri,
          {
            range: {
              start: { line: colorInfo.range.startLineNumber - 1, character: colorInfo.range.startColumn - 1 },
              end: { line: colorInfo.range.endLineNumber - 1, character: colorInfo.range.endColumn - 1 },
            },
            color: colorInfo.color,
          },
        ],
      });

      if (result && Array.isArray(result)) {
        return result.map((presentation: any) => ({
          label: presentation.label,
          textEdit: presentation.textEdit ? {
            range: this.convertRangeToMonaco(presentation.textEdit.range),
            text: presentation.textEdit.newText,
          } : undefined,
          additionalTextEdits: presentation.additionalTextEdits
            ? presentation.additionalTextEdits.map((edit: any) => ({
                range: this.convertRangeToMonaco(edit.range),
                text: edit.newText,
              }))
            : undefined,
        }));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Color presentations provider error:', error);
      return null;
    }
  }

  /**
   * Provide selection ranges (called by Monaco)
   */
  private async provideSelectionRanges(
    uri: string,
    positions: monaco.Position[],
    providerId: string
  ): Promise<monaco.languages.SelectionRange[][] | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideSelectionRanges`,
        args: [
          uri,
          positions.map(pos => ({ line: pos.lineNumber - 1, character: pos.column - 1 })),
        ],
      });

      if (result && Array.isArray(result)) {
        return result.map((rangeList: any[]) =>
          rangeList.map(range => this.convertSelectionRange(range))
        );
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Selection ranges provider error:', error);
      return null;
    }
  }

  /**
   * Provide linked editing ranges (called by Monaco)
   */
  private async provideLinkedEditingRanges(
    uri: string,
    position: monaco.Position,
    providerId: string
  ): Promise<monaco.languages.LinkedEditingRanges | null> {
    try {
      const result = await invoke<any>('extension_execute_command', {
        command: `${providerId}.provideLinkedEditingRanges`,
        args: [uri, { line: position.lineNumber - 1, character: position.column - 1 }],
      });

      if (result && result.ranges) {
        return {
          ranges: result.ranges.map((range: any) => this.convertRangeToMonaco(range)),
          wordPattern: result.wordPattern ? new RegExp(result.wordPattern) : undefined,
        };
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Linked editing ranges provider error:', error);
      return null;
    }
  }

  /**
   * Convert selection range from extension format to Monaco format
   * Note: Monaco's SelectionRange doesn't support parent, so we flatten the hierarchy
   */
  private convertSelectionRange(range: any): monaco.languages.SelectionRange {
    return {
      range: this.convertRangeToMonaco(range.range),
    };
  }

  /**
   * Convert document symbol from extension format to Monaco format
   */
  private convertDocumentSymbol(symbol: any): monaco.languages.DocumentSymbol {
    return {
      name: symbol.name,
      detail: symbol.detail || '',
      kind: symbol.kind || monaco.languages.SymbolKind.Variable,
      tags: symbol.tags || [],
      range: this.convertRangeToMonaco(symbol.range),
      selectionRange: this.convertRangeToMonaco(symbol.selectionRange || symbol.range),
      children: symbol.children ? symbol.children.map((c: any) => this.convertDocumentSymbol(c)) : [],
    };
  }

  /**
   * Update Monaco diagnostics from extension diagnostics
   */
  private updateMonacoDiagnostics(uri: string, diagnostics: Diagnostic[]) {
    const model = monaco.editor.getModels().find((m) => m.uri.toString() === uri);
    if (!model) return;

    const markers: monaco.editor.IMarkerData[] = diagnostics.map((diag) => ({
      severity: this.convertSeverityToMonaco(diag.severity),
      startLineNumber: diag.range.start.line + 1,
      startColumn: diag.range.start.character + 1,
      endLineNumber: diag.range.end.line + 1,
      endColumn: diag.range.end.character + 1,
      message: diag.message,
      source: diag.source,
      code: diag.code,
    }));

    monaco.editor.setModelMarkers(model, 'extension', markers);
  }

  /**
   * Convert diagnostic severity to Monaco severity
   */
  private convertSeverityToMonaco(
    severity: DiagnosticSeverity
  ): monaco.MarkerSeverity {
    switch (severity) {
      case DiagnosticSeverity.Error:
        return monaco.MarkerSeverity.Error;
      case DiagnosticSeverity.Warning:
        return monaco.MarkerSeverity.Warning;
      case DiagnosticSeverity.Information:
        return monaco.MarkerSeverity.Info;
      case DiagnosticSeverity.Hint:
        return monaco.MarkerSeverity.Hint;
      default:
        return monaco.MarkerSeverity.Info;
    }
  }

  /**
   * Convert range from extension format to Monaco format
   */
  private convertRangeToMonaco(range: Range): monaco.IRange {
    return {
      startLineNumber: range.start.line + 1,
      startColumn: range.start.character + 1,
      endLineNumber: range.end.line + 1,
      endColumn: range.end.character + 1,
    };
  }

  /**
   * Convert location from extension format to Monaco format
   */
  private convertLocationToMonaco(location: Location): monaco.languages.Location {
    return {
      uri: monaco.Uri.parse(location.uri),
      range: this.convertRangeToMonaco(location.range),
    };
  }

  /**
   * Dispose all registered providers
   */
  dispose() {
    this.disposables.forEach((d) => d.dispose());
    this.disposables = [];
    this.initialized = false;
  }
}

// Export singleton instance
export const languageFeaturesManager = new LanguageFeaturesManager();
