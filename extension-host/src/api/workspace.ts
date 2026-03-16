/**
 * VS Code Workspace API
 *
 * Handles workspace folder management, file operations, and configuration
 */

import { ExtensionHostBridge } from '../bridge';
import * as path from 'path';
import * as fs from 'fs';

export interface WorkspaceFolder {
  uri: { fsPath: string; scheme: string };
  name: string;
  index: number;
}

export class WorkspaceAPI {
  private workspaceFolders: WorkspaceFolder[] = [];

  constructor(private bridge: ExtensionHostBridge) {
    // Request workspace folders from main app
    this.loadWorkspaceFolders();
  }

  private async loadWorkspaceFolders() {
    try {
      const folders = await this.bridge.request('workspace-get-folders', {});
      this.workspaceFolders = folders.map((folderPath: string, index: number) => ({
        uri: {
          fsPath: folderPath,
          scheme: 'file',
        },
        name: path.basename(folderPath),
        index,
      }));
    } catch (error) {
      console.error('[Workspace] Failed to load workspace folders:', error);
      this.workspaceFolders = [];
    }
  }

  /**
   * Get workspace folders
   */
  get folders(): readonly WorkspaceFolder[] | undefined {
    return this.workspaceFolders.length > 0 ? this.workspaceFolders : undefined;
  }

  /**
   * Get workspace name
   */
  get name(): string | undefined {
    if (this.workspaceFolders.length > 0) {
      return this.workspaceFolders[0].name;
    }
    return undefined;
  }

  /**
   * Get workspace root path (deprecated, use workspaceFolders instead)
   */
  get rootPath(): string | undefined {
    if (this.workspaceFolders.length > 0) {
      return this.workspaceFolders[0].uri.fsPath;
    }
    return undefined;
  }

  /**
   * Find files in workspace
   */
  async findFiles(
    include: string,
    exclude?: string,
    maxResults?: number
  ): Promise<{ fsPath: string; scheme: string }[]> {
    const result = await this.bridge.request('workspace-find-files', {
      include,
      exclude,
      maxResults,
    });

    return result.files.map((file: string) => ({
      fsPath: file,
      scheme: 'file',
    }));
  }

  /**
   * Open a text document
   */
  async openTextDocument(uri: string | { fsPath: string }): Promise<any> {
    const filePath = typeof uri === 'string' ? uri : uri.fsPath;

    const content = fs.readFileSync(filePath, 'utf-8');

    return {
      uri: { fsPath: filePath, scheme: 'file' },
      fileName: filePath,
      getText: () => content,
      lineCount: content.split('\n').length,
      // Add more TextDocument properties as needed
    };
  }

  /**
   * Save a text document
   */
  async saveTextDocument(document: any): Promise<boolean> {
    try {
      await this.bridge.request('workspace-save-document', {
        path: document.fileName,
        content: document.getText(),
      });
      return true;
    } catch (error) {
      console.error('[Workspace] Failed to save document:', error);
      return false;
    }
  }

  /**
   * Get configuration
   */
  getConfiguration(section?: string): any {
    // For now, return a simple config object
    // In the future, this will fetch from actual settings
    return {
      get: (key: string, defaultValue?: any) => {
        // TODO: Implement actual configuration storage
        return defaultValue;
      },
      has: (key: string) => {
        return false;
      },
      update: async (key: string, value: any, configurationTarget?: any) => {
        // TODO: Implement configuration update
        console.log(`[Workspace] Update config: ${key} = ${value}`);
      },
    };
  }

  /**
   * Create file system watcher
   */
  createFileSystemWatcher(
    globPattern: string,
    ignoreCreateEvents?: boolean,
    ignoreChangeEvents?: boolean,
    ignoreDeleteEvents?: boolean
  ): any {
    // TODO: Implement file watching
    return {
      onDidCreate: (callback: Function) => ({ dispose: () => {} }),
      onDidChange: (callback: Function) => ({ dispose: () => {} }),
      onDidDelete: (callback: Function) => ({ dispose: () => {} }),
      dispose: () => {},
    };
  }
}
