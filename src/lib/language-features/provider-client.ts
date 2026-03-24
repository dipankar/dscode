import * as monaco from 'monaco-editor';
import { EditingProviderClient } from './provider-clients/editing-provider-client';
import { FormattingProviderClient } from './provider-clients/formatting-provider-client';
import { NavigationProviderClient } from './provider-clients/navigation-provider-client';
import { SymbolProviderClient } from './provider-clients/symbol-provider-client';

export class LanguageFeatureProviderClient {
  private readonly navigationClient = new NavigationProviderClient();
  private readonly editingClient = new EditingProviderClient();
  private readonly formattingClient = new FormattingProviderClient();
  private readonly symbolClient = new SymbolProviderClient();

  async provideHover(
    uri: string,
    position: monaco.Position,
    providerId: string,
  ): Promise<monaco.languages.Hover | null> {
    return this.navigationClient.provideHover(uri, position, providerId);
  }

  async provideDefinition(
    uri: string,
    position: monaco.Position,
    providerId: string,
  ): Promise<monaco.languages.Definition | null> {
    return this.navigationClient.provideDefinition(uri, position, providerId);
  }

  async provideCompletionItems(
    uri: string,
    position: monaco.Position,
    providerId: string,
  ): Promise<monaco.languages.CompletionList | null> {
    return this.editingClient.provideCompletionItems(uri, position, providerId);
  }

  async provideSignatureHelp(
    uri: string,
    position: monaco.Position,
    providerId: string,
  ): Promise<monaco.languages.SignatureHelpResult | null> {
    return this.editingClient.provideSignatureHelp(uri, position, providerId);
  }

  async provideReferences(
    uri: string,
    position: monaco.Position,
    providerId: string,
    includeDeclaration: boolean,
  ): Promise<monaco.languages.Location[] | null> {
    return this.navigationClient.provideReferences(
      uri,
      position,
      providerId,
      includeDeclaration,
    );
  }

  async provideCodeLenses(
    uri: string,
    providerId: string,
  ): Promise<monaco.languages.CodeLensList | null> {
    return this.symbolClient.provideCodeLenses(uri, providerId);
  }

  async provideDocumentHighlights(
    uri: string,
    position: monaco.Position,
    providerId: string,
  ): Promise<monaco.languages.DocumentHighlight[] | null> {
    return this.navigationClient.provideDocumentHighlights(uri, position, providerId);
  }

  async provideFoldingRanges(
    uri: string,
    providerId: string,
  ): Promise<monaco.languages.FoldingRange[] | null> {
    return this.navigationClient.provideFoldingRanges(uri, providerId);
  }

  async provideRenameEdits(
    uri: string,
    position: monaco.Position,
    newName: string,
    providerId: string,
  ): Promise<monaco.languages.WorkspaceEdit | null> {
    return this.editingClient.provideRenameEdits(uri, position, newName, providerId);
  }

  async prepareRename(
    uri: string,
    position: monaco.Position,
    providerId: string,
  ): Promise<monaco.languages.RenameLocation | monaco.languages.Rejection | null> {
    return this.editingClient.prepareRename(uri, position, providerId);
  }

  async provideDocumentSymbols(
    uri: string,
    providerId: string,
  ): Promise<monaco.languages.DocumentSymbol[] | null> {
    return this.symbolClient.provideDocumentSymbols(uri, providerId);
  }

  async provideDocumentFormattingEdits(
    uri: string,
    options: monaco.languages.FormattingOptions,
    providerId: string,
  ): Promise<monaco.languages.TextEdit[] | null> {
    return this.formattingClient.provideDocumentFormattingEdits(uri, options, providerId);
  }

  async provideRangeFormattingEdits(
    uri: string,
    range: monaco.IRange,
    options: monaco.languages.FormattingOptions,
    providerId: string,
  ): Promise<monaco.languages.TextEdit[] | null> {
    return this.formattingClient.provideRangeFormattingEdits(uri, range, options, providerId);
  }

  async provideOnTypeFormattingEdits(
    uri: string,
    position: monaco.Position,
    ch: string,
    options: monaco.languages.FormattingOptions,
    providerId: string,
  ): Promise<monaco.languages.TextEdit[] | null> {
    return this.formattingClient.provideOnTypeFormattingEdits(
      uri,
      position,
      ch,
      options,
      providerId,
    );
  }

  async provideSemanticTokens(
    uri: string,
    providerId: string,
  ): Promise<monaco.languages.SemanticTokens | null> {
    return this.symbolClient.provideSemanticTokens(uri, providerId);
  }

  async provideDocumentColors(
    uri: string,
    providerId: string,
  ): Promise<monaco.languages.IColorInformation[] | null> {
    return this.symbolClient.provideDocumentColors(uri, providerId);
  }

  async provideColorPresentations(
    uri: string,
    colorInfo: monaco.languages.IColorInformation,
    providerId: string,
  ): Promise<monaco.languages.IColorPresentation[] | null> {
    return this.symbolClient.provideColorPresentations(uri, colorInfo, providerId);
  }

  async provideSelectionRanges(
    uri: string,
    positions: monaco.Position[],
    providerId: string,
  ): Promise<monaco.languages.SelectionRange[][] | null> {
    return this.navigationClient.provideSelectionRanges(uri, positions, providerId);
  }

  async provideLinkedEditingRanges(
    uri: string,
    position: monaco.Position,
    providerId: string,
  ): Promise<monaco.languages.LinkedEditingRanges | null> {
    return this.navigationClient.provideLinkedEditingRanges(uri, position, providerId);
  }
}
