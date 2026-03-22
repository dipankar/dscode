import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export enum FileType {
  Unknown = 'unknown',
  File = 'file',
  Directory = 'directory',
  SymbolicLink = 'symboliclink',
}

export enum FileChangeType {
  Created = 'created',
  Changed = 'changed',
  Deleted = 'deleted',
}

export interface FileSystemProvider {
  id: string;
  owner: string;
  scheme: string;
  is_case_sensitive: boolean;
  is_readonly: boolean;
}

export interface FileWatcher {
  id: string;
  owner: string;
  glob_pattern: string;
  ignore_create_events: boolean;
  ignore_change_events: boolean;
  ignore_delete_events: boolean;
}

export interface FileChangeEvent {
  uri: string;
  type: FileChangeType;
}

export interface FileStat {
  uri: string;
  type: FileType;
  ctime: number;
  mtime: number;
  size: number;
  permissions?: number;
}

export interface DirectoryEntry {
  name: string;
  type: FileType;
}

export interface CreateFileOptions {
  overwrite: boolean;
  ignore_if_exists: boolean;
}

export interface DeleteFileOptions {
  recursive: boolean;
  ignore_if_not_exists: boolean;
}

export interface RenameFileOptions {
  overwrite: boolean;
}

export interface TextEditItem {
  range: TextRange;
  new_text: string;
}

export interface TextRange {
  start: TextPosition;
  end: TextPosition;
}

export interface TextPosition {
  line: number;
  character: number;
}

export type FileEdit =
  | { type: 'create'; uri: string; options?: CreateFileOptions }
  | { type: 'delete'; uri: string; options?: DeleteFileOptions }
  | { type: 'rename'; old_uri: string; new_uri: string; options?: RenameFileOptions }
  | { type: 'text'; uri: string; edits: TextEditItem[] };

export interface WorkspaceEdit {
  id: string;
  edits: FileEdit[];
}

/**
 * File System API manager
 */
export class FileSystemManager {
  private providers: Map<string, FileSystemProvider> = new Map();
  private watchers: Map<string, FileWatcher> = new Map();
  private watcherCallbacks: Map<string, Array<(event: FileChangeEvent) => void>> = new Map();
  private onDidChangeFileCallbacks: Array<(event: FileChangeEvent) => void> = [];

  constructor() {}

  /**
   * Initialize file system manager
   */
  async initialize(): Promise<void> {
    // Listen for provider registration events
    await listen<FileSystemProvider>('filesystem-provider-registered', (event) => {
      this.providers.set(event.payload.scheme, event.payload);
    });

    // Listen for provider unregistration events
    await listen<string>('filesystem-provider-unregistered', (event) => {
      this.providers.delete(event.payload);
    });

    // Listen for watcher creation events
    await listen<FileWatcher>('file-watcher-created', (event) => {
      this.watchers.set(event.payload.id, event.payload);
    });

    // Listen for watcher disposal events
    await listen<string>('file-watcher-disposed', (event) => {
      this.watchers.delete(event.payload);
      this.watcherCallbacks.delete(event.payload);
    });

    console.log('[FileSystem] Initialized');
  }

  // ===== File Operations =====

  /**
   * Read file as binary
   */
  async readFile(uri: string): Promise<Uint8Array> {
    const data = await invoke<number[]>('fs_read_file', { uri });
    return new Uint8Array(data);
  }

  /**
   * Read file as text
   */
  async readTextFile(uri: string): Promise<string> {
    const data = await this.readFile(uri);
    return new TextDecoder().decode(data);
  }

  /**
   * Write file as binary
   */
  async writeFile(uri: string, content: Uint8Array): Promise<void> {
    await invoke('fs_write_file', { uri, content: Array.from(content) });
  }

  /**
   * Write file as text
   */
  async writeTextFile(uri: string, content: string): Promise<void> {
    const encoded = new TextEncoder().encode(content);
    await this.writeFile(uri, encoded);
  }

  /**
   * Get file stat
   */
  async stat(uri: string): Promise<FileStat> {
    return await invoke<FileStat>('fs_stat', { uri });
  }

  /**
   * Read directory
   */
  async readDirectory(uri: string): Promise<DirectoryEntry[]> {
    return await invoke<DirectoryEntry[]>('fs_read_directory', { uri });
  }

  /**
   * Create directory
   */
  async createDirectory(uri: string): Promise<void> {
    await invoke('fs_create_directory', { uri });
  }

  /**
   * Delete file or directory
   */
  async delete(uri: string, recursive: boolean = false): Promise<void> {
    await invoke('fs_delete', { uri, recursive });
  }

  /**
   * Rename file or directory
   */
  async rename(oldUri: string, newUri: string): Promise<void> {
    await invoke('fs_rename', { oldUri, newUri });
  }

  /**
   * Copy file or directory
   */
  async copy(sourceUri: string, destinationUri: string): Promise<void> {
    await invoke('fs_copy', { sourceUri, destinationUri });
  }

  // ===== File System Providers =====

  /**
   * Register file system provider
   */
  async registerFileSystemProvider(
    scheme: string,
    providerId: string,
    owner: string,
    isCaseSensitive: boolean = true,
    isReadonly: boolean = false
  ): Promise<string> {
    const provider: FileSystemProvider = {
      id: providerId,
      owner,
      scheme,
      is_case_sensitive: isCaseSensitive,
      is_readonly: isReadonly,
    };

    return await invoke<string>('register_file_system_provider', { provider });
  }

  /**
   * Unregister file system provider
   */
  async unregisterFileSystemProvider(scheme: string): Promise<void> {
    await invoke('unregister_file_system_provider', { scheme });
  }

