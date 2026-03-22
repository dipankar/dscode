/**
 * FileSystem API (workspace.fs)
 *
 * File system operations for workspace
 */

import { ExtensionHostBridge } from '../bridge';
import { Uri } from './uri';
import { FileType, FileStat, FilePermission, FileSystemError } from './common';

export { FileType, FileStat, FilePermission, FileSystemError };

export class FileSystemAPI {
  constructor(private bridge: ExtensionHostBridge) {}

  async stat(uri: Uri): Promise<FileStat> {
    const result = await this.bridge.request('fsStat', {
      uri: uri.toString()
    }) as { stat: FileStat };
    return result.stat;
  }

  async readDirectory(uri: Uri): Promise<[string, FileType][]> {
    const result = await this.bridge.request('fsReadDirectory', {
      uri: uri.toString()
    }) as { entries: [string, FileType][] };
    return result.entries;
  }

  async createDirectory(uri: Uri): Promise<void> {
    await this.bridge.request('fsCreateDirectory', {
      uri: uri.toString()
    });
  }

  async readFile(uri: Uri): Promise<Uint8Array> {
    const result = await this.bridge.request('fsReadFile', {
      uri: uri.toString()
    }) as { data: number[] };
    // Convert array/base64 to Uint8Array if needed
    return new Uint8Array(result.data);
  }

  async writeFile(uri: Uri, content: Uint8Array): Promise<void> {
    await this.bridge.request('fsWriteFile', {
      uri: uri.toString(),
      content: Array.from(content)
    });
  }

  async delete(uri: Uri, options?: { recursive?: boolean; useTrash?: boolean }): Promise<void> {
    await this.bridge.request('fsDelete', {
      uri: uri.toString(),
      options
    });
  }

  async rename(source: Uri, target: Uri, options?: { overwrite?: boolean }): Promise<void> {
    await this.bridge.request('fsRename', {
      source: source.toString(),
      target: target.toString(),
      options
    });
  }

  async copy(source: Uri, target: Uri, options?: { overwrite?: boolean }): Promise<void> {
    await this.bridge.request('fsCopy', {
      source: source.toString(),
      target: target.toString(),
      options
    });
  }

  isWritableFileSystem(scheme: string): boolean | undefined {
    // Default implementation - can be extended
    return scheme !== 'vscode' && scheme !== 'untitled';
  }
}

// TextDocumentContentProvider
export interface TextDocumentContentProvider {
  onDidChange?: any; // Event<Uri>
  provideTextDocumentContent(uri: Uri, token: any): string | Promise<string> | undefined | null;
}

export class TextDocumentContentAPI {
  private providers = new Map<string, TextDocumentContentProvider>();

  constructor(private bridge: ExtensionHostBridge) {
    this.setupListeners();
  }

  private setupListeners(): void {
    this.bridge.on('provideTextDocumentContent', async (data: any) => {
      const provider = this.providers.get(data.scheme);
      if (provider) {
        const content = await provider.provideTextDocumentContent(
          Uri.parse(data.uri),
          data.token
        );
        this.bridge.send('textDocumentContentResult', {
          requestId: data.requestId,
          content
        });
      }
    });
  }

  registerTextDocumentContentProvider(
    scheme: string,
    provider: TextDocumentContentProvider
  ): any {
    this.providers.set(scheme, provider);

    this.bridge.send('registerTextDocumentContentProvider', { scheme });

    return {
      dispose: () => {
        this.providers.delete(scheme);
        this.bridge.send('unregisterTextDocumentContentProvider', { scheme });
      }
    };
  }
}
