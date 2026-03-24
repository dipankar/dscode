import * as monaco from 'monaco-editor';
import { LanguageFeatureDiagnostics } from './language-features/diagnostics';
import { LanguageFeatureMonacoRegistrar } from './language-features/monaco-registrar';
import { LanguageFeatureEditingRegistration } from './language-features/registration-services/editing-registration';
import { LanguageFeatureFormattingRegistration } from './language-features/registration-services/formatting-registration';
import { LanguageFeatureNavigationRegistration } from './language-features/registration-services/navigation-registration';
import { LanguageFeatureSymbolRegistration } from './language-features/registration-services/symbol-registration';
import type {
  Diagnostic,
  DocumentSelector,
  SemanticTokensLegend,
} from './language-features/types';

export type {
  CodeActionProvider,
  CodeLensProvider,
  ColorProvider,
  CompletionProvider,
  DefinitionProvider,
  Diagnostic,
  DiagnosticRelatedInformation,
  DiagnosticSeverity,
  DocumentFilter,
  DocumentFormattingProvider,
  DocumentHighlightProvider,
  DocumentSelector,
  DocumentSymbolsProvider,
  FoldingRangeProvider,
  HoverProvider,
  InlineValuesProvider,
  LinkedEditingRangeProvider,
  Location,
  OnTypeFormattingProvider,
  Position,
  Range,
  RangeFormattingProvider,
  ReferencesProvider,
  RenameProvider,
  SelectionRangeProvider,
  SemanticTokensLegend,
  SemanticTokensProvider,
  SignatureHelpProvider,
  WorkspaceSymbolsProvider,
} from './language-features/types';

/**
 * Language Features Manager
 * Integrates extension language providers with Monaco editor
 */
export class LanguageFeaturesManager {
  private readonly monacoRegistrar = new LanguageFeatureMonacoRegistrar();
  private readonly registrationContext = {
    monacoRegistrar: this.monacoRegistrar,
  };
  private readonly navigationRegistration = new LanguageFeatureNavigationRegistration(
    this.registrationContext,
  );
  private readonly editingRegistration = new LanguageFeatureEditingRegistration(
    this.registrationContext,
  );
  private readonly formattingRegistration = new LanguageFeatureFormattingRegistration(
    this.registrationContext,
  );
  private readonly symbolRegistration = new LanguageFeatureSymbolRegistration(
    this.registrationContext,
  );
  private readonly diagnostics = new LanguageFeatureDiagnostics();
  private initialized = false;

  /**
   * Initialize the language features manager
   */
  async initialize(_editor: monaco.editor.IStandaloneCodeEditor) {
    if (this.initialized) return;

    this.initialized = true;

    await this.diagnostics.initialize();

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
    return this.navigationRegistration.registerHoverProvider(selector, providerId, owner);
  }

  /**
   * Register a definition provider
   */
  async registerDefinitionProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    return this.navigationRegistration.registerDefinitionProvider(selector, providerId, owner);
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
    return this.editingRegistration.registerCompletionProvider(
      selector,
      providerId,
      owner,
      triggerCharacters,
    );
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
    return this.editingRegistration.registerSignatureHelpProvider(
      selector,
      providerId,
      owner,
      triggerCharacters,
      retriggerCharacters,
    );
  }

  /**
   * Register a references provider
   */
  async registerReferencesProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    return this.navigationRegistration.registerReferencesProvider(selector, providerId, owner);
  }

  /**
   * Register a code lens provider
   */
  async registerCodeLensProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    return this.symbolRegistration.registerCodeLensProvider(selector, providerId, owner);
  }

  /**
   * Register a document highlight provider
   */
  async registerDocumentHighlightProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    return this.navigationRegistration.registerDocumentHighlightProvider(
      selector,
      providerId,
      owner,
    );
  }

  /**
   * Register a folding range provider
   */
  async registerFoldingRangeProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    return this.navigationRegistration.registerFoldingRangeProvider(selector, providerId, owner);
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
    return this.editingRegistration.registerRenameProvider(
      selector,
      providerId,
      owner,
      prepareProvider,
    );
  }

  /**
   * Register a document symbols provider
   */
  async registerDocumentSymbolsProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    return this.symbolRegistration.registerDocumentSymbolsProvider(selector, providerId, owner);
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
    return this.symbolRegistration.registerWorkspaceSymbolsProvider(providerId, owner);
  }

  /**
   * Register a document formatting provider
   */
  async registerDocumentFormattingProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    return this.formattingRegistration.registerDocumentFormattingProvider(
      selector,
      providerId,
      owner,
    );
  }

  /**
   * Register a range formatting provider
   */
  async registerRangeFormattingProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    return this.formattingRegistration.registerRangeFormattingProvider(
      selector,
      providerId,
      owner,
    );
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
    return this.formattingRegistration.registerOnTypeFormattingProvider(
      selector,
      providerId,
      owner,
      triggerCharacters,
    );
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
    return this.symbolRegistration.registerSemanticTokensProvider(
      selector,
      providerId,
      owner,
      legend,
    );
  }

  /**
   * Register a color provider
   */
  async registerColorProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    return this.symbolRegistration.registerColorProvider(selector, providerId, owner);
  }

  /**
   * Register a selection range provider
   */
  async registerSelectionRangeProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    return this.navigationRegistration.registerSelectionRangeProvider(selector, providerId, owner);
  }

  /**
   * Register a linked editing range provider
   */
  async registerLinkedEditingRangeProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string
  ): Promise<string> {
    return this.navigationRegistration.registerLinkedEditingRangeProvider(
      selector,
      providerId,
      owner,
    );
  }

  /**
   * Publish diagnostics for a document
   */
  async publishDiagnostics(uri: string, diagnostics: Diagnostic[]): Promise<void> {
    await this.diagnostics.publishDiagnostics(uri, diagnostics);
  }

  /**
   * Get diagnostics for a document
   */
  async getDiagnostics(uri: string): Promise<Diagnostic[]> {
    return await this.diagnostics.getDiagnostics(uri);
  }

  /**
   * Clear diagnostics for a specific owner
   */
  async clearDiagnostics(owner: string): Promise<void> {
    await this.diagnostics.clearDiagnostics(owner);
  }

  /**
   * Dispose all registered providers
   */
  dispose() {
    this.monacoRegistrar.dispose();
    this.initialized = false;
  }
}

// Export singleton instance
export const languageFeaturesManager = new LanguageFeaturesManager();
