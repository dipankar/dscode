import * as monaco from 'monaco-editor';
import { MonacoEditingRegistrar } from './registrars/editing-registrar';
import { MonacoFormattingRegistrar } from './registrars/formatting-registrar';
import { MonacoNavigationRegistrar } from './registrars/navigation-registrar';
import { MonacoSymbolRegistrar } from './registrars/symbol-registrar';
import type { MonacoRegistrationContext } from './registrars/shared';
import { LanguageFeatureProviderClient } from './provider-client';
import type { DocumentSelector, SemanticTokensLegend } from './types';

export class LanguageFeatureMonacoRegistrar {
  private readonly disposables: monaco.IDisposable[] = [];
  private readonly providerClient = new LanguageFeatureProviderClient();
  private readonly context: MonacoRegistrationContext = {
    disposables: this.disposables,
    providerClient: this.providerClient,
  };
  private readonly navigationRegistrar = new MonacoNavigationRegistrar(this.context);
  private readonly editingRegistrar = new MonacoEditingRegistrar(this.context);
  private readonly formattingRegistrar = new MonacoFormattingRegistrar(this.context);
  private readonly symbolRegistrar = new MonacoSymbolRegistrar(this.context);

  registerHoverProvider(selector: DocumentSelector, providerId: string) {
    this.navigationRegistrar.registerHoverProvider(selector, providerId);
  }

  registerDefinitionProvider(selector: DocumentSelector, providerId: string) {
    this.navigationRegistrar.registerDefinitionProvider(selector, providerId);
  }

  registerCompletionProvider(
    selector: DocumentSelector,
    providerId: string,
    triggerCharacters: string[],
  ) {
    this.editingRegistrar.registerCompletionProvider(selector, providerId, triggerCharacters);
  }

  registerSignatureHelpProvider(
    selector: DocumentSelector,
    providerId: string,
    triggerCharacters: string[],
    retriggerCharacters: string[],
  ) {
    this.editingRegistrar.registerSignatureHelpProvider(
      selector,
      providerId,
      triggerCharacters,
      retriggerCharacters,
    );
  }

  registerReferencesProvider(selector: DocumentSelector, providerId: string) {
    this.navigationRegistrar.registerReferencesProvider(selector, providerId);
  }

  registerCodeLensProvider(selector: DocumentSelector, providerId: string) {
    this.symbolRegistrar.registerCodeLensProvider(selector, providerId);
  }

  registerDocumentHighlightProvider(selector: DocumentSelector, providerId: string) {
    this.navigationRegistrar.registerDocumentHighlightProvider(selector, providerId);
  }

  registerFoldingRangeProvider(selector: DocumentSelector, providerId: string) {
    this.navigationRegistrar.registerFoldingRangeProvider(selector, providerId);
  }

  registerRenameProvider(
    selector: DocumentSelector,
    providerId: string,
    prepareProvider: boolean,
  ) {
    this.editingRegistrar.registerRenameProvider(selector, providerId, prepareProvider);
  }

  registerDocumentSymbolsProvider(selector: DocumentSelector, providerId: string) {
    this.symbolRegistrar.registerDocumentSymbolsProvider(selector, providerId);
  }

  registerDocumentFormattingProvider(selector: DocumentSelector, providerId: string) {
    this.formattingRegistrar.registerDocumentFormattingProvider(selector, providerId);
  }

  registerRangeFormattingProvider(selector: DocumentSelector, providerId: string) {
    this.formattingRegistrar.registerRangeFormattingProvider(selector, providerId);
  }

  registerOnTypeFormattingProvider(
    selector: DocumentSelector,
    providerId: string,
    triggerCharacters: string[],
  ) {
    this.formattingRegistrar.registerOnTypeFormattingProvider(
      selector,
      providerId,
      triggerCharacters,
    );
  }

  registerSemanticTokensProvider(
    selector: DocumentSelector,
    providerId: string,
    legend: SemanticTokensLegend,
  ) {
    this.symbolRegistrar.registerSemanticTokensProvider(selector, providerId, legend);
  }

  registerColorProvider(selector: DocumentSelector, providerId: string) {
    this.symbolRegistrar.registerColorProvider(selector, providerId);
  }

  registerSelectionRangeProvider(selector: DocumentSelector, providerId: string) {
    this.navigationRegistrar.registerSelectionRangeProvider(selector, providerId);
  }

  registerLinkedEditingRangeProvider(selector: DocumentSelector, providerId: string) {
    this.navigationRegistrar.registerLinkedEditingRangeProvider(selector, providerId);
  }

  dispose() {
    this.disposables.forEach((disposable) => disposable.dispose());
    this.disposables.length = 0;
  }
}