  /**
   * Get file system provider
   */
  async getFileSystemProvider(scheme: string): Promise<FileSystemProvider | null> {
    try {
      return await invoke<FileSystemProvider>('get_file_system_provider', { scheme });
    } catch {
      return null;
    }
  }

  /**
   * Get all file system providers
   */
  async getAllFileSystemProviders(): Promise<FileSystemProvider[]> {
    return await invoke<FileSystemProvider[]>('get_all_file_system_providers');
  }

  // ===== File Watchers =====

  /**
   * Create file watcher
   */
  async createFileWatcher(
    watcherId: string,
    owner: string,
    globPattern: string,
    ignoreCreateEvents: boolean = false,
    ignoreChangeEvents: boolean = false,
    ignoreDeleteEvents: boolean = false
  ): Promise<string> {
    const watcher: FileWatcher = {
      id: watcherId,
      owner,
      glob_pattern: globPattern,
      ignore_create_events: ignoreCreateEvents,
      ignore_change_events: ignoreChangeEvents,
      ignore_delete_events: ignoreDeleteEvents,
    };

    const id = await invoke<string>('create_file_watcher', { watcher });

    // Set up event listener for this watcher
    await listen<FileChangeEvent>(`file-watcher-event:${id}`, (event) => {
      this.notifyWatcherCallbacks(id, event.payload);
      this.notifyFileChangeCallbacks(event.payload);
    });

    return id;
  }

  /**
   * Dispose file watcher
   */
  async disposeFileWatcher(watcherId: string): Promise<void> {
    await invoke('dispose_file_watcher', { watcherId });
  }

  /**
   * Get file watcher
   */
  async getFileWatcher(watcherId: string): Promise<FileWatcher | null> {
    try {
      return await invoke<FileWatcher>('get_file_watcher', { watcherId });
    } catch {
      return null;
    }
  }

  /**
   * Get all file watchers
   */
  async getAllFileWatchers(): Promise<FileWatcher[]> {
    return await invoke<FileWatcher[]>('get_all_file_watchers');
  }

  /**
   * Subscribe to file watcher events
   */
  onFileWatcherEvent(watcherId: string, callback: (event: FileChangeEvent) => void): () => void {
    if (!this.watcherCallbacks.has(watcherId)) {
      this.watcherCallbacks.set(watcherId, []);
    }

    const callbacks = this.watcherCallbacks.get(watcherId)!;
    callbacks.push(callback);

    // Return unsubscribe function
    return () => {
      const index = callbacks.indexOf(callback);
      if (index > -1) {
        callbacks.splice(index, 1);
      }
    };
  }

  /**
   * Subscribe to all file change events
   */
  onDidChangeFile(callback: (event: FileChangeEvent) => void): () => void {
    this.onDidChangeFileCallbacks.push(callback);

    // Return unsubscribe function
    return () => {
      const index = this.onDidChangeFileCallbacks.indexOf(callback);
      if (index > -1) {
        this.onDidChangeFileCallbacks.splice(index, 1);
      }
    };
  }

  // ===== Workspace Edits =====

  /**
   * Apply workspace edit
   */
  async applyWorkspaceEdit(edit: WorkspaceEdit): Promise<void> {
    await invoke('apply_workspace_edit', { edit });
  }

  /**
   * Create a workspace edit
   */
  createWorkspaceEdit(): WorkspaceEditBuilder {
    return new WorkspaceEditBuilder();
  }

  /**
   * Clear file system data for owner
   */
  async clearFileSystemData(owner: string): Promise<void> {
    await invoke('clear_filesystem_data', { owner });
  }

  // ===== Private Methods =====

  /**
   * Notify watcher-specific callbacks
   */
  private notifyWatcherCallbacks(watcherId: string, event: FileChangeEvent): void {
    const callbacks = this.watcherCallbacks.get(watcherId);
    if (!callbacks) return;

    for (const callback of callbacks) {
      try {
        callback(event);
      } catch (error) {
        console.error('[FileSystem] Error in watcher callback:', error);
      }
    }
  }

  /**
   * Notify global file change callbacks
   */
  private notifyFileChangeCallbacks(event: FileChangeEvent): void {
    for (const callback of this.onDidChangeFileCallbacks) {
      try {
        callback(event);
      } catch (error) {
        console.error('[FileSystem] Error in file change callback:', error);
      }
    }
  }
}

/**
 * Workspace edit builder
 */
export class WorkspaceEditBuilder {
  private edits: FileEdit[] = [];
  private editId: string;

  constructor() {
    this.editId = `edit-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Add create file edit
   */
  createFile(uri: string, options?: CreateFileOptions): this {
    this.edits.push({ type: 'create', uri, options });
    return this;
  }

  /**
   * Add delete file edit
   */
  deleteFile(uri: string, options?: DeleteFileOptions): this {
    this.edits.push({ type: 'delete', uri, options });
    return this;
  }

  /**
   * Add rename file edit
   */
  renameFile(oldUri: string, newUri: string, options?: RenameFileOptions): this {
    this.edits.push({ type: 'rename', old_uri: oldUri, new_uri: newUri, options });
    return this;
  }

  /**
   * Add text edit
   */
  textEdit(uri: string, edits: TextEditItem[]): this {
    this.edits.push({ type: 'text', uri, edits });
    return this;
  }

  /**
   * Build the workspace edit
   */
  build(): WorkspaceEdit {
    return {
      id: this.editId,
      edits: this.edits,
    };
  }
}

// Export singleton instance
export const fileSystemManager = new FileSystemManager();
