import * as monaco from 'monaco-editor';
import { registerForSelectorLanguages } from './shared';
import type { MonacoRegistrationContext } from './shared';
import type { DocumentSelector } from '../types';

export class MonacoFormattingRegistrar {
  constructor(private readonly context: MonacoRegistrationContext) {}

  registerDocumentFormattingProvider(selector: DocumentSelector, providerId: string) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerDocumentFormattingEditProvider(language, {
        provideDocumentFormattingEdits: async (model, options) => {
          return this.context.providerClient.provideDocumentFormattingEdits(
            model.uri.toString(),
            options,
            providerId,
          );
        },
      }),
    );
  }

  registerRangeFormattingProvider(selector: DocumentSelector, providerId: string) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerDocumentRangeFormattingEditProvider(language, {
        provideDocumentRangeFormattingEdits: async (model, range, options) => {
          return this.context.providerClient.provideRangeFormattingEdits(
            model.uri.toString(),
            range,
            options,
            providerId,
          );
        },
      }),
    );
  }

  registerOnTypeFormattingProvider(
    selector: DocumentSelector,
    providerId: string,
    triggerCharacters: string[],
  ) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerOnTypeFormattingEditProvider(language, {
        autoFormatTriggerCharacters: triggerCharacters,
        provideOnTypeFormattingEdits: async (model, position, ch, options) => {
          return this.context.providerClient.provideOnTypeFormattingEdits(
            model.uri.toString(),
            position,
            ch,
            options,
            providerId,
          );
        },
      }),
    );
  }
}
