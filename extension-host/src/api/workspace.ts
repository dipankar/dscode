/**
 * VS Code Workspace API
 *
 * Handles workspace folder management, file operations, and configuration
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter } from './events';
import * as path from 'path';
import * as fs from 'fs';

export interface WorkspaceFolder {
  uri: { fsPath: string; scheme: string };
  name: string;
  index: number;
}

export class WorkspaceAPI {
  private workspaceFolders: WorkspaceFolder[] = [];

  // Event emitters
  private _onDidChangeTextDocument = new EventEmitter<any>();
  private _onDidSaveTextDocument = new EventEmitter<any>();
  private _onDidOpenTextDocument = new EventEmitter<any>();
  private _onDidCloseTextDocument = new EventEmitter<any>();
  private _onDidChangeWorkspaceFolders = new EventEmitter<any>();
  private _onDidChangeConfiguration = new EventEmitter<any>();

  // Events
  readonly onDidChangeTextDocument = this._onDidChangeTextDocument.event;
  readonly onDidSaveTextDocument = this._onDidSaveTextDocument.event;
  readonly onDidOpenTextDocument = this._onDidOpenTextDocument.event;
  readonly onDidCloseTextDocument = this._onDidCloseTextDocument.event;
  readonly onDidChangeWorkspaceFolders = this._onDidChangeWorkspaceFolders.event;
  readonly onDidChangeConfiguration = this._onDidChangeConfiguration.event;

  constructor(private bridge: ExtensionHostBridge) {
    // Request workspace folders from main app
    this.loadWorkspaceFolders();
    this.setupEventListeners();
  }

  private setupEventListeners(): void {
    this.bridge.on('textDocumentChanged', (data: any) => {
      this._onDidChangeTextDocument.fire(data);
    });

    this.bridge.on('textDocumentSaved', (data: any) => {
      this._onDidSaveTextDocument.fire(data);
    });

    this.bridge.on('textDocumentOpened', (data: any) => {
      this._onDidOpenTextDocument.fire(data);
    });

    this.bridge.on('textDocumentClosed', (data: any) => {
      this._onDidCloseTextDocument.fire(data);
    });

    this.bridge.on('workspaceFoldersChanged', (data: any) => {
      this._onDidChangeWorkspaceFolders.fire(data);
    });

    this.bridge.on('configurationChanged', (data: any) => {
      this._onDidChangeConfiguration.fire(data);
    });
  }

  private async loadWorkspaceFolders() {
    try {
      const response = await this.bridge.request('workspace-get-folders', {});
      // Handle response - it's already an array from Tauri
      const folders = Array.isArray(response) ? response : [];
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
        console.error(`[Workspace] Update config: ${key} = ${value}`);
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
