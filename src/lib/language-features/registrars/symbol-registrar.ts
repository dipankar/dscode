import * as monaco from 'monaco-editor';
import { registerForSelectorLanguages } from './shared';
import type { MonacoRegistrationContext } from './shared';
import type { DocumentSelector, SemanticTokensLegend } from '../types';

export class MonacoSymbolRegistrar {
  constructor(private readonly context: MonacoRegistrationContext) {}

  registerCodeLensProvider(selector: DocumentSelector, providerId: string) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerCodeLensProvider(language, {
        provideCodeLenses: async (model) => {
          return this.context.providerClient.provideCodeLenses(
            model.uri.toString(),
            providerId,
          );
        },
      }),
    );
  }

  registerDocumentSymbolsProvider(selector: DocumentSelector, providerId: string) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerDocumentSymbolProvider(language, {
        provideDocumentSymbols: async (model) => {
          return this.context.providerClient.provideDocumentSymbols(
            model.uri.toString(),
            providerId,
          );
        },
      }),
    );
  }

  registerSemanticTokensProvider(
    selector: DocumentSelector,
    providerId: string,
    legend: SemanticTokensLegend,
  ) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerDocumentSemanticTokensProvider(language, {
        getLegend: () => ({
          tokenTypes: legend.token_types,
          tokenModifiers: legend.token_modifiers,
        }),
        provideDocumentSemanticTokens: async (model) => {
          return this.context.providerClient.provideSemanticTokens(
            model.uri.toString(),
            providerId,
          );
        },
        releaseDocumentSemanticTokens: () => {},
      }),
    );
  }

  registerColorProvider(selector: DocumentSelector, providerId: string) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerColorProvider(language, {
        provideDocumentColors: async (model) => {
          return this.context.providerClient.provideDocumentColors(
            model.uri.toString(),
            providerId,
          );
        },
        provideColorPresentations: async (model, colorInfo) => {
          return this.context.providerClient.provideColorPresentations(
            model.uri.toString(),
            colorInfo,
            providerId,
          );
        },
      }),
    );
  }
}
