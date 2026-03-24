import * as languageFeaturesTransport from '../transport';
import { logProviderRegistered } from './shared';
import type { LanguageFeatureRegistrationContext } from './shared';
import type {
  DocumentFormattingProvider,
  DocumentSelector,
  OnTypeFormattingProvider,
  RangeFormattingProvider,
} from '../types';

export class LanguageFeatureFormattingRegistration {
  constructor(private readonly context: LanguageFeatureRegistrationContext) {}

  async registerDocumentFormattingProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
  ): Promise<string> {
    const provider: DocumentFormattingProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await languageFeaturesTransport.registerDocumentFormattingProvider(provider);

    this.context.monacoRegistrar.registerDocumentFormattingProvider(selector, providerId);

    logProviderRegistered('document formatting', id);
    return id;
  }

  async registerRangeFormattingProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
  ): Promise<string> {
    const provider: RangeFormattingProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await languageFeaturesTransport.registerRangeFormattingProvider(provider);

    this.context.monacoRegistrar.registerRangeFormattingProvider(selector, providerId);

    logProviderRegistered('range formatting', id);
    return id;
  }

  async registerOnTypeFormattingProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
    triggerCharacters: string[],
  ): Promise<string> {
    const provider: OnTypeFormattingProvider = {
      id: providerId,
      owner,
      selector,
      trigger_characters: triggerCharacters,
    };

    const id = await languageFeaturesTransport.registerOnTypeFormattingProvider(provider);

    this.context.monacoRegistrar.registerOnTypeFormattingProvider(
      selector,
      providerId,
      triggerCharacters,
    );

    logProviderRegistered('on-type formatting', id);
    return id;
  }
}
