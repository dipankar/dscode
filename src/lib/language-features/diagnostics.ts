import { listen } from '@tauri-apps/api/event';
import * as monaco from 'monaco-editor';
import { updateMonacoDiagnostics } from './monaco-conversions';
import * as languageFeaturesTransport from './transport';
import type { Diagnostic } from './types';

export class LanguageFeatureDiagnostics {
  async initialize() {
    await listen<[string, Diagnostic[]]>('diagnostics-changed', (event) => {
      const [uri, diagnostics] = event.payload;
      updateMonacoDiagnostics(uri, diagnostics);
    });

    await listen<string>('diagnostics-cleared', () => {
      monaco.editor.getModels().forEach((model) => {
        monaco.editor.setModelMarkers(model, 'extension', []);
      });
    });
  }

  async publishDiagnostics(uri: string, diagnostics: Diagnostic[]): Promise<void> {
    await languageFeaturesTransport.publishDiagnostics(uri, diagnostics);
  }

  async getDiagnostics(uri: string): Promise<Diagnostic[]> {
    return await languageFeaturesTransport.getDiagnostics(uri);
  }

  async clearDiagnostics(owner: string): Promise<void> {
    await languageFeaturesTransport.clearDiagnostics(owner);
  }
}
