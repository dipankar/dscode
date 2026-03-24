import * as monaco from 'monaco-editor';
import { registerForSelectorLanguages } from './shared';
import type { MonacoRegistrationContext } from './shared';
import type { DocumentSelector } from '../types';

export class MonacoEditingRegistrar {
  constructor(private readonly context: MonacoRegistrationContext) {}

  registerCompletionProvider(
    selector: DocumentSelector,
    providerId: string,
    triggerCharacters: string[],
  ) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerCompletionItemProvider(language, {
        triggerCharacters,
        provideCompletionItems: async (model, position) => {
          return this.context.providerClient.provideCompletionItems(
            model.uri.toString(),
            position,
            providerId,
          );
        },
      }),
    );
  }

  registerSignatureHelpProvider(
    selector: DocumentSelector,
    providerId: string,
    triggerCharacters: string[],
    retriggerCharacters: string[],
  ) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerSignatureHelpProvider(language, {
        signatureHelpTriggerCharacters: triggerCharacters,
        signatureHelpRetriggerCharacters: retriggerCharacters,
        provideSignatureHelp: async (model, position) => {
          return this.context.providerClient.provideSignatureHelp(
            model.uri.toString(),
            position,
            providerId,
          );
        },
      }),
    );
  }

  registerRenameProvider(
    selector: DocumentSelector,
    providerId: string,
    prepareProvider: boolean,
  ) {
    registerForSelectorLanguages(this.context, selector, (language) =>
      monaco.languages.registerRenameProvider(language, {
        provideRenameEdits: async (model, position, newName) => {
          return this.context.providerClient.provideRenameEdits(
            model.uri.toString(),
            position,
            newName,
            providerId,
          );
        },
        resolveRenameLocation: prepareProvider
          ? (async (
              model: monaco.editor.ITextModel,
              position: monaco.Position,
              _token: monaco.CancellationToken,
            ) => {
              return this.context.providerClient.prepareRename(
                model.uri.toString(),
                position,
                providerId,
              );
            }) as any
          : undefined,
      }),
    );
  }
}
