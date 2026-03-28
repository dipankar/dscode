/**
 * FileSystem API (workspace.fs)
 *
 * File system operations for workspace
 */

import { ExtensionHostBridge } from '../bridge';
import { Uri } from './uri';
import { FileType, FileStat, FilePermission, FileSystemError } from './common';

export { FileType, FileStat, FilePermission, FileSystemError };

function classifyFsError(error: Error, uri: Uri): FileSystemError {
  const msg = error.message || '';
  if (msg.includes('No such file') || msg.includes('not found') || msg.includes('ENOENT')) {
    return FileSystemError.FileNotFound(uri.toString());
  }
  if (msg.includes('Already exists') || msg.includes('EEXIST') || msg.includes('EntryExists')) {
    return FileSystemError.FileExists(uri.toString());
  }
  if (
    msg.includes('Not a directory') ||
    msg.includes('ENOTDIR') ||
    msg.includes('EntryNotADirectory')
  ) {
    return FileSystemError.FileNotADirectory(uri.toString());
  }
  if (
    msg.includes('Is a directory') ||
    msg.includes('EISDIR') ||
    msg.includes('EntryIsADirectory')
  ) {
    return FileSystemError.FileIsADirectory(uri.toString());
  }
  if (
    msg.includes('Permission denied') ||
    msg.includes('EACCES') ||
    msg.includes('NoPermissions') ||
    msg.includes('Access denied')
  ) {
    return FileSystemError.NoPermissions(uri.toString());
  }
  return FileSystemError.Unavailable(uri.toString());
}

export class FileSystemAPI {
  constructor(private bridge: ExtensionHostBridge) {}

  async stat(uri: Uri): Promise<FileStat> {
    try {
      const result = (await this.bridge.request('fsStat', {
        uri: uri.toString(),
      })) as { stat: FileStat };
      return result.stat;
    } catch (error: any) {
      throw classifyFsError(error instanceof Error ? error : new Error(String(error)), uri);
    }
  }

  async readDirectory(uri: Uri): Promise<[string, FileType][]> {
    try {
      const result = (await this.bridge.request('fsReadDirectory', {
        uri: uri.toString(),
      })) as { entries: [string, FileType][] };
      return result.entries;
    } catch (error: any) {
      throw classifyFsError(error instanceof Error ? error : new Error(String(error)), uri);
    }
  }

  async createDirectory(uri: Uri): Promise<void> {
    try {
      await this.bridge.request('fsCreateDirectory', {
        uri: uri.toString(),
      });
    } catch (error: any) {
      throw classifyFsError(error instanceof Error ? error : new Error(String(error)), uri);
    }
  }

  async readFile(uri: Uri): Promise<Uint8Array> {
    try {
      const result = (await this.bridge.request('fsReadFile', {
        uri: uri.toString(),
      })) as { data: number[] };
      return new Uint8Array(result.data);
    } catch (error: any) {
      throw classifyFsError(error instanceof Error ? error : new Error(String(error)), uri);
    }
  }

  async writeFile(uri: Uri, content: Uint8Array): Promise<void> {
    try {
      await this.bridge.request('fsWriteFile', {
        uri: uri.toString(),
        content: Array.from(content),
      });
    } catch (error: any) {
      throw classifyFsError(error instanceof Error ? error : new Error(String(error)), uri);
    }
  }

  async delete(uri: Uri, options?: { recursive?: boolean; useTrash?: boolean }): Promise<void> {
    try {
      await this.bridge.request('fsDelete', {
        uri: uri.toString(),
        options,
      });
    } catch (error: any) {
      throw classifyFsError(error instanceof Error ? error : new Error(String(error)), uri);
    }
  }

  async rename(source: Uri, target: Uri, options?: { overwrite?: boolean }): Promise<void> {
    try {
      await this.bridge.request('fsRename', {
        oldUri: source.toString(),
        newUri: target.toString(),
        options,
      });
    } catch (error: any) {
      throw classifyFsError(error instanceof Error ? error : new Error(String(error)), source);
    }
  }

  async copy(source: Uri, target: Uri, options?: { overwrite?: boolean }): Promise<void> {
    try {
      await this.bridge.request('fsCopy', {
        source: source.toString(),
        destination: target.toString(),
        options,
      });
    } catch (error: any) {
      throw classifyFsError(error instanceof Error ? error : new Error(String(error)), source);
    }
  }

  isWritableFileSystem(scheme: string): boolean | undefined {
    return scheme !== 'vscode' && scheme !== 'untitled';
  }
}

// TextDocumentContentProvider
export interface TextDocumentContentProvider {
  onDidChange?: any;
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
        const content = await provider.provideTextDocumentContent(Uri.parse(data.uri), data.token);
        this.bridge.send('textDocumentContentResult', {
          requestId: data.requestId,
          content,
        });
      }
    });
  }

  registerTextDocumentContentProvider(scheme: string, provider: TextDocumentContentProvider): any {
    this.providers.set(scheme, provider);

    this.bridge.send('registerTextDocumentContentProvider', { scheme });

    return {
      dispose: () => {
        this.providers.delete(scheme);
        this.bridge.send('unregisterTextDocumentContentProvider', { scheme });
      },
    };
  }
}
