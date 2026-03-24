import * as monaco from 'monaco-editor';
import {
  convertLocationToMonaco,
  convertRangeToMonaco,
  convertSelectionRange,
} from '../monaco-conversions';
import { BaseProviderClient, toProviderPosition } from './shared';

export class NavigationProviderClient extends BaseProviderClient {
  async provideHover(
    uri: string,
    position: monaco.Position,
    providerId: string,
  ): Promise<monaco.languages.Hover | null> {
    try {
      const result = await this.executeProviderCommand<any>(providerId, 'provideHover', [
        uri,
        toProviderPosition(position),
      ]);

      if (result && result.contents) {
        return {
          contents: Array.isArray(result.contents) ? result.contents : [result.contents],
          range: result.range ? convertRangeToMonaco(result.range) : undefined,
        };
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Hover provider error:', error);
      return null;
    }
  }

  async provideDefinition(
    uri: string,
    position: monaco.Position,
    providerId: string,
  ): Promise<monaco.languages.Definition | null> {
    try {
      const result = await this.executeProviderCommand<any>(providerId, 'provideDefinition', [
        uri,
        toProviderPosition(position),
      ]);

      if (result) {
        if (Array.isArray(result)) {
          return result.map((location) => convertLocationToMonaco(location));
        }

        return convertLocationToMonaco(result);
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Definition provider error:', error);
      return null;
    }
  }

  async provideReferences(
    uri: string,
    position: monaco.Position,
    providerId: string,
    includeDeclaration: boolean,
  ): Promise<monaco.languages.Location[] | null> {
    try {
      const result = await this.executeProviderCommand<any>(providerId, 'provideReferences', [
        uri,
        toProviderPosition(position),
        { includeDeclaration },
      ]);

      if (result && Array.isArray(result)) {
        return result.map((location) => convertLocationToMonaco(location));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] References provider error:', error);
      return null;
    }
  }

  async provideDocumentHighlights(
    uri: string,
    position: monaco.Position,
    providerId: string,
  ): Promise<monaco.languages.DocumentHighlight[] | null> {
    try {
      const result = await this.executeProviderCommand<any>(
        providerId,
        'provideDocumentHighlights',
        [uri, toProviderPosition(position)],
      );

      if (result && Array.isArray(result)) {
        return result.map((highlight: any) => ({
          range: convertRangeToMonaco(highlight.range),
          kind: highlight.kind || monaco.languages.DocumentHighlightKind.Text,
        }));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Document highlight provider error:', error);
      return null;
    }
  }

  async provideFoldingRanges(
    uri: string,
    providerId: string,
  ): Promise<monaco.languages.FoldingRange[] | null> {
    try {
      const result = await this.executeProviderCommand<any>(
        providerId,
        'provideFoldingRanges',
        [uri],
      );

      if (result && Array.isArray(result)) {
        return result.map((range: any) => ({
          start: range.startLine + 1,
          end: range.endLine + 1,
          kind: range.kind,
        }));
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Folding range provider error:', error);
      return null;
    }
  }

  async provideSelectionRanges(
    uri: string,
    positions: monaco.Position[],
    providerId: string,
  ): Promise<monaco.languages.SelectionRange[][] | null> {
    try {
      const result = await this.executeProviderCommand<any>(
        providerId,
        'provideSelectionRanges',
        [uri, positions.map((position) => toProviderPosition(position))],
      );

      if (result && Array.isArray(result)) {
        return result.map((rangeList: any[]) =>
          rangeList.map((range) => convertSelectionRange(range)),
        );
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Selection ranges provider error:', error);
      return null;
    }
  }

  async provideLinkedEditingRanges(
    uri: string,
    position: monaco.Position,
    providerId: string,
  ): Promise<monaco.languages.LinkedEditingRanges | null> {
    try {
      const result = await this.executeProviderCommand<any>(
        providerId,
        'provideLinkedEditingRanges',
        [uri, toProviderPosition(position)],
      );

      if (result && result.ranges) {
        return {
          ranges: result.ranges.map((range: any) => convertRangeToMonaco(range)),
          wordPattern: result.wordPattern ? new RegExp(result.wordPattern) : undefined,
        };
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Linked editing ranges provider error:', error);
      return null;
    }
  }
}
