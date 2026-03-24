import * as monaco from 'monaco-editor';
import { registerForSelectorLanguages } from './shared';
import type { MonacoRegistrationContext } from './shared';
import type { DocumentSelector } from '../types';

export class MonacoNavigationRegistrar {
  constructor(private readonly context: MonacoRegistrationContext) {}

  registerHoverProvider(selector: DocumentSelector, providerId: string) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerHoverProvider(language, {
        provideHover: async (model, position) => {
          return this.context.providerClient.provideHover(
            model.uri.toString(),
            position,
            providerId,
          );
        },
      }),
    );
  }

  registerDefinitionProvider(selector: DocumentSelector, providerId: string) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerDefinitionProvider(language, {
        provideDefinition: async (model, position) => {
          return this.context.providerClient.provideDefinition(
            model.uri.toString(),
            position,
            providerId,
          );
        },
      }),
    );
  }

  registerReferencesProvider(selector: DocumentSelector, providerId: string) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerReferenceProvider(language, {
        provideReferences: async (model, position, context) => {
          return this.context.providerClient.provideReferences(
            model.uri.toString(),
            position,
            providerId,
            context.includeDeclaration,
          );
        },
      }),
    );
  }

  registerDocumentHighlightProvider(selector: DocumentSelector, providerId: string) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerDocumentHighlightProvider(language, {
        provideDocumentHighlights: async (model, position) => {
          return this.context.providerClient.provideDocumentHighlights(
            model.uri.toString(),
            position,
            providerId,
          );
        },
      }),
    );
  }

  registerFoldingRangeProvider(selector: DocumentSelector, providerId: string) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerFoldingRangeProvider(language, {
        provideFoldingRanges: async (model) => {
          return this.context.providerClient.provideFoldingRanges(
            model.uri.toString(),
            providerId,
          );
        },
      }),
    );
  }

  registerSelectionRangeProvider(selector: DocumentSelector, providerId: string) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerSelectionRangeProvider(language, {
        provideSelectionRanges: async (model, positions) => {
          return this.context.providerClient.provideSelectionRanges(
            model.uri.toString(),
            positions,
            providerId,
          );
        },
      }),
    );
  }

  registerLinkedEditingRangeProvider(selector: DocumentSelector, providerId: string) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerLinkedEditingRangeProvider(language, {
        provideLinkedEditingRanges: async (model, position) => {
          return this.context.providerClient.provideLinkedEditingRanges(
            model.uri.toString(),
            position,
            providerId,
          );
        },
      }),
    );
  }
}
