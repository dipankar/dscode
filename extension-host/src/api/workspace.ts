/**
 * VS Code Workspace API
 *
 * Handles workspace folder management, file operations, and configuration
 */

import { ExtensionHostBridge } from '../bridge';
import { Event, EventEmitter, Disposable } from './events';
import { Uri } from './uri';
import { WorkspaceEdit, TextEdit } from './textEditor';
import { Range } from './textDocument';
import * as path from 'path';
import * as fs from 'fs';

// Types for workspace edit serialization
interface WorkspaceEditEntry {
  uri: string;
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  newText: string;
}

interface FileOperation {
  uri: string;
  options?: {
    overwrite?: boolean;
    ignoreIfExists?: boolean;
    ignoreIfNotExists?: boolean;
    recursive?: boolean;
  };
}

interface RenameOperation {
  oldUri: string;
  newUri: string;
  options?: {
    overwrite?: boolean;
    ignoreIfExists?: boolean;
  };
}

export interface WorkspaceFolder {
  uri: { fsPath: string; scheme: string };
  name: string;
  index: number;
}

export class WorkspaceAPI {
  private workspaceFolders: WorkspaceFolder[] = [];
  private configurationData: any = {};
  private fileWatchers = new Map<string, FileSystemWatcherImpl>();

  // Event emitters
  private _onDidChangeTextDocument = new EventEmitter<any>();
  private _onDidSaveTextDocument = new EventEmitter<any>();
  private _onDidOpenTextDocument = new EventEmitter<any>();
  private _onDidCloseTextDocument = new EventEmitter<any>();
  private _onDidChangeWorkspaceFolders = new EventEmitter<any>();
  private _onDidChangeConfiguration = new EventEmitter<any>();
  private _onDidGrantWorkspaceTrust = new EventEmitter<void>();

  // Events
  readonly onDidChangeTextDocument = this._onDidChangeTextDocument.event;
  readonly onDidSaveTextDocument = this._onDidSaveTextDocument.event;
  readonly onDidOpenTextDocument = this._onDidOpenTextDocument.event;
  readonly onDidCloseTextDocument = this._onDidCloseTextDocument.event;
  readonly onDidChangeWorkspaceFolders = this._onDidChangeWorkspaceFolders.event;
  readonly onDidChangeConfiguration = this._onDidChangeConfiguration.event;
  readonly onDidGrantWorkspaceTrust = this._onDidGrantWorkspaceTrust.event;

  // Workspace trust - always trusted for now
  readonly isTrusted: boolean = true;

