import * as languageFeaturesTransport from '../transport';
import { logProviderRegistered } from './shared';
import type { LanguageFeatureRegistrationContext } from './shared';
import type {
  CompletionProvider,
  DocumentSelector,
  RenameProvider,
  SignatureHelpProvider,
} from '../types';

export class LanguageFeatureEditingRegistration {
  constructor(private readonly context: LanguageFeatureRegistrationContext) {}

  async registerCompletionProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
    triggerCharacters: string[],
  ): Promise<string> {
    const provider: CompletionProvider = {
      id: providerId,
      owner,
      selector,
      trigger_characters: triggerCharacters,
    };

    const id = await languageFeaturesTransport.registerCompletionProvider(provider);

    this.context.monacoRegistrar.registerCompletionProvider(
      selector,
      providerId,
      triggerCharacters,
    );

    logProviderRegistered('completion', id);
    return id;
  }

  async registerSignatureHelpProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
    triggerCharacters: string[],
    retriggerCharacters: string[] = [],
  ): Promise<string> {
    const provider: SignatureHelpProvider = {
      id: providerId,
      owner,
      selector,
      trigger_characters: triggerCharacters,
      retrigger_characters: retriggerCharacters,
    };

    const id = await languageFeaturesTransport.registerSignatureHelpProvider(provider);

    this.context.monacoRegistrar.registerSignatureHelpProvider(
      selector,
      providerId,
      triggerCharacters,
      retriggerCharacters,
    );

    logProviderRegistered('signature help', id);
    return id;
  }

  async registerRenameProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
    prepareProvider: boolean = false,
  ): Promise<string> {
    const provider: RenameProvider = {
      id: providerId,
      owner,
      selector,
      prepare_provider: prepareProvider,
    };

    const id = await languageFeaturesTransport.registerRenameProvider(provider);

    this.context.monacoRegistrar.registerRenameProvider(selector, providerId, prepareProvider);

    logProviderRegistered('rename', id);
    return id;
  }
}
