import * as monaco from 'monaco-editor';
import { convertRangeToMonaco } from '../monaco-conversions';
import { BaseProviderClient, toProviderPosition } from './shared';

export class EditingProviderClient extends BaseProviderClient {
  async provideCompletionItems(
    uri: string,
    position: monaco.Position,
    providerId: string,
  ): Promise<monaco.languages.CompletionList | null> {
    try {
      const result = await this.executeProviderCommand<any>(
        providerId,
        'provideCompletionItems',
        [uri, toProviderPosition(position)],
      );

      if (result && result.items) {
        return {
          suggestions: result.items.map((item: any) => ({
            label: item.label,
            kind: item.kind || monaco.languages.CompletionItemKind.Text,
            insertText: item.insertText || item.label,
            detail: item.detail,
            documentation: item.documentation,
            range: item.range ? convertRangeToMonaco(item.range) : undefined,
          })),
        };
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Completion provider error:', error);
      return null;
    }
  }

  async provideSignatureHelp(
    uri: string,
    position: monaco.Position,
    providerId: string,
  ): Promise<monaco.languages.SignatureHelpResult | null> {
    try {
      const result = await this.executeProviderCommand<any>(
        providerId,
        'provideSignatureHelp',
        [uri, toProviderPosition(position)],
      );

      if (result && result.signatures) {
        return {
          value: {
            signatures: result.signatures.map((signature: any) => ({
              label: signature.label,
              documentation: signature.documentation,
              parameters: signature.parameters || [],
              activeParameter: signature.activeParameter,
            })),
            activeSignature: result.activeSignature || 0,
            activeParameter: result.activeParameter || 0,
          },
          dispose: () => {},
        };
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Signature help provider error:', error);
      return null;
    }
  }

  async provideRenameEdits(
    uri: string,
    position: monaco.Position,
    newName: string,
    providerId: string,
  ): Promise<monaco.languages.WorkspaceEdit | null> {
    try {
      const result = await this.executeProviderCommand<any>(providerId, 'provideRenameEdits', [
        uri,
        toProviderPosition(position),
        newName,
      ]);

      if (result && result.changes) {
        const edits: monaco.languages.WorkspaceEdit = {
          edits: [],
        };

        for (const [changeUri, textEdits] of Object.entries(result.changes)) {
          edits.edits.push({
            resource: monaco.Uri.parse(changeUri as string),
            versionId: undefined,
            textEdit: {
              range: convertRangeToMonaco((textEdits as any)[0].range),
              text: (textEdits as any)[0].newText,
            },
          } as any);
        }

        return edits;
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Rename provider error:', error);
      return null;
    }
  }

  async prepareRename(
    uri: string,
    position: monaco.Position,
    providerId: string,
  ): Promise<monaco.languages.RenameLocation | monaco.languages.Rejection | null> {
    try {
      const result = await this.executeProviderCommand<any>(providerId, 'prepareRename', [
        uri,
        toProviderPosition(position),
      ]);

      if (result && result.range) {
        return {
          range: convertRangeToMonaco(result.range),
          text: result.placeholder || '',
        };
      }

      return null;
    } catch (error) {
      console.error('[LanguageFeatures] Prepare rename error:', error);
      return null;
    }
  }
}