  constructor(private bridge: ExtensionHostBridge) {
    // Request workspace folders from main app
    this.loadWorkspaceFolders();
    this.loadConfiguration();
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
      void this.loadConfiguration();
      const section: string | undefined = data?.section || undefined;
      const key: string | undefined = data?.key || undefined;
      const changedPath = [section, key].filter(Boolean).join('.');

      this._onDidChangeConfiguration.fire({
        affectsConfiguration: (testSection?: string) => {
          if (!testSection) {
            return true;
          }
          if (!changedPath) {
            return true;
          }
          return (
            changedPath === testSection ||
            changedPath.startsWith(`${testSection}.`)
          );
        },
      });
    });

    this.bridge.on('fsWatcher:event', (data: any) => {
      this.handleFileSystemWatcherEvent(data);
    });
  }

  private async loadConfiguration() {
    try {
      const response = await this.bridge.request('workspace-get-configuration', {
        section: null,
      }) as { config?: any } | null;
      this.configurationData = response?.config ?? {};
    } catch (error) {
      console.error('[Workspace] Failed to load configuration:', error);
      this.configurationData = {};
    }
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
    }) as { files: string[] };

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
   * Apply a workspace edit
   * This allows making edits to multiple files at once
   */
  async applyEdit(edit: WorkspaceEdit): Promise<boolean> {
    console.log('[Workspace] Applying workspace edit');

    try {
      // Serialize the workspace edit
      const edits: WorkspaceEditEntry[] = [];

      for (const [uri, textEdits] of edit.entries()) {
        const uriPath = typeof uri === 'string' ? uri : (uri as any).path || (uri as any).fsPath;

        for (const textEdit of textEdits) {
          edits.push({
            uri: uriPath,
            range: {
              start: {
                line: textEdit.range.start.line,
                character: textEdit.range.start.character
              },
              end: {
                line: textEdit.range.end.line,
                character: textEdit.range.end.character
              }
            },
            newText: textEdit.newText
          });
        }
      }

      // Also handle file operations (create, delete, rename)
      const createFiles: FileOperation[] = [];
      const deleteFiles: FileOperation[] = [];
      const renameFiles: RenameOperation[] = [];

      // Check if edit has file operations (VS Code 1.48+)
      if (typeof (edit as any)._fileOperations !== 'undefined') {
        const fileOps = (edit as any)._fileOperations as Map<string, any>;
        for (const [opUri, op] of fileOps) {
          if (op.type === 'create') {
            createFiles.push({
              uri: opUri,
              options: op.options
            });
          } else if (op.type === 'delete') {
            deleteFiles.push({
              uri: opUri,
              options: op.options
            });
          } else if (op.type === 'rename') {
            renameFiles.push({
              oldUri: opUri,
              newUri: op.newUri,
              options: op.options
            });
          }
        }
      }

      const result = await this.bridge.request('workspace-apply-edit', {
        edits,
        createFiles,
        deleteFiles,
        renameFiles
      }) as { success: boolean; failureReason?: string };

      if (!result.success) {
        console.error('[Workspace] Apply edit failed:', result.failureReason);
        return false;
      }

      return true;
    } catch (error) {
      console.error('[Workspace] Failed to apply workspace edit:', error);
      return false;
    }
  }

  /**
   * Get configuration
   */
  getConfiguration(section?: string): any {
    return WorkspaceConfiguration(this, section);
  }

  private getSectionData(section?: string): any {
    if (!section || section.length === 0) {
      return this.configurationData;
    }

    const segments = section.split('.').filter(Boolean);
    let current: any = this.configurationData;
    for (const segment of segments) {
      if (current && typeof current === 'object' && segment in current) {
        current = current[segment];
      } else {
        return undefined;
      }
    }
    return current;
  }

  getConfigurationValue(section: string | undefined, key: string, defaultValue?: any): any {
    const base = this.getSectionData(section);
    if (base === undefined || base === null) {
      return defaultValue;
    }

    const segments = key.split('.').filter(Boolean);
    let current: any = base;
    for (const segment of segments) {
      if (current && typeof current === 'object' && segment in current) {
        current = current[segment];
      } else {
        return defaultValue;
      }
    }

    return current === undefined ? defaultValue : current;
  }

  async updateConfiguration(section: string | undefined, key: string, value: any, configurationTarget?: any): Promise<void> {
    const payloadValue = value === undefined ? null : value;

    await this.bridge.request('workspace-update-configuration', {
      section: section ?? null,
      key,
      value: payloadValue,
      target: configurationTarget ?? 'global',
    });
    await this.loadConfiguration();
  }

  /**
   * Create file system watcher
   */
  createFileSystemWatcher(
    globPattern: string,
    ignoreCreateEvents?: boolean,
    ignoreChangeEvents?: boolean,
    ignoreDeleteEvents?: boolean
  ): FileSystemWatcherImpl {
    const pattern = globPattern || '**/*';
    const watcherId = `fsWatcher_${Date.now()}_${Math.random().toString(36).slice(2)}`;

    void this.bridge
      .request('workspace-register-watcher', {
        id: watcherId,
        globPattern: pattern,
        ignoreCreateEvents: !!ignoreCreateEvents,
        ignoreChangeEvents: !!ignoreChangeEvents,
        ignoreDeleteEvents: !!ignoreDeleteEvents,
      })
      .catch((error: any) => {
        console.error('[Workspace] Failed to register file watcher:', error);
      });

    const watcher = new FileSystemWatcherImpl(
      this,
      this.bridge,
      watcherId,
      !!ignoreCreateEvents,
      !!ignoreChangeEvents,
      !!ignoreDeleteEvents
    );

    this.fileWatchers.set(watcherId, watcher);
    return watcher;
  }

  private handleFileSystemWatcherEvent(payload: any) {
    const id = payload?.id;
    const event = payload?.event;
    const pathValue = payload?.path;
    if (!id || !event || !pathValue) {
      return;
    }

    const watcher = this.fileWatchers.get(id);
    watcher?.handleHostEvent(event, pathValue);
  }

  removeWatcher(id: string) {
    this.fileWatchers.delete(id);
  }
}

class FileSystemWatcherImpl implements Disposable {
  private disposed = false;
  private _onDidCreate = new EventEmitter<Uri>();
  private _onDidChange = new EventEmitter<Uri>();
  private _onDidDelete = new EventEmitter<Uri>();

