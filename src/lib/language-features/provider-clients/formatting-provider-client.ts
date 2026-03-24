import * as monaco from 'monaco-editor';
import { convertRangeToMonaco } from '../monaco-conversions';
import { BaseProviderClient, toProviderPosition, toProviderRange } from './shared';

export class FormattingProviderClient extends BaseProviderClient {
  async provideDocumentFormattingEdits(
    uri: string,
    options: monaco.languages.FormattingOptions,
    providerId: string,
  ): Promise<monaco.languages.TextEdit[] | null> {
    try {
      const result = await this.executeProviderCommand<any>(
        providerId,
        'provideDocumentFormattingEdits',
        [uri, options],
      );

      if (result && Array.isArray(result)) {
        return result.map((edit: any) => ({
          range: convertRangeToMonaco(edit.range),
          text: edit.newText,
        }));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Document formatting provider error:', error);
      return null;
    }
  }

  async provideRangeFormattingEdits(
    uri: string,
    range: monaco.IRange,
    options: monaco.languages.FormattingOptions,
    providerId: string,
  ): Promise<monaco.languages.TextEdit[] | null> {
    try {
      const result = await this.executeProviderCommand<any>(
        providerId,
        'provideRangeFormattingEdits',
        [uri, toProviderRange(range), options],
      );

      if (result && Array.isArray(result)) {
        return result.map((edit: any) => ({
          range: convertRangeToMonaco(edit.range),
          text: edit.newText,
        }));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Range formatting provider error:', error);
      return null;
    }
  }

  async provideOnTypeFormattingEdits(
    uri: string,
    position: monaco.Position,
    ch: string,
    options: monaco.languages.FormattingOptions,
    providerId: string,
  ): Promise<monaco.languages.TextEdit[] | null> {
    try {
      const result = await this.executeProviderCommand<any>(
        providerId,
        'provideOnTypeFormattingEdits',
        [uri, toProviderPosition(position), ch, options],
      );

      if (result && Array.isArray(result)) {
        return result.map((edit: any) => ({
          range: convertRangeToMonaco(edit.range),
          text: edit.newText,
        }));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] On-type formatting provider error:', error);
      return null;
    }
  }
}
