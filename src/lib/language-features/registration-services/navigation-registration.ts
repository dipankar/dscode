import * as languageFeaturesTransport from '../transport';
import { logProviderRegistered } from './shared';
import type { LanguageFeatureRegistrationContext } from './shared';
import type {
  DefinitionProvider,
  DocumentHighlightProvider,
  DocumentSelector,
  FoldingRangeProvider,
  HoverProvider,
  LinkedEditingRangeProvider,
  ReferencesProvider,
  SelectionRangeProvider,
} from '../types';

export class LanguageFeatureNavigationRegistration {
  constructor(private readonly context: LanguageFeatureRegistrationContext) {}

  async registerHoverProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
  ): Promise<string> {
    const provider: HoverProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await languageFeaturesTransport.registerHoverProvider(provider);

    this.context.monacoRegistrar.registerHoverProvider(selector, providerId);

    logProviderRegistered('hover', id);
    return id;
  }

  async registerDefinitionProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
  ): Promise<string> {
    const provider: DefinitionProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await languageFeaturesTransport.registerDefinitionProvider(provider);

    this.context.monacoRegistrar.registerDefinitionProvider(selector, providerId);

    logProviderRegistered('definition', id);
    return id;
  }

  async registerReferencesProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
  ): Promise<string> {
    const provider: ReferencesProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await languageFeaturesTransport.registerReferencesProvider(provider);

    this.context.monacoRegistrar.registerReferencesProvider(selector, providerId);

    logProviderRegistered('references', id);
    return id;
  }

  async registerDocumentHighlightProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
  ): Promise<string> {
    const provider: DocumentHighlightProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await languageFeaturesTransport.registerDocumentHighlightProvider(provider);

    this.context.monacoRegistrar.registerDocumentHighlightProvider(selector, providerId);

    logProviderRegistered('document highlight', id);
    return id;
  }

  async registerFoldingRangeProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
  ): Promise<string> {
    const provider: FoldingRangeProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await languageFeaturesTransport.registerFoldingRangeProvider(provider);

    this.context.monacoRegistrar.registerFoldingRangeProvider(selector, providerId);

    logProviderRegistered('folding range', id);
    return id;
  }

  async registerSelectionRangeProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
  ): Promise<string> {
    const provider: SelectionRangeProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await languageFeaturesTransport.registerSelectionRangeProvider(provider);

    this.context.monacoRegistrar.registerSelectionRangeProvider(selector, providerId);

    logProviderRegistered('selection range', id);
    return id;
  }

  async registerLinkedEditingRangeProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
  ): Promise<string> {
    const provider: LinkedEditingRangeProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await languageFeaturesTransport.registerLinkedEditingRangeProvider(provider);

    this.context.monacoRegistrar.registerLinkedEditingRangeProvider(selector, providerId);

    logProviderRegistered('linked editing range', id);
    return id;
  }
}