  readonly onDidCreate = this._onDidCreate.event;
  readonly onDidChange = this._onDidChange.event;
  readonly onDidDelete = this._onDidDelete.event;

  constructor(
    private workspace: WorkspaceAPI,
    private bridge: ExtensionHostBridge,
    private id: string,
    private ignoreCreate: boolean,
    private ignoreChange: boolean,
    private ignoreDelete: boolean
  ) {}

  handleHostEvent(event: string, path: string) {
    if (!path) {
      return;
    }

    const uri = Uri.file(path);

    switch (event) {
      case 'created':
        if (!this.ignoreCreate) {
          this._onDidCreate.fire(uri);
        }
        break;
      case 'changed':
        if (!this.ignoreChange) {
          this._onDidChange.fire(uri);
        }
        break;
      case 'deleted':
        if (!this.ignoreDelete) {
          this._onDidDelete.fire(uri);
        }
        break;
      default:
        break;
    }
  }

  dispose(): void {
    if (this.disposed) {
      return;
    }

    this.disposed = true;
    this.workspace.removeWatcher(this.id);

    void this.bridge.request('workspace-unregister-watcher', { id: this.id }).catch((error: any) => {
      console.error('[Workspace] Failed to unregister file watcher:', error);
    });

    this._onDidCreate.dispose();
    this._onDidChange.dispose();
    this._onDidDelete.dispose();
  }
}

// Empty configuration proxy for undefined configuration values
// This allows chained access like config.pylance.indexing.subkey to return undefined
// instead of throwing "Cannot read properties of undefined"
const EMPTY_CONFIG: any = new Proxy({}, {
  get(_target: any, prop: string | symbol): any {
    // Return undefined for primitive conversions and known methods
    if (typeof prop === 'symbol') return undefined;
    if (prop === 'toString' || prop === 'valueOf' || prop === 'toJSON') return () => undefined;
    // Return itself for any other property access to support deep chaining
    return EMPTY_CONFIG;
  },
  has(): boolean {
    return false;
  }
});

class WorkspaceConfigurationImpl {
  constructor(private workspace: WorkspaceAPI, private section?: string) {}

  get<T = any>(key: string, defaultValue?: T): T {
    const value = this.workspace.getConfigurationValue(this.section, key, defaultValue);
    // Return the value or default - don't use EMPTY_CONFIG here as it breaks array methods
    // Extensions are expected to handle undefined config values
    return value as T;
  }

  has(key: string): boolean {
    return this.workspace.getConfigurationValue(this.section, key, undefined) !== undefined;
  }

  update(key: string, value: any, configurationTarget?: any): Promise<void> {
    return this.workspace.updateConfiguration(this.section, key, value, configurationTarget);
  }

  inspect<T>(key: string): {
    key: string;
    defaultValue?: T;
    globalValue?: T;
    workspaceValue?: T;
    workspaceFolderValue?: T;
    defaultLanguageValue?: T;
    globalLanguageValue?: T;
    workspaceLanguageValue?: T;
    workspaceFolderLanguageValue?: T;
    languageIds?: string[];
  } | undefined {
    const value = this.get<T>(key);
    return {
      key: this.section ? `${this.section}.${key}` : key,
      globalValue: value,
      workspaceValue: undefined,
      workspaceFolderValue: undefined,
    };
  }
}

/**
 * Create a WorkspaceConfiguration with Proxy support for bracket notation access
 * VS Code allows both config.get('key') and config['key'] access patterns
 */
function WorkspaceConfiguration(workspace: WorkspaceAPI, section?: string): any {
  const impl = new WorkspaceConfigurationImpl(workspace, section);

  return new Proxy(impl, {
    get(target: WorkspaceConfigurationImpl, prop: string | symbol): any {
      // Handle known methods first
      if (prop === 'get') return target.get.bind(target);
      if (prop === 'has') return target.has.bind(target);
      if (prop === 'update') return target.update.bind(target);
      if (prop === 'inspect') return target.inspect.bind(target);

      // Handle symbol properties (like Symbol.toStringTag)
      if (typeof prop === 'symbol') return undefined;

      // For any other property access, use get() to retrieve the value
      // This enables config['pylance'] style access
      // Note: target.get() already returns EMPTY_CONFIG for undefined values
      return target.get(prop);
    },
    has(target: WorkspaceConfigurationImpl, prop: string | symbol): boolean {
      if (typeof prop === 'symbol') return false;
      return target.has(prop as string);
    }
  });
}
