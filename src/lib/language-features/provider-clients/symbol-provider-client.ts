import * as monaco from 'monaco-editor';
import { convertDocumentSymbol, convertRangeToMonaco } from '../monaco-conversions';
import { BaseProviderClient, toProviderRange } from './shared';

export class SymbolProviderClient extends BaseProviderClient {
  async provideCodeLenses(
    uri: string,
    providerId: string,
  ): Promise<monaco.languages.CodeLensList | null> {
    try {
      const result = await this.executeProviderCommand<any>(providerId, 'provideCodeLenses', [
        uri,
      ]);

      if (result && Array.isArray(result)) {
        return {
          lenses: result.map((lens: any) => ({
            range: convertRangeToMonaco(lens.range),
            command: lens.command
              ? {
                  id: lens.command.command,
                  title: lens.command.title,
                  arguments: lens.command.arguments,
                }
              : undefined,
          })),
          dispose: () => {},
        };
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Code lens provider error:', error);
      return null;
    }
  }

  async provideDocumentSymbols(
    uri: string,
    providerId: string,
  ): Promise<monaco.languages.DocumentSymbol[] | null> {
    try {
      const result = await this.executeProviderCommand<any>(
        providerId,
        'provideDocumentSymbols',
        [uri],
      );

      if (result && Array.isArray(result)) {
        return result.map((symbol: any) => convertDocumentSymbol(symbol));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Document symbols provider error:', error);
      return null;
    }
  }

  async provideSemanticTokens(
    uri: string,
    providerId: string,
  ): Promise<monaco.languages.SemanticTokens | null> {
    try {
      const result = await this.executeProviderCommand<any>(
        providerId,
        'provideSemanticTokens',
        [uri],
      );

      if (result && result.data) {
        return {
          data: new Uint32Array(result.data),
          resultId: result.resultId,
        };
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Semantic tokens provider error:', error);
      return null;
    }
  }

  async provideDocumentColors(
    uri: string,
    providerId: string,
  ): Promise<monaco.languages.IColorInformation[] | null> {
    try {
      const result = await this.executeProviderCommand<any>(
        providerId,
        'provideDocumentColors',
        [uri],
      );

      if (result && Array.isArray(result)) {
        return result.map((color: any) => ({
          range: convertRangeToMonaco(color.range),
          color: {
            red: color.color.red,
            green: color.color.green,
            blue: color.color.blue,
            alpha: color.color.alpha,
          },
        }));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Document colors provider error:', error);
      return null;
    }
  }

  async provideColorPresentations(
    uri: string,
    colorInfo: monaco.languages.IColorInformation,
    providerId: string,
  ): Promise<monaco.languages.IColorPresentation[] | null> {
    try {
      const result = await this.executeProviderCommand<any>(
        providerId,
        'provideColorPresentations',
        [
          uri,
          {
            range: toProviderRange(colorInfo.range),
            color: colorInfo.color,
          },
        ],
      );

      if (result && Array.isArray(result)) {
        return result.map((presentation: any) => ({
          label: presentation.label,
          textEdit: presentation.textEdit
            ? {
                range: convertRangeToMonaco(presentation.textEdit.range),
                text: presentation.textEdit.newText,
              }
            : undefined,
          additionalTextEdits: presentation.additionalTextEdits
            ? presentation.additionalTextEdits.map((edit: any) => ({
                range: convertRangeToMonaco(edit.range),
                text: edit.newText,
              }))
            : undefined,
        }));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Color presentations provider error:', error);
      return null;
    }
  }
}
