import * as languageFeaturesTransport from '../transport';
import { logProviderRegistered } from './shared';
import type { LanguageFeatureRegistrationContext } from './shared';
import type {
  CodeLensProvider,
  ColorProvider,
  DocumentSelector,
  DocumentSymbolsProvider,
  SemanticTokensLegend,
  SemanticTokensProvider,
  WorkspaceSymbolsProvider,
} from '../types';

export class LanguageFeatureSymbolRegistration {
  constructor(private readonly context: LanguageFeatureRegistrationContext) {}

  async registerCodeLensProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
  ): Promise<string> {
    const provider: CodeLensProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await languageFeaturesTransport.registerCodeLensProvider(provider);

    this.context.monacoRegistrar.registerCodeLensProvider(selector, providerId);

    logProviderRegistered('code lens', id);
    return id;
  }

  async registerDocumentSymbolsProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
  ): Promise<string> {
    const provider: DocumentSymbolsProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await languageFeaturesTransport.registerDocumentSymbolsProvider(provider);

    this.context.monacoRegistrar.registerDocumentSymbolsProvider(selector, providerId);

    logProviderRegistered('document symbols', id);
    return id;
  }

  async registerWorkspaceSymbolsProvider(providerId: string, owner: string): Promise<string> {
    const provider: WorkspaceSymbolsProvider = {
      id: providerId,
      owner,
    };

    const id = await languageFeaturesTransport.registerWorkspaceSymbolsProvider(provider);

    logProviderRegistered('workspace symbols', id);
    return id;
  }

  async registerSemanticTokensProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
    legend: SemanticTokensLegend,
  ): Promise<string> {
    const provider: SemanticTokensProvider = {
      id: providerId,
      owner,
      selector,
      legend,
    };

    const id = await languageFeaturesTransport.registerSemanticTokensProvider(provider);

    this.context.monacoRegistrar.registerSemanticTokensProvider(selector, providerId, legend);

    logProviderRegistered('semantic tokens', id);
    return id;
  }

  async registerColorProvider(
    selector: DocumentSelector,
    providerId: string,
    owner: string,
  ): Promise<string> {
    const provider: ColorProvider = {
      id: providerId,
      owner,
      selector,
    };

    const id = await languageFeaturesTransport.registerColorProvider(provider);

    this.context.monacoRegistrar.registerColorProvider(selector, providerId);

    logProviderRegistered('color', id);
    return id;
  }
}
